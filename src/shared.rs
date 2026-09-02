//! Разделяемое состояние между GUI, движком и OSC.

use crate::config::Config;
use crate::control::ControlOut;
use crate::diag::LogBus;
use crate::osc_map::OscSnapshot;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

/// Захват mutex с восстановлением после паники в другом потоке.
///
/// Отравленный lock не должен ронять аудио/OSC/GUI: данные под ним — метрики и снимки,
/// частично записанное значение не нарушает инварианты и будет перезаписано следующим тиком.
fn lock_or_recover<'a, T>(m: &'a Mutex<T>, name: &str) -> std::sync::MutexGuard<'a, T> {
    match m.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            crate::diag::warn("shared", format!("mutex '{name}' poisoned, recovering"));
            poisoned.into_inner()
        }
    }
}

/// Чтение RwLock с восстановлением после отравления (см. [`lock_or_recover`]).
fn read_or_recover<'a, T>(r: &'a RwLock<T>, name: &str) -> std::sync::RwLockReadGuard<'a, T> {
    match r.read() {
        Ok(g) => g,
        Err(poisoned) => {
            crate::diag::warn("shared", format!("rwlock '{name}' poisoned on read, recovering"));
            poisoned.into_inner()
        }
    }
}

/// Запись в RwLock с восстановлением после отравления (см. [`lock_or_recover`]).
fn write_or_recover<'a, T>(r: &'a RwLock<T>, name: &str) -> std::sync::RwLockWriteGuard<'a, T> {
    match r.write() {
        Ok(g) => g,
        Err(poisoned) => {
            crate::diag::warn("shared", format!("rwlock '{name}' poisoned on write, recovering"));
            poisoned.into_inner()
        }
    }
}

/// Число бинов спектра для GUI (фиксированный массив — без alloc на hot path).
pub const SPECTRUM_DRAW_BINS: usize = 256;

/// Снимок метрик для GUI/мониторинга (движок пишет, GUI читает).
#[derive(Clone)]
pub struct Metrics {
    pub device_name: String,
    pub sample_rate: f32,
    pub input_rms: f32,
    pub band_levels: [f32; 3],
    pub band_rms: [f32; 3],
    pub kick: (f32, f32),
    pub snare: (f32, f32),
    pub rythm: (f32, f32),
    pub kick_env: f32,
    pub snare_env: f32,
    pub rythm_env: f32,
    pub control: ControlOut,
    pub centroid: f32,
    pub fms: f32,
    pub sms: f32,
    pub flux: f32,
    /// Вход детекторов: kick=low RMS*pregain, snare=high RMS*pregain, rythm=flux 0..1.
    pub detect: [f32; 3],
    pub detect_thr: [f32; 3],
    pub dsp_rms: f32,
    pub beat_phase: f32,
    /// Позиция в такте 4 (1..4), 0 если ещё не было ударов.
    pub kick_bar_pos: u32,
    pub snare_bar_pos: u32,
    pub compute_frame_id: u64,
    pub spectrum: [f32; SPECTRUM_DRAW_BINS],
    /// Сколько первых бинов в `spectrum` валидны (≤ SPECTRUM_DRAW_BINS).
    pub spectrum_len: usize,
    pub osc_channels: usize,
    pub error: Option<String>,
    pub osc_last_error: Option<String>,
    pub osc_send_ok: u64,
    pub osc_send_err: u64,
    pub osc_bundle_seq: u64,
    /// Доля заполнения ringbuf 0..1.
    pub ringbuf_fill: f32,
    /// Фактическое время compute-тика, мс.
    pub compute_dt_ms: f32,
    /// Отклонение OSC-тика от дедлайна, мс (EMA).
    pub osc_jitter_ms: f32,
    /// Задержка OSC send относительно compute-кадра (send_mono − t_mono), мс.
    pub osc_send_latency_ms: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            device_name: String::new(),
            sample_rate: 0.0,
            input_rms: 0.0,
            band_levels: [0.0; 3],
            band_rms: [0.0; 3],
            kick: (0.0, 0.0),
            snare: (0.0, 0.0),
            rythm: (0.0, 0.0),
            kick_env: 0.0,
            snare_env: 0.0,
            rythm_env: 0.0,
            control: ControlOut::default(),
            centroid: 0.0,
            fms: 0.0,
            sms: 0.0,
            flux: 0.0,
            detect: [0.0; 3],
            detect_thr: [0.0; 3],
            dsp_rms: 0.0,
            beat_phase: 0.0,
            kick_bar_pos: 0,
            snare_bar_pos: 0,
            compute_frame_id: 0,
            spectrum: [0.0; SPECTRUM_DRAW_BINS],
            spectrum_len: 0,
            osc_channels: 0,
            error: None,
            osc_last_error: None,
            osc_send_ok: 0,
            osc_send_err: 0,
            osc_bundle_seq: 0,
            ringbuf_fill: 0.0,
            compute_dt_ms: 0.0,
            osc_jitter_ms: 0.0,
            osc_send_latency_ms: 0.0,
        }
    }
}

