//! Базовые DSP-блоки. Формулы соответствуют md_plans/01–03.

use crate::config::{BandCfg, FilterKind, LagCfg};
use realfft::{RealFftPlanner, RealToComplex};
use std::collections::VecDeque;
use std::sync::Arc;

/// RBJ-биквад (Direct Form I).
#[derive(Clone, Default)]
pub struct Biquad {
    b0: f32, b1: f32, b2: f32, a1: f32, a2: f32,    // coefficients
    x1: f32, x2: f32, y1: f32, y2: f32,             // state
}
impl Biquad {
    pub fn design(kind: FilterKind, sr: f32, cutoff: f32, q: f32) -> Self {
        let w0 = 2.0 * std::f32::consts::PI * (cutoff / sr).clamp(1e-5, 0.49);
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q.max(1e-3));
        let (b0, b1, b2, a0, a1, a2) = match kind {
            FilterKind::LowPass => {
                let b1 = 1.0 - cos;
                (b1 / 2.0, b1, b1 / 2.0, 1.0 + alpha, -2.0 * cos, 1.0 - alpha)
            }
            FilterKind::HighPass => {
                let b1 = -(1.0 + cos);
                ((1.0 + cos) / 2.0, b1, (1.0 + cos) / 2.0, 1.0 + alpha, -2.0 * cos, 1.0 - alpha)
            }
            FilterKind::BandPass => {
                // constant 0 dB peak gain
                (alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cos, 1.0 - alpha)
            }
        };
        Self {
            b0: b0 / a0, b1: b1 / a0, b2: b2 / a0, a1: a1 / a0, a2: a2 / a0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
        }
    }
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2 - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1; self.x1 = x;
        self.y2 = self.y1; self.y1 = y;
        y
    }
}

/// Каскад биквадов (число секций ≈ rolloff/6, md_plans/09 C14).
#[derive(Clone, Default)]
pub struct BiquadCascade { stages: Vec<Biquad> }
impl BiquadCascade {
    pub fn design(kind: FilterKind, sr: f32, cutoff: f32, q: f32, rolloff_db_oct: f32) -> Self {
        let n = ((rolloff_db_oct / 6.0).round() as usize).clamp(1, 8);
        Self { stages: vec![Biquad::design(kind, sr, cutoff, q); n] }
    }
    #[inline]
    pub fn process(&mut self, mut x: f32) -> f32 {
        for s in &mut self.stages { x = s.process(x); }
        x
    }
}

/// RMS-power по блоку сэмплов (TD `analyze rmspower`).
pub fn rms_power(block: &[f32]) -> f32 {
    if block.is_empty() { return 0.0; }
    let sum: f32 = block.iter().map(|v| v * v).sum();
    (sum / block.len() as f32).sqrt()
}

/// Скользящее среднее окном `width` секунд — как TD Filter CHOP (boxcar), а не one-pole.
/// Даёт корректную форму атаки/спада огибающих.
#[derive(Clone, Default)]
pub struct SmoothFilter {
    buf: VecDeque<f32>,
    sum: f32,
}
impl SmoothFilter {
    #[inline]
    pub fn process(&mut self, x: f32, width_s: f32, dt: f32) -> f32 {
        let target_len = ((width_s / dt).round() as usize).max(1);
        self.buf.push_back(x);
        self.sum += x;
        while self.buf.len() > target_len {
            if let Some(old) = self.buf.pop_front() {
                self.sum -= old;
            }
        }
        self.sum / self.buf.len() as f32
    }
}

/// TD Lag CHOP: асимметричные времена нарастания/спада + ограничение ускорения.
/// Stateful (md_plans/10 R3): состояние НЕ сбрасывается при смене параметров.
#[derive(Clone, Default)]
pub struct Lag { y: f32, v: f32 }
impl Lag {
    #[inline]
    pub fn process(&mut self, target: f32, cfg: &LagCfg, dt: f32) -> f32 {
        let rising = target > self.y;
        let lag = if rising { cfg.lag_up } else { cfg.lag_dn }.max(1e-4);
        let accel = if rising { cfg.accel_up } else { cfg.accel_dn }.max(1e-4);
        // целевая скорость сближения (обратно пропорц. времени лага)
        let desired_v = (target - self.y) / lag;
        // ограничение ускорения: не более accel единиц/с за тик (сглаживание рывков)
        let max_dv = accel * dt;
        let dv = (desired_v - self.v).clamp(-max_dv, max_dv);
        self.v += dv;
        self.y += self.v * dt;
        self.y
    }
}

