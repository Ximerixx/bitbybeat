//! Worker-движок: audio-callback → кольцо → обработка на control-rate 60 Гц → метрики + OSC.
//! (md_plans/01–06, 09 C12: разделение audio-rate и control-rate.)

use crate::audio::AudioInput;
use crate::config::{Config, Source};
use crate::control::Controller;
use crate::detect::{BeatDetector, CounterBank};
use crate::dsp::{rms_power, BandProcessor, Spectrum};
use crate::osc::OscSender;
use crate::shared::Shared;
use ringbuf::traits::{Consumer, Split};
use ringbuf::HeapRb;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

const FFT_SIZE: usize = 1024;
const SPECTRUM_DRAW_BINS: usize = 256;

pub fn spawn(shared: Arc<Shared>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || run(shared))
}

fn run(shared: Arc<Shared>) {
    let dt = 1.0 / crate::config::CONTROL_RATE_HZ;
    let tick = Duration::from_secs_f32(dt);

    let mut sample_rate = 48000.0f32;
    let cfg0 = shared.config.lock().unwrap().clone();

    // первичное открытие аудио
    let (mut audio, mut consumer, sr0, err0) = open_audio(&shared, sample_rate);
    sample_rate = sr0;
    if let Some(e) = err0 { shared.metrics.lock().unwrap().error = Some(e); }

    let mut bands: Vec<BandProcessor> = cfg0.bands.iter().map(|b| BandProcessor::new(b, sample_rate)).collect();
    let mut kick_det = BeatDetector::default();
    let mut snare_det = BeatDetector::default();
    let mut rythm_det = BeatDetector::default();
    let mut kick_counts = CounterBank::default();
    let mut snare_counts = CounterBank::default();
    let mut controller = Controller::default();
    let mut spectrum = Spectrum::new(FFT_SIZE);
    let mut osc: Option<OscSender> = OscSender::new(&cfg0.osc).ok();

    let mut frame: Vec<f32> = Vec::with_capacity(8192);

    // Пик-фолловер для нормировки спектрального flux (rythm) и огибающие ламп.
    let mut flux_peak = 1e-6f32;
    let mut kick_env = 0.0f32;
    let mut snare_env = 0.0f32;
    let mut rythm_env = 0.0f32;
    const ENV_DECAY: f32 = 0.86;

    while shared.running.load(Ordering::Relaxed) {
        let t0 = Instant::now();

        if shared.restart_audio.swap(false, Ordering::Relaxed) {
            let (a, c, sr, err) = open_audio(&shared, sample_rate);
            audio = a; consumer = c; sample_rate = sr;
            let cfg = shared.config.lock().unwrap().clone();
            for (b, bc) in bands.iter_mut().zip(&cfg.bands) { b.redesign(bc, sample_rate); }
            shared.metrics.lock().unwrap().error = err;
        }

        let cfg: Config = shared.config.lock().unwrap().clone();

        // синхронизировать число/дизайн полос при изменении конфига
        if bands.len() != cfg.bands.len() {
            bands = cfg.bands.iter().map(|b| BandProcessor::new(b, sample_rate)).collect();
        }

        // ── собрать накопленные сэмплы ──
        frame.clear();
        if let Some(cons) = consumer.as_mut() {
            while let Some(s) = cons.try_pop() {
                frame.push(s);
                if frame.len() >= 8192 { break; }
            }
        }

        // ── пре-обработка входа ──
        if cfg.compressor.enabled {
            apply_compressor(&mut frame, &cfg.compressor.cfg);
        }
        // R2: опциональный RMS-power во входную DSP-ветвь (для порогов/визуала)
        let dsp_signal_rms = rms_power(&frame);

        // ── общий RMS входа (для адаптива) ──
        let input_rms = dsp_signal_rms;

        // ── адаптивный контроль ──
        let ctl = controller.step(input_rms, &cfg.control, dt);

        // ── полосы ──
        let mut levels = [0.0f32; 3];
        let mut brms = [0.0f32; 3];
        for (i, (proc, bcfg)) in bands.iter_mut().zip(cfg.bands.iter()).enumerate().take(3) {
            // применяем адаптивный гейн, если контроль включён
            let mut eff = bcfg.clone();
            if cfg.control.enabled {
                eff.gain = match i { 0 => ctl.low_gain, 1 => ctl.mid_gain, _ => ctl.high_gain };
            }
            let (pre, out) = proc.process_block(&frame, &eff, dt);
            brms[i] = pre;
            levels[i] = out;
        }

        // ── спектр ──
        if !frame.is_empty() {
            spectrum.push_block(&frame);
        }
        let centroid = normalize(spectrum.centroid_bins, 18.0, 32.0); // md_plans/03
        let fms = normalize(spectrum.energy, 0.0, 1000.0);
        let sms = normalize(spectrum.energy, 100.0, 1800.0);

        // ── детекторы ──
        // kick/snare: порог подменяется адаптивом (сигмоида). rythm: собственный порог 0..1
        // по нормированному flux (onset), не завязан на большой rythm-маппер (см. md_plans/11).
        let (kick_thr, snare_thr) = if cfg.control.enabled {
            (ctl.kick_thresh, ctl.snare_thresh)
        } else {
            (cfg.detectors[0].threshold, cfg.detectors[1].threshold)
        };
        let mut kd = cfg.detectors[0].clone(); kd.threshold = kick_thr;
        let mut sd = cfg.detectors[1].clone(); sd.threshold = snare_thr;
        let rd = cfg.detectors[2].clone(); // rythm — ручной порог (0..1)

        // нормировка flux пик-фолловером → устойчивая шкала 0..1 (onset)
        flux_peak = (flux_peak * 0.999).max(spectrum.flux).max(1e-6);
        let flux_norm = (spectrum.flux / flux_peak).clamp(0.0, 1.0);

        let kick = kick_det.process(brms[0], &kd, dt);       // kick по low RMS
        let snare = snare_det.process(brms[2], &sd, dt);     // snare по high RMS
        let rythm = rythm_det.process(flux_norm, &rd, dt);   // rythm — onset по flux

        // огибающие для ламп (импульсы плохо видны при 60 Гц)
        kick_env = (kick_env * ENV_DECAY).max(kick.1);
        snare_env = (snare_env * ENV_DECAY).max(snare.0.max(snare.1));
        rythm_env = (rythm_env * ENV_DECAY).max(rythm.1);

        let (k4, k8, k16) = kick_counts.process(kick.1);
        let (s4, s8, s16) = snare_counts.process(snare.1);

        // ── R2: опциональная RMS-power ветвь в DSP (через dsp_gain, math1) ──
        let dsp_rms = if cfg.dsp_rmspower {
            rms_power(&frame) * cfg.dsp_gain.gain
        } else {
            0.0
        };

        // ── OSC ──
        let mut channels: Vec<(String, f32)> = vec![
            ("low".into(), levels[0]),
            ("mid".into(), levels[1]),
            ("high".into(), levels[2]),
            ("kick".into(), kick.1),
            ("snare".into(), snare.0),
            ("rythm".into(), rythm.1),
            ("spectralCentroid".into(), centroid),
            ("fmsd".into(), fms),
            ("smsd".into(), sms),
            ("trigger4k".into(), k4), ("trigger8k".into(), k8), ("trigger16k".into(), k16),
            ("trigger4s".into(), s4), ("trigger8s".into(), s8), ("trigger16s".into(), s16),
        ];
        if cfg.dsp_rmspower {
            channels.push(("dsprms".into(), dsp_rms));
        }
        if cfg.osc.enabled {
            if osc.is_none() { osc = OscSender::new(&cfg.osc).ok(); }
            if let Some(sender) = osc.as_ref() {
                let _ = sender.send(&channels, cfg.osc.bundle);
            }
        }

        // ── метрики ──
        {
            let mut m = shared.metrics.lock().unwrap();
            m.device_name = audio.as_ref().map(|a| a.device_name.clone()).unwrap_or_else(|| "—".into());
            m.sample_rate = sample_rate;
            m.input_rms = input_rms;
            m.band_levels = levels;
            m.band_rms = brms;
            m.kick = kick; m.snare = snare; m.rythm = rythm;
            m.kick_env = kick_env; m.snare_env = snare_env; m.rythm_env = rythm_env;
            m.control = ctl;
            m.centroid = centroid; m.fms = fms; m.sms = sms;
            m.flux = spectrum.flux; m.dsp_rms = dsp_rms;
            let n = spectrum.mags.len().min(SPECTRUM_DRAW_BINS);
            m.spectrum = spectrum.mags[..n].to_vec();
            m.osc_channels = channels.len();
        }

        // ── ритм control-rate ──
        let elapsed = t0.elapsed();
        if elapsed < tick {
            std::thread::sleep(tick - elapsed);
        }
    }
}