impl Metrics {
    /// Копия в уже выделенный буфер (GUI: без повторного alloc спектра).
    pub fn copy_from(&self, dst: &mut Metrics) {
        dst.device_name.clone_from(&self.device_name);
        dst.sample_rate = self.sample_rate;
        dst.input_rms = self.input_rms;
        dst.band_levels = self.band_levels;
        dst.band_rms = self.band_rms;
        dst.kick = self.kick;
        dst.snare = self.snare;
        dst.rythm = self.rythm;
        dst.kick_env = self.kick_env;
        dst.snare_env = self.snare_env;
        dst.rythm_env = self.rythm_env;
        dst.control = self.control;
        dst.centroid = self.centroid;
        dst.fms = self.fms;
        dst.sms = self.sms;
        dst.flux = self.flux;
        dst.detect = self.detect;
        dst.detect_thr = self.detect_thr;
        dst.dsp_rms = self.dsp_rms;
        dst.beat_phase = self.beat_phase;
        dst.kick_bar_pos = self.kick_bar_pos;
        dst.snare_bar_pos = self.snare_bar_pos;
        dst.compute_frame_id = self.compute_frame_id;
        dst.spectrum = self.spectrum;
        dst.spectrum_len = self.spectrum_len;
        dst.osc_channels = self.osc_channels;
        dst.error.clone_from(&self.error);
        dst.osc_last_error.clone_from(&self.osc_last_error);
        dst.osc_send_ok = self.osc_send_ok;
        dst.osc_send_err = self.osc_send_err;
        dst.osc_bundle_seq = self.osc_bundle_seq;
        dst.ringbuf_fill = self.ringbuf_fill;
        dst.compute_dt_ms = self.compute_dt_ms;
        dst.osc_jitter_ms = self.osc_jitter_ms;
        dst.osc_send_latency_ms = self.osc_send_latency_ms;
    }
}

/// Double-buffer метрик: engine публикует целиком, OSC дописывает счётчики атомарно.
pub struct MetricsDoubleBuffer {
    slots: [Mutex<Metrics>; 2],
    write_idx: AtomicUsize,
    engine_error: Mutex<Option<String>>,
    osc_send_ok: AtomicU64,
    osc_send_err: AtomicU64,
    osc_bundle_seq: AtomicU64,
    osc_jitter_bits: AtomicU32,
    osc_send_latency_bits: AtomicU32,
    osc_last_error: Mutex<Option<String>>,
}

impl MetricsDoubleBuffer {
    pub fn new() -> Self {
        Self {
            slots: [Mutex::new(Metrics::default()), Mutex::new(Metrics::default())],
            write_idx: AtomicUsize::new(0),
            engine_error: Mutex::new(None),
            osc_send_ok: AtomicU64::new(0),
            osc_send_err: AtomicU64::new(0),
            osc_bundle_seq: AtomicU64::new(0),
            osc_jitter_bits: AtomicU32::new(0f32.to_bits()),
            osc_send_latency_bits: AtomicU32::new(0f32.to_bits()),
            osc_last_error: Mutex::new(None),
        }
    }

    pub fn note_engine_error(&self, err: Option<String>) {
        *lock_or_recover(&self.engine_error, "engine_error") = err;
    }

    fn merge_osc_fields(&self, m: &mut Metrics) {
        m.osc_send_ok = self.osc_send_ok.load(Ordering::Relaxed);
        m.osc_send_err = self.osc_send_err.load(Ordering::Relaxed);
        m.osc_bundle_seq = self.osc_bundle_seq.load(Ordering::Relaxed);
        m.osc_jitter_ms = f32::from_bits(self.osc_jitter_bits.load(Ordering::Relaxed));
        m.osc_send_latency_ms =
            f32::from_bits(self.osc_send_latency_bits.load(Ordering::Relaxed));
        m.osc_last_error = lock_or_recover(&self.osc_last_error, "osc_last_error").clone();
        m.error = lock_or_recover(&self.engine_error, "engine_error").clone();
    }

    pub fn publish(&self, mut m: Metrics) {
        self.merge_osc_fields(&mut m);
        let idx = self.write_idx.load(Ordering::Relaxed);
        *lock_or_recover(&self.slots[idx], "metrics_slot") = m;
        self.write_idx.store(1 - idx, Ordering::Release);
    }

    pub fn copy_latest(&self, dst: &mut Metrics) {
        let idx = 1 - self.write_idx.load(Ordering::Acquire);
        let src = lock_or_recover(&self.slots[idx], "metrics_slot");
        src.copy_from(dst);
        self.merge_osc_fields(dst);
    }

    pub fn record_osc_ok(&self, seq: u64) {
        self.osc_send_ok.fetch_add(1, Ordering::Relaxed);
        self.osc_bundle_seq.store(seq, Ordering::Relaxed);
    }

