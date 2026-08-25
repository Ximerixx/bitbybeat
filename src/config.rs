//! Конфигурация тракта. Все дефолты — из `md_plans/07_params_defaults.md` (эталон дампа).
//!
//! Главная стезя (md_plans/10 R0): каждая ступень настраивается вживую. Ступени, которые
//! можно включать/выключать, обёрнуты в [`Toggle`]. Гейны (`GainCfg`) всегда активны — только
//! крутилки. Lag ([`LagCfg`]) — без bypass и stateful (состояние живёт в DSP, не тут).

use serde::{Deserialize, Serialize};

fn default_compute_rate() -> f32 { 120.0 }
fn default_osc_rate() -> f32 { 60.0 }

/// toggle + parameters (md_plans/10 R1/R3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Toggle<T> {
    pub enabled: bool,
    pub cfg: T,
}
impl<T> Toggle<T> {
    #[allow(dead_code)]
    pub fn on(cfg: T) -> Self { Self { enabled: true, cfg } }
    pub fn off(cfg: T) -> Self { Self { enabled: false, cfg } }
}

// input config

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    /// Audio device (including monitor output device) — priority (R6).
    Device,
    /// File — optional, for feature `file-input`.
    File,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputCfg {
    pub source: Source,
    /// Name of selected cpal/ALSA device (None → default).
    pub device: Option<String>,
    /// PulseAudio source name (incl. `.monitor`). If set — takes priority over `device`
    /// and is opened via ALSA `pulse` + `PULSE_SOURCE` (see audio.rs).
    pub pulse_source: Option<String>,
    /// Show/prefer monitor sources (loopback system output).
    pub prefer_monitor: bool,
    /// Индексы каналов устройства для анализа (0-based). Пусто = даунмикс всех в моно.
    /// Для многоканальных интерфейсов (напр. микшер 18 in) — выбрать нужные 1–2 канала.
    #[serde(default)]
    pub channels_pick: Vec<usize>,
    pub file_path: Option<String>,
}
impl Default for InputCfg {
    fn default() -> Self {
        Self { source: Source::Device, device: None, pulse_source: None, prefer_monitor: true, channels_pick: Vec::new(), file_path: None }
    }
}

// pre-processing config

/// Compressor `audiodyna` (dump: thr −20.6, ratio 0.638, gain +6.9 dB). Default — bypass (R1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompressorCfg {
    pub threshold_db: f32,
    pub ratio: f32,
    pub makeup_db: f32,
}
impl Default for CompressorCfg {
    fn default() -> Self { Self { threshold_db: -20.6, ratio: 0.638, makeup_db: 6.9 } }
}

// gain config

/// Аффинный множитель TD Math CHOP: `y = postoff + gain*(preoff + x)`, затем опц. remap 0..1 → torange.
/// Всегда активен (R5) — только крутилки.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GainCfg {
    pub preoff: f32,
    pub gain: f32,
    pub postoff: f32,
    /// remap `fromrange(0..1)` → `torange`.
    pub torange: Option<(f32, f32)>,
}
impl GainCfg {
    pub const fn new(preoff: f32, gain: f32, postoff: f32) -> Self {
        Self { preoff, gain, postoff, torange: None }
    }
    pub const fn with_range(preoff: f32, gain: f32, postoff: f32, lo: f32, hi: f32) -> Self {
        Self { preoff, gain, postoff, torange: Some((lo, hi)) }
    }
    /// Применить (TD-порядок: multiply-add, затем range remap).
    pub fn apply(&self, x: f32) -> f32 {
        let y = self.postoff + self.gain * (self.preoff + x);
        match self.torange {
            Some((lo, hi)) => lo + (hi - lo) * y, // fromrange 0..1
            None => y,
        }
    }
}

// sigmoid config

/// Логистическая функция `ceil / (1 + exp(-k*(x - center)))` (R4/R5).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SigmoidCfg {
    pub enabled: bool,
    pub ceil: f32,
    /// Общая крутизна (используется, когда `asymmetric = false`).
    pub k: f32,
    pub center: f32,
    /// Асимметрия: раздельная крутизна левой/правой половины относительно `center`.
    #[serde(default)]
    pub asymmetric: bool,
    /// Крутизна левой половины (x < center). При загрузке старых пресетов = `k`.
    #[serde(default = "default_side_k")]
    pub k_left: f32,
    /// Крутизна правой половины (x > center).
    #[serde(default = "default_side_k")]
    pub k_right: f32,
}
impl SigmoidCfg {
    pub fn eval(&self, x: f32) -> f32 {
        if !self.enabled {
            return x; // R4 OFF → линейный выход маппера
        }
        // Кусочно-логистическая, непрерывная в center (обе половины дают ceil/2 при x=center).
        let k = if !self.asymmetric {
            self.k
        } else if x < self.center {
            self.k_left
        } else {
            self.k_right
        };
        self.ceil / (1.0 + (-k * (x - self.center)).exp())
    }

