//! Beat detection and beat counters (md_plans/02, 05).

use crate::config::DetectorCfg;

/// Детектор удара: порог → гейт → триггер (с retrigger-блокировкой).
#[derive(Clone, Default)]
pub struct BeatDetector {
    prev_gate: bool,
    cooldown_s: f32,
    pub gate: f32,
    pub trigger: f32,
}
impl BeatDetector {
    /// `value` — уровень полосы/спектра; возвращает (gate, trigger) как 0/1.
    pub fn process(&mut self, value: f32, cfg: &DetectorCfg, dt: f32) -> (f32, f32) {
        if !cfg.active {
            self.gate = 0.0; self.trigger = 0.0; self.prev_gate = false;
            return (0.0, 0.0);
        }
        if self.cooldown_s > 0.0 { self.cooldown_s -= dt; }
        let gate = value > cfg.threshold;
        let rising = gate && !self.prev_gate;
        let mut trig = 0.0;
        if rising && self.cooldown_s <= 0.0 {
            trig = 1.0;
            self.cooldown_s = cfg.retrigger_s;
        }
        self.prev_gate = gate;
        self.gate = if gate { 1.0 } else { 0.0 };
        self.trigger = trig;
        (self.gate, trig)
    }
}

/// Счётчик доли по модулю (4/8/16) — md_plans/05 Count_Analysis.
#[derive(Clone)]
pub struct BeatCounter {
    modulo: u32,
    count: u32,
    pub trigger: f32,
}
impl BeatCounter {
    pub fn new(modulo: u32) -> Self { Self { modulo, count: 0, trigger: 0.0 } }
    /// На каждый входной trigger инкремент; импульс на начале цикла (count == 1, express `if $V==1`).
    pub fn process(&mut self, in_trigger: f32) -> f32 {
        self.trigger = 0.0;
        if in_trigger > 0.5 {
            self.count = (self.count % self.modulo) + 1; // 1..=modulo
            if self.count == 1 { self.trigger = 1.0; }
        }
        self.trigger
    }
}

/// Набор счётчиков 4/8/16 для одного источника (kick или snare).
#[derive(Clone)]
pub struct CounterBank {
    pub c4: BeatCounter,
    pub c8: BeatCounter,
    pub c16: BeatCounter,
}
impl Default for CounterBank {
    fn default() -> Self {
        Self { c4: BeatCounter::new(4), c8: BeatCounter::new(8), c16: BeatCounter::new(16) }
    }
}
impl CounterBank {
    /// Возвращает (t4, t8, t16).
    pub fn process(&mut self, trig: f32) -> (f32, f32, f32) {
        (self.c4.process(trig), self.c8.process(trig), self.c16.process(trig))
    }
}