/// Обработчик одной полосы: каскад-фильтр + пред-гейн + порог/гейн + clamp + add + сглаживание.
#[derive(Clone)]
pub struct BandProcessor {
    pub filter: BiquadCascade,
    pub smooth: SmoothFilter,
    pub last_rms: f32,
    pub last_out: f32,
}
impl BandProcessor {
    pub fn new(cfg: &BandCfg, sr: f32) -> Self {
        Self {
            filter: BiquadCascade::design(cfg.kind, sr, cfg.cutoff_hz, cfg.resonance, cfg.rolloff_db_oct),
            smooth: SmoothFilter::default(),
            last_rms: 0.0,
            last_out: 0.0,
        }
    }
    pub fn redesign(&mut self, cfg: &BandCfg, sr: f32) {
        self.filter = BiquadCascade::design(cfg.kind, sr, cfg.cutoff_hz, cfg.resonance, cfg.rolloff_db_oct);
    }
    /// Прогнать блок аудио, вернуть (rms_полосы, уровень_после_сглаживания).
    pub fn process_block(&mut self, mono: &[f32], cfg: &BandCfg, dt: f32) -> (f32, f32) {
        if !cfg.active {
            self.last_rms = 0.0;
            self.last_out = self.smooth.process(0.0, cfg.smooth_s, dt);
            return (0.0, self.last_out);
        }
        // фильтруем блок и берём RMS
        let mut acc = 0.0f32;
        for &x in mono {
            let f = self.filter.process(x);
            acc += f * f;
        }
        let rms = if mono.is_empty() { 0.0 } else { (acc / mono.len() as f32).sqrt() };
        let pre = rms * cfg.pregain;                                   // math12/math16
        let lvl = ((pre - cfg.threshold) * cfg.gain).clamp(0.0, 100.0); // math3/limit
        let out = self.smooth.process(lvl + cfg.add, cfg.smooth_s, dt); // add + filter
        self.last_rms = pre;
        self.last_out = out;
        (pre, out)
    }
}

/// FFT-спектр + центроид (md_plans/03).
pub struct Spectrum {
    fft: Arc<dyn RealToComplex<f32>>,
    size: usize,
    ring: Vec<f32>,
    window: Vec<f32>,
    scratch_in: Vec<f32>,
    scratch_out: Vec<realfft::num_complex::Complex<f32>>,
    prev_mags: Vec<f32>,
    pub mags: Vec<f32>,
    pub centroid_bins: f32,
    pub energy: f32,
    /// Спектральный flux (Σ положительных приращений магнитуд) — индикатор ритмических событий.
    pub flux: f32,
}
impl Spectrum {
    pub fn new(size: usize) -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(size);
        let scratch_out = fft.make_output_vec();
        // окно Ханна
        let window = (0..size)
            .map(|i| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / size as f32).cos())
            .collect();
        Self {
            fft,
            size,
            ring: vec![0.0; size],
            window,
            scratch_in: vec![0.0; size],
            mags: vec![0.0; size / 2 + 1],
            prev_mags: vec![0.0; size / 2 + 1],
            scratch_out,
            centroid_bins: 0.0,
            energy: 0.0,
            flux: 0.0,
        }
    }
    /// Добавить блок в кольцо и пересчитать спектр.
    pub fn push_block(&mut self, block: &[f32]) {
        if block.len() >= self.size {
            self.ring.copy_from_slice(&block[block.len() - self.size..]);
        } else {
            self.ring.rotate_left(block.len());
            let start = self.size - block.len();
            self.ring[start..].copy_from_slice(block);
        }
        for i in 0..self.size {
            self.scratch_in[i] = self.ring[i] * self.window[i];
        }
        if self.fft.process(&mut self.scratch_in, &mut self.scratch_out).is_err() {
            return;
        }
        let mut num = 0.0f32;
        let mut den = 0.0f32;
        let mut flux = 0.0f32;
        for (i, c) in self.scratch_out.iter().enumerate() {
            let m = c.norm();
            let d = m - self.prev_mags[i];
            if d > 0.0 { flux += d; }
            self.prev_mags[i] = m;
            self.mags[i] = m;
            num += i as f32 * m;
            den += m;
        }
        // нормировка flux на число бинов → стабильная ~0..несколько шкала
        self.flux = flux / (self.mags.len() as f32).max(1.0);
        self.energy = den;
        self.centroid_bins = if den > 1e-9 { num / den } else { 0.0 };
    }
}
