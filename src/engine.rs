//! Worker-движок: audio-callback → кольцо → обработка на control-rate → метрики + OSC.

use crate::audio::AudioInput;
use crate::config::Source;
use crate::control::Controller;
use crate::detect::{BeatDetector, CounterBank};
use crate::dsp::{rms_power, BandBank, Spectrum};
use crate::diag;
use crate::osc_map::{build_snapshot, AnalysisFrame, TriggerState};
use crate::shared::{Metrics, Shared, SPECTRUM_DRAW_BINS};
use ringbuf::traits::{Consumer, Observer, Split};
use ringbuf::HeapRb;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

const FFT_SIZE: usize = 1024;
const RING_CAP: usize = 48000 * 2;

pub fn spawn(shared: Arc<Shared>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || run(shared))
}

fn run(shared: Arc<Shared>) {
    let mut sample_rate = 48000.0f32;
    let cfg0 = shared.config.load();

    let (mut audio, mut consumer, sr0, err0) = open_audio(&shared, sample_rate);
    sample_rate = sr0;
    if let Some(e) = err0 {
        shared.metrics.note_engine_error(Some(e));
    }

    let mut band_bank = BandBank::new(&cfg0.bands, sample_rate);
    let mut last_cfg_version = shared.config.version();
    let mut kick_det = BeatDetector::default();
    let mut snare_det = BeatDetector::default();
    let mut rythm_det = BeatDetector::default();
    let mut kick_counts = CounterBank::default();
    let mut snare_counts = CounterBank::default();
    let mut controller = Controller::default();
    let mut spectrum = Spectrum::new(FFT_SIZE);
    let mut trigger_state = TriggerState::default();
    let mut frame_id: u64 = 0;

    let mut frame: Vec<f32> = Vec::with_capacity(8192);

    let mut flux_peak = 1e-6f32;
    let mut kick_env = 0.0f32;
    let mut snare_env = 0.0f32;
    let mut rythm_env = 0.0f32;

    diag::info("engine", "движок запущен");

    while shared.running.load(Ordering::Acquire) {
        let loop_start = Instant::now();
        let cfg = shared.config.load();
        let rate = cfg.compute_rate_hz.clamp(30.0, 480.0);
        let dt = 1.0 / rate;
        let deadline = if cfg.osc.phase.sync_timeline {
            shared.timeline.next_compute_deadline(rate)
        } else {
            Instant::now() + std::time::Duration::from_secs_f32(dt)
        };

        if shared.restart_audio.swap(false, Ordering::AcqRel) {
            let (a, c, sr, err) = open_audio(&shared, sample_rate);
            audio = a;
            consumer = c;
            sample_rate = sr;
            let rcfg = shared.config.load();
            band_bank.redesign(&rcfg.bands, sample_rate);
            kick_det = BeatDetector::default();
            snare_det = BeatDetector::default();
            rythm_det = BeatDetector::default();
            kick_counts = CounterBank::default();
            snare_counts = CounterBank::default();
            trigger_state = TriggerState::default();
            shared.timeline.reset();
            diag::debug(
                "engine",
                format!("audio restart sr={sample_rate:.0} err={err:?}"),
            );
            shared.metrics.note_engine_error(err);
            diag::info("engine", "аудио перезапущено");
        }

        let cfg_version = shared.config.version();
        if cfg_version != last_cfg_version {
            band_bank.redesign(&cfg.bands, sample_rate);
            last_cfg_version = cfg_version;
            diag::debug("engine", format!("config v{cfg_version}: band_bank redesign"));
        }

        frame.clear();
        let ring_occupied = if let Some(cons) = consumer.as_mut() {
            let n = cons.occupied_len();
            while let Some(s) = cons.try_pop() {
                frame.push(s);
                if frame.len() >= 8192 {
                    break;
                }
            }
            n
        } else {
            0
        };
        let ring_fill = ring_occupied as f32 / RING_CAP as f32;

        if cfg.compressor.enabled {
            apply_compressor(&mut frame, &cfg.compressor.cfg);
        }
        let dsp_signal_rms = rms_power(&frame);
        let input_rms = dsp_signal_rms;

        let ctl = controller.step(input_rms, &cfg.control, dt);

        let adaptive_gains = [ctl.low_gain, ctl.mid_gain, ctl.high_gain];
        let (brms, levels) = band_bank.process_frame(
            &frame,
            &cfg.bands,
            adaptive_gains,
            cfg.control.enabled,
            dt,
        );

        if !frame.is_empty() {
            spectrum.push_block(&frame);
        }
        let centroid = cfg.spectral.centroid(spectrum.centroid_bins);
        let fms = cfg.spectral.fms(spectrum.energy);
        let sms = cfg.spectral.sms(spectrum.energy);

        let (kick_thr, snare_thr) = if cfg.control.enabled {
            (ctl.kick_thresh, ctl.snare_thresh)
        } else {
            (cfg.detectors[0].threshold, cfg.detectors[1].threshold)
        };
        let mut kd = cfg.detectors[0].clone();
        kd.threshold = kick_thr;
        let mut sd = cfg.detectors[1].clone();
        sd.threshold = snare_thr;
        let rd = cfg.detectors[2].clone();

        let peak_decay = (-dt / 3.0).exp();
        flux_peak = (flux_peak * peak_decay).max(spectrum.flux).max(1e-6);
        let flux_norm = (spectrum.flux / flux_peak).clamp(0.0, 1.0);

        let kick = kick_det.process(brms[0], &kd, dt);
        let snare = snare_det.process(brms[2], &sd, dt);
        let rythm = rythm_det.process(flux_norm, &rd, dt);

        let env_decay = (-dt / 0.12).exp();
        kick_env = (kick_env * env_decay).max(kick.1);
        snare_env = (snare_env * env_decay).max(snare.0.max(snare.1));
        rythm_env = (rythm_env * env_decay).max(rythm.1);

        let (k4, k8, k16) = kick_counts.process(kick.1);
        let (s4, s8, s16) = snare_counts.process(snare.1);
        let beat_phase = kick_counts.c4.phase();

        let dsp_rms = if cfg.dsp_rmspower {
            cfg.dsp_gain.apply(rms_power(&frame))
        } else {
            0.0
        };

        frame_id += 1;
        let analysis = AnalysisFrame {
            frame_id,
            t_mono: shared.timeline.mono_secs(),
            beat_phase,
            levels,
            kick,
            snare,
            rythm,
            centroid,
            fms,
            sms,
            triggers_kick: (k4, k8, k16),
            triggers_snare: (s4, s8, s16),
            dsp_rms,
        };

        let snapshot = build_snapshot(&analysis, &cfg, &mut trigger_state);
        let osc_channels = snapshot
            .channels
            .iter()
            .filter(|c| cfg.osc.sends(&c.address))
            .count();
        if !snapshot.pulses.is_empty() && !cfg.osc.phase.immediate_triggers {
            shared.trigger_queue.push(snapshot.pulses.clone());
        }
        shared.osc_out.publish(snapshot);

        let n = spectrum.mags.len().min(SPECTRUM_DRAW_BINS);
        let mut metrics = Metrics::default();
        metrics.device_name = audio.as_ref().map(|a| a.device_name.clone()).unwrap_or_else(|| "-".into());
        metrics.sample_rate = sample_rate;
        metrics.input_rms = input_rms;
        metrics.band_levels = levels;
        metrics.band_rms = brms;
        metrics.kick = kick;
        metrics.snare = snare;
        metrics.rythm = rythm;
        metrics.kick_env = kick_env;
        metrics.snare_env = snare_env;
        metrics.rythm_env = rythm_env;
        metrics.control = ctl;
        metrics.centroid = centroid;
        metrics.fms = fms;
        metrics.sms = sms;
        metrics.flux = spectrum.flux;
        metrics.dsp_rms = dsp_rms;
        metrics.beat_phase = beat_phase;
        metrics.kick_bar_pos = kick_counts.c4.count();
        metrics.snare_bar_pos = snare_counts.c4.count();
        metrics.compute_frame_id = frame_id;
        if n > 0 {
            metrics.spectrum[..n].copy_from_slice(&spectrum.mags[..n]);
        }
        metrics.spectrum_len = n;
        metrics.osc_channels = osc_channels;
        metrics.ringbuf_fill = ring_fill;
        metrics.compute_dt_ms = loop_start.elapsed().as_secs_f32() * 1000.0;
        shared.metrics.publish(metrics);

        let now = Instant::now();
        if deadline > now {
            std::thread::sleep(deadline - now);
        }
    }

    diag::info("engine", "движок остановлен");
}