/// Открыть аудиовход по текущему конфигу; вернуть (вход, потребитель кольца, sample_rate, ошибка).
fn open_audio(
    shared: &Arc<Shared>,
    fallback_sr: f32,
) -> (Option<AudioInput>, Option<ringbuf::HeapCons<f32>>, f32, Option<String>) {
    let cfg = shared.config.lock().unwrap().clone();
    if cfg.input.source == Source::File {
        return (None, None, fallback_sr, Some("файловый вход не собран (feature file-input)".into()));
    }
    let rb = HeapRb::<f32>::new(48000 * 2);
    let (prod, cons) = rb.split();
    match AudioInput::open(cfg.input.device.as_deref(), cfg.input.pulse_source.as_deref(), prod) {
        Ok(inp) => {
            let sr = inp.sample_rate;
            (Some(inp), Some(cons), sr, None)
        }
        Err(e) => (None, None, fallback_sr, Some(format!("аудио: {e}"))),
    }
}

/// Простейший downward-компрессор (аппроксимация `audiodyna`).
fn apply_compressor(frame: &mut [f32], cfg: &crate::config::CompressorCfg) {
    let thr = 10f32.powf(cfg.threshold_db / 20.0);
    let makeup = 10f32.powf(cfg.makeup_db / 20.0);
    let ratio = cfg.ratio.max(1e-3);
    for s in frame.iter_mut() {
        let a = s.abs();
        if a > thr {
            let over = a / thr;
            let comp = over.powf(1.0 / ratio - 1.0); // >1 сжатие
            *s *= comp;
        }
        *s *= makeup;
    }
}

#[inline]
fn normalize(x: f32, lo: f32, hi: f32) -> f32 {
    if (hi - lo).abs() < 1e-9 { return 0.0; }
    ((x - lo) / (hi - lo)).clamp(0.0, 1.0)
}
