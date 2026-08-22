//! Адаптивное управление (md_plans/04): RMS → math2 → lag(stateful) → мапперы + сигмоиды.
//! Выход правит гейны полос и пороги детекторов.

use crate::config::ControlCfg;
use crate::dsp::Lag;

/// Значения, которыми адаптив правит DSP.
#[derive(Clone, Copy, Debug, Default)]
pub struct ControlOut {
    pub low_gain: f32,
    pub mid_gain: f32,
    pub high_gain: f32,
    pub kick_thresh: f32,
    pub snare_thresh: f32,
    pub rythm_thresh: f32,
    pub lag_value: f32,
    /// Выходы мапперов ДО сигмоиды (вход сигмоиды) — для графиков в GUI.
    pub kick_x: f32,
    pub snare_x: f32,
    pub rythm_x: f32,
}

/// Состояние контроллера (держит stateful lag).
#[derive(Default)]
pub struct Controller {
    pub lag: Lag,
}

impl Controller {
    /// `input_rms` — общий RMS входа. `dt` — шаг control-rate.
    pub fn step(&mut self, input_rms: f32, cfg: &ControlCfg, dt: f32) -> ControlOut {
        // R3: RMS в control-ветви можно выключить (но гейн и lag — всегда).
        let level = if cfg.control_rms { input_rms } else { input_rms.abs() };
        let corr = cfg.corr_gain.apply(level);      // math2
        let l = self.lag.process(corr, &cfg.lag, dt); // stateful lag

        let high_gain = if cfg.use_high_alt {
            cfg.high_gain_alt.apply(l)
        } else {
            cfg.high_gain.apply(l)
        };

        let kick_x = cfg.kick_map.apply(l);
        let snare_x = cfg.snare_map.apply(l);
        let rythm_x = cfg.rythm_map.apply(l);

        ControlOut {
            low_gain: cfg.low_gain.apply(l),
            mid_gain: cfg.mid_gain.apply(l),
            high_gain,
            kick_thresh: cfg.kick_sigmoid.eval(kick_x),   // R4: сигмоида вкл/выкл внутри eval
            snare_thresh: cfg.snare_sigmoid.eval(snare_x),
            rythm_thresh: cfg.rythm_sigmoid.eval(rythm_x),
            lag_value: l,
            kick_x,
            snare_x,
            rythm_x,
        }
    }
}