    /// Полный конструктор (симметричный по умолчанию, половины = k).
    pub fn new(enabled: bool, ceil: f32, k: f32, center: f32) -> Self {
        Self { enabled, ceil, k, center, asymmetric: false, k_left: k, k_right: k }
    }
}

fn default_side_k() -> f32 { 3.0 }

// lag config (stateful, no bypass)

/// TD Lag CHOP. Времена нарастания/спада + ограничение ускорения. Bypass запрещён (R3).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LagCfg {
    pub lag_up: f32,
    pub lag_dn: f32,
    pub accel_up: f32,
    pub accel_dn: f32,
}
impl Default for LagCfg {
    fn default() -> Self { Self { lag_up: 2.0, lag_dn: 4.0, accel_up: 1.0, accel_dn: 3.0 } }
}

// band config

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterKind { LowPass, BandPass, HighPass }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BandCfg {
    pub name: String,
    pub kind: FilterKind,
    pub cutoff_hz: f32,
    pub rolloff_db_oct: f32,
    pub resonance: f32,
    /// Пред-гейн (low=1, mid=2, high=4 в дампе — math12/math16).
    pub pregain: f32,
    /// Статический порог полосы (`Lowthresh`/…).
    pub threshold: f32,
    /// Гейн полосы — приходит из адаптива, но можно и вручную (крутилка).
    pub gain: f32,
    pub add: f32,
    /// Окно сглаживания (сек) — TD Filter width.
    pub smooth_s: f32,
    pub active: bool,
}

// ─────────────────────────── Детекторы ───────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DetectorCfg {
    pub name: String,
    /// Порог (может подменяться адаптивом).
    pub threshold: f32,
    /// Минимальный интервал между импульсами триггера, сек.
    pub retrigger_s: f32,
    pub active: bool,
    /// Вкл. гистерезис gate: гаснет ниже `threshold - hysteresis`.
    #[serde(default)]
    pub hysteresis_enabled: bool,
    #[serde(default)]
    pub hysteresis: f32,
    /// Удерживать триггер = 1, пока не пройдёт `trigger_hold_s` без новых импульсов.
    #[serde(default)]
    pub trigger_hold_enabled: bool,
    #[serde(default = "default_trigger_hold")]
    pub trigger_hold_s: f32,
}

fn default_trigger_hold() -> f32 { 0.05 }

// ─────────────────────────── Адаптивное управление ───────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlCfg {
    pub enabled: bool,
    /// Включить RMS в control-ветви (R3 — можно выключить, кроме гейна/lag).
    pub control_rms: bool,
    /// `math2` — гейн коррекции.
    pub corr_gain: GainCfg,
    pub lag: LagCfg,
    /// Мапперы → значения DSP.
    pub low_gain: GainCfg,
    pub mid_gain: GainCfg,
    pub high_gain: GainCfg,      // highControlGain1
    pub high_gain_alt: GainCfg,  // highControlGain (альтернативный, R1)
    pub use_high_alt: bool,
    pub kick_map: GainCfg,
    pub snare_map: GainCfg,
    pub rythm_map: GainCfg,
    pub kick_sigmoid: SigmoidCfg,
    pub snare_sigmoid: SigmoidCfg,
    pub rythm_sigmoid: SigmoidCfg,
}
impl Default for ControlCfg {
    fn default() -> Self {
        Self {
            enabled: true,
            control_rms: true,
            corr_gain: GainCfg::new(0.1, 0.64, 0.0),
            lag: LagCfg::default(),
            low_gain: GainCfg::new(0.2, 4.57, -0.2),
            mid_gain: GainCfg::new(0.5, 4.05, -0.6),
            high_gain: GainCfg::new(0.0, 1.93, 0.0),
            high_gain_alt: GainCfg::new(0.0, 0.77, 0.3),
            use_high_alt: false,
            kick_map: GainCfg::with_range(0.0, 3.92, -0.3, 0.0, 0.5),
            snare_map: GainCfg::with_range(0.5, 6.78, -0.2, 0.0, 0.09),
            rythm_map: GainCfg::with_range(1.8, 4.76, -0.8, 0.0, 6.0),
            kick_sigmoid: SigmoidCfg::new(true, 0.7, 5.4, 0.3),
            snare_sigmoid: SigmoidCfg::new(true, 0.9, 2.1, 0.5),
            rythm_sigmoid: SigmoidCfg::new(false, 1.0, 3.0, 0.5),
        }
    }
}

// osc config

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OscTransport {
    #[default]
    Udp,
    Tcp,
}

