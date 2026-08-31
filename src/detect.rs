//! Beat detection and beat counters (md_plans/02, 05).

use crate::config::DetectorCfg;

/// Детектор удара: порог → гейт → триггер (retrigger + опц. hold + опц. hysteresis).
#[derive(Clone, Default)]
pub struct BeatDetector {
    prev_gate: bool,
    cooldown_s: f32,
    /// Удержание выхода триггера после импульса.
    trigger_held: f32,
    silence_since_trigger: f32,
    pub gate: f32,
    pub trigger: f32,
}
impl BeatDetector {
    /// `value` — уровень полосы/спектра; возвращает (gate, trigger) как 0/1.
    pub fn process(&mut self, value: f32, cfg: &DetectorCfg, dt: f32) -> (f32, f32) {
        if !cfg.active {
            self.gate = 0.0;
            self.trigger = 0.0;
            self.prev_gate = false;
            self.trigger_held = 0.0;
            self.silence_since_trigger = 0.0;
            return (0.0, 0.0);
        }
        if self.cooldown_s > 0.0 {
            self.cooldown_s -= dt;
        }

        let on_thr = cfg.threshold;
        let off_thr = if cfg.hysteresis_enabled && cfg.hysteresis > 0.0 {
            (cfg.threshold - cfg.hysteresis).max(0.0)
        } else {
            cfg.threshold
        };
        let gate = if self.prev_gate { value > off_thr } else { value > on_thr };
        let rising = gate && !self.prev_gate;

        let mut impulse = 0.0;
        if rising && self.cooldown_s <= 0.0 {
            impulse = 1.0;
            self.cooldown_s = cfg.retrigger_s;
        }

        let trigger_out = if cfg.trigger_hold_enabled && cfg.trigger_hold_s > 0.0 {
            if impulse > 0.5 {
                self.trigger_held = 1.0;
                self.silence_since_trigger = 0.0;
            } else {
                self.silence_since_trigger += dt;
                if self.silence_since_trigger >= cfg.trigger_hold_s {
                    self.trigger_held = 0.0;
                }
            }
            self.trigger_held
        } else {
            impulse
        };

        self.prev_gate = gate;
        self.gate = if gate { 1.0 } else { 0.0 };
        self.trigger = trigger_out;
        (self.gate, trigger_out)
    }
}

/// Счётчик доли по модулю (4/8/16) — md_plans/05 Count_Analysis.
#[derive(Clone)]
pub struct BeatCounter {
    modulo: u32,
    count: u32,
    prev_high: bool,
    pub trigger: f32,
}
impl BeatCounter {
    pub fn new(modulo: u32) -> Self {
        Self { modulo, count: 0, prev_high: false, trigger: 0.0 }
    }

    /// Текущая позиция в цикле 1..=modulo (0 если ещё не было триггеров).
    pub fn count(&self) -> u32 {
        self.count
    }

    #[allow(dead_code)]
    pub fn modulo(&self) -> u32 {
        self.modulo
    }

    /// Фаза 0..1 внутри цикла счётчика.
    pub fn phase(&self) -> f32 {
        if self.count == 0 { 0.0 } else { (self.count - 1) as f32 / self.modulo as f32 }
    }
    /// Инкремент только на фронте 0→1 (hold не крутит счётчик каждый кадр).
    /// Импульс на начале цикла (count == 1).
    pub fn process(&mut self, in_trigger: f32) -> f32 {
        self.trigger = 0.0;
        let high = in_trigger > 0.5;
        let rising = high && !self.prev_high;
        self.prev_high = high;
        if rising {
            self.count = (self.count % self.modulo) + 1; // 1..=modulo
            if self.count == 1 {
                self.trigger = 1.0;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DetectorCfg;

    fn det(thr: f32) -> DetectorCfg {
        DetectorCfg {
            name: "t".into(),
            threshold: thr,
            retrigger_s: 0.0,
            active: true,
            hysteresis_enabled: false,
            hysteresis: 0.0,
            trigger_hold_enabled: false,
            trigger_hold_s: 0.05,
        }
    }

    #[test]
    fn trigger_only_on_rise() {
        let mut d = BeatDetector::default();
        let cfg = det(0.5);
        let dt = 1.0 / 60.0;
        let a = d.process(0.1, &cfg, dt);
        assert_eq!(a.1, 0.0);
        let b = d.process(0.9, &cfg, dt);
        assert_eq!(b.1, 1.0);
        let c = d.process(0.9, &cfg, dt);
        assert_eq!(c.1, 0.0);
    }

    #[test]
    fn counter_hold_does_not_tick() {
        let mut c = BeatCounter::new(4);
        assert_eq!(c.process(1.0), 1.0);
        assert_eq!(c.process(1.0), 0.0);
        assert_eq!(c.process(0.0), 0.0);
        assert_eq!(c.process(1.0), 0.0);
        assert_eq!(c.count(), 2);
    }
}