    pub fn record_osc_err(&self, msg: String) {
        *lock_or_recover(&self.osc_last_error, "osc_last_error") = Some(msg);
        self.osc_send_err.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_osc_jitter(&self, jitter_ms: f32) {
        self.osc_jitter_bits
            .store(jitter_ms.to_bits(), Ordering::Relaxed);
    }

    pub fn set_osc_send_latency(&self, latency_ms: f32) {
        self.osc_send_latency_bits
            .store(latency_ms.to_bits(), Ordering::Relaxed);
    }
}

/// Конфиг: короткий lock — только clone/swap `Arc<Config>`.
pub struct ConfigHandle {
    inner: RwLock<Arc<Config>>,
    version: AtomicU64,
}

impl ConfigHandle {
    pub fn new(config: Config) -> Self {
        Self {
            inner: RwLock::new(Arc::new(config)),
            version: AtomicU64::new(0),
        }
    }

    /// Для engine/osc: дешёвое чтение без клонирования всего Config.
    pub fn load(&self) -> Arc<Config> {
        Arc::clone(&read_or_recover(&self.inner, "config_inner"))
    }

    pub fn store(&self, config: Config) {
        *write_or_recover(&self.inner, "config_inner") = Arc::new(config);
        self.version.fetch_add(1, Ordering::Release);
    }

    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }
}

/// Double-buffer OSC-снимков: engine пишет, OSC читает без clone всего Vec.
pub struct OscDoubleBuffer {
    slots: [Mutex<Arc<OscSnapshot>>; 2],
    write_idx: AtomicUsize,
}

impl OscDoubleBuffer {
    pub fn new() -> Self {
        let empty = Arc::new(OscSnapshot::empty());
        Self {
            slots: [Mutex::new(Arc::clone(&empty)), Mutex::new(empty)],
            write_idx: AtomicUsize::new(0),
        }
    }

    pub fn publish(&self, snapshot: OscSnapshot) {
        let idx = self.write_idx.load(Ordering::Relaxed);
        *lock_or_recover(&self.slots[idx], "osc_slot") = Arc::new(snapshot);
        self.write_idx.store(1 - idx, Ordering::Release);
    }

    pub fn latest(&self) -> Arc<OscSnapshot> {
        let idx = 1 - self.write_idx.load(Ordering::Acquire);
        Arc::clone(&lock_or_recover(&self.slots[idx], "osc_slot"))
    }
}

/// Общая временная шкала: compute и OSC якорят sleep к одному origin.
pub struct Timeline {
    pub origin: Instant,
    compute_tick: AtomicU64,
    osc_tick: AtomicU64,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
            compute_tick: AtomicU64::new(0),
            osc_tick: AtomicU64::new(0),
        }
    }

    pub fn mono_secs(&self) -> f64 {
        self.origin.elapsed().as_secs_f64()
    }

    pub fn next_compute_deadline(&self, rate_hz: f32) -> Instant {
        let n = self.compute_tick.fetch_add(1, Ordering::Relaxed) + 1;
        self.origin + std::time::Duration::from_secs_f64(n as f64 / rate_hz as f64)
    }

    pub fn next_osc_deadline(&self, rate_hz: f32) -> Instant {
        let n = self.osc_tick.fetch_add(1, Ordering::Relaxed) + 1;
        self.origin + std::time::Duration::from_secs_f64(n as f64 / rate_hz as f64)
    }

    pub fn reset(&self) {
        self.compute_tick.store(0, Ordering::Relaxed);
        self.osc_tick.store(0, Ordering::Relaxed);
    }
}

/// Очередь импульсов триггеров (engine → OSC), чтобы не терять при разной частоте.
pub struct TriggerQueue {
    pending: Mutex<Vec<crate::osc_map::TriggerPulse>>,
}

impl TriggerQueue {
    pub fn new() -> Self {
        Self { pending: Mutex::new(Vec::new()) }
    }

    pub fn push(&self, pulses: Vec<crate::osc_map::TriggerPulse>) {
        if pulses.is_empty() {
            return;
        }
        lock_or_recover(&self.pending, "trigger_pending").extend(pulses);
    }

    pub fn drain(&self) -> Vec<crate::osc_map::TriggerPulse> {
        std::mem::take(&mut *lock_or_recover(&self.pending, "trigger_pending"))
    }
}

pub struct Shared {
    pub config: ConfigHandle,
    pub metrics: MetricsDoubleBuffer,
    pub osc_out: OscDoubleBuffer,
    pub timeline: Timeline,
    pub trigger_queue: TriggerQueue,
    pub logs: Arc<LogBus>,
    pub running: AtomicBool,
    pub restart_audio: AtomicBool,
}

impl Shared {
    pub fn new(config: Config, logs: Arc<LogBus>) -> Arc<Self> {
        Arc::new(Self {
            config: ConfigHandle::new(config),
            metrics: MetricsDoubleBuffer::new(),
            osc_out: OscDoubleBuffer::new(),
            timeline: Timeline::new(),
            trigger_queue: TriggerQueue::new(),
            logs,
            running: AtomicBool::new(true),
            restart_audio: AtomicBool::new(false),
        })
    }
}