fn open_audio(
    shared: &Arc<Shared>,
    fallback_sr: f32,
) -> (Option<AudioInput>, Option<ringbuf::HeapCons<f32>>, f32, Option<String>) {
    let cfg = shared.config.load();
    if cfg.input.source == Source::File {
        return (
            None,
            None,
            fallback_sr,
            Some("файловый вход не собран (feature file-input)".into()),
        );
    }
    let rb = HeapRb::<f32>::new(RING_CAP);
    let (prod, cons) = rb.split();
    match AudioInput::open(
        cfg.input.device.as_deref(),
        cfg.input.pulse_source.as_deref(),
        &cfg.input.channels_pick,
        prod,
    ) {
        Ok(inp) => {
            let sr = inp.sample_rate;
            (Some(inp), Some(cons), sr, None)
        }
        Err(e) => (None, None, fallback_sr, Some(format!("аудио: {e}"))),
    }
}

fn apply_compressor(frame: &mut [f32], cfg: &crate::config::CompressorCfg) {
    let thr = 10f32.powf(cfg.threshold_db / 20.0);
    let makeup = 10f32.powf(cfg.makeup_db / 20.0);
    let ratio = cfg.ratio.max(1e-3);
    for s in frame.iter_mut() {
        let a = s.abs();
        if a > thr {
            let over = a / thr;
            let comp = over.powf(1.0 / ratio - 1.0);
            *s *= comp;
        }
        *s *= makeup;
    }
}