/// Привязка триггеров и OSC-тиков к фазе такта (снижает джиттер относительно бита).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OscPhaseCfg {
    /// Квантовать импульсы к сетке фазы (0..1).
    #[serde(default = "default_true")]
    pub quantize_triggers: bool,
    /// Шаг сетки фазы (0.25 = четверти такта в 4/4).
    #[serde(default = "default_phase_grid")]
    pub phase_grid: f32,
    /// Якорить OSC sleep к общей временной шкале (вместо накопления дрейфа).
    #[serde(default = "default_true")]
    pub sync_timeline: bool,
    /// Слать импульсы триггеров сразу при детекте (минуя очередь фазы).
    #[serde(default)]
    pub immediate_triggers: bool,
}

fn default_true() -> bool { true }
fn default_phase_grid() -> f32 { 0.25 }

impl Default for OscPhaseCfg {
    fn default() -> Self {
        Self {
            quantize_triggers: true,
            phase_grid: default_phase_grid(),
            sync_timeline: true,
            immediate_triggers: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OscCfg {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    /// Слать одним OSC-bundle (рекомендуется; отдельные сообщения — legacy).
    pub bundle: bool,
    #[serde(default)]
    pub transport: OscTransport,
    #[serde(default)]
    pub phase: OscPhaseCfg,
    /// Включать `/bundleSeq` и `/bundleTime` в каждый bundle.
    #[serde(default = "default_true")]
    pub bundle_meta: bool,
    /// На OSC: low/mid/high не ниже 0 (внутренняя математика без изменений).
    #[serde(default)]
    pub clip_levels_at_zero: bool,
}
impl Default for OscCfg {
    fn default() -> Self {
        Self {
            enabled: true,
            host: "127.0.0.1".into(),
            port: 7700,
            bundle: true,
            transport: OscTransport::Udp,
            phase: OscPhaseCfg::default(),
            bundle_meta: true,
            clip_levels_at_zero: false,
        }
    }
}

// core config

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub input: InputCfg,
    /// Пре-компрессор (R1, дефолт bypass).
    pub compressor: Toggle<CompressorCfg>,
    /// RMS-power во входную DSP-ветвь (R2).
    pub dsp_rmspower: bool,
    /// `math1` — DSP-гейн входа.
    pub dsp_gain: GainCfg,
    pub bands: Vec<BandCfg>,
    pub detectors: Vec<DetectorCfg>,
    pub control: ControlCfg,
    pub osc: OscCfg,
    /// Частота обсчёта DSP/детекторов, Гц (компьют-луп).
    #[serde(default = "default_compute_rate")]
    pub compute_rate_hz: f32,
    /// Частота отправки OSC, Гц (отдельный таймер, читает последний снимок).
    #[serde(default = "default_osc_rate")]
    pub osc_rate_hz: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input: InputCfg::default(),
            compressor: Toggle::off(CompressorCfg::default()),
            dsp_rmspower: false,
            dsp_gain: GainCfg::new(0.1, 2.28, 0.0),
            bands: vec![
                BandCfg {
                    name: "low".into(), kind: FilterKind::LowPass, cutoff_hz: 150.0,
                    rolloff_db_oct: 20.0, resonance: 0.707, pregain: 1.0,
                    threshold: 0.116, gain: 1.84, add: -0.492, smooth_s: 0.276, active: true,
                },
                BandCfg {
                    name: "mid".into(), kind: FilterKind::BandPass, cutoff_hz: 800.0,
                    rolloff_db_oct: 20.0, resonance: 0.707, pregain: 2.0,
                    threshold: 0.052, gain: 1.98, add: -0.363, smooth_s: 0.224, active: true,
                },
                BandCfg {
                    name: "high".into(), kind: FilterKind::HighPass, cutoff_hz: 3500.0,
                    rolloff_db_oct: 15.0, resonance: 0.8, pregain: 4.0,
                    threshold: 0.116, gain: 1.45, add: -0.32, smooth_s: 0.093, active: true,
                },
            ],
            detectors: vec![
                DetectorCfg { name: "kick".into(),  threshold: 0.328, retrigger_s: 0.08, active: true, hysteresis_enabled: false, hysteresis: 0.02, trigger_hold_enabled: false, trigger_hold_s: 0.05 },
                DetectorCfg { name: "snare".into(), threshold: 0.338, retrigger_s: 0.0,  active: true, hysteresis_enabled: false, hysteresis: 0.02, trigger_hold_enabled: false, trigger_hold_s: 0.05 },
                DetectorCfg { name: "rythm".into(), threshold: 0.45,  retrigger_s: 0.12, active: true, hysteresis_enabled: false, hysteresis: 0.02, trigger_hold_enabled: false, trigger_hold_s: 0.08 },
            ],
            control: ControlCfg::default(),
            osc: OscCfg::default(),
            compute_rate_hz: default_compute_rate(),
            osc_rate_hz: default_osc_rate(),
        }
    }
}

impl Config {
    pub fn load_ron(path: &str) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        Ok(ron::from_str(&s)?)
    }
    pub fn save_ron(&self, path: &str) -> anyhow::Result<()> {
        let s = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        std::fs::write(path, s)?;
        Ok(())
    }
}
