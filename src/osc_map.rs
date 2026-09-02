//! Маппинг результатов анализа → OSC-каналы (вынесено из engine).

use crate::config::{Config, OscCfg};

/// Каналы, которые можно включить/выключить в GUI (адрес, подпись).
pub const OSC_CHANNEL_LIST: &[(&str, &str)] = &[
    ("low", "полоса low"),
    ("mid", "полоса mid"),
    ("high", "полоса high"),
    ("kick", "триггер kick (hold)"),
    ("snare", "триггер snare (hold)"),
    ("rythm", "триггер rythm (hold)"),
    ("spectralCentroid", "spectral centroid"),
    ("fmsd", "fmsd"),
    ("smsd", "smsd"),
    ("beatPhase", "фаза такта"),
    ("trigger4k", "доля 1/4 kick"),
    ("trigger8k", "доля 1/8 kick"),
    ("trigger16k", "доля 1/16 kick"),
    ("trigger4s", "доля 1/4 snare"),
    ("trigger8s", "доля 1/8 snare"),
    ("trigger16s", "доля 1/16 snare"),
    ("dsprms", "DSP RMS"),
];

/// Результат одного compute-тика (без транспорта OSC).
#[derive(Clone, Debug, Default)]
pub struct AnalysisFrame {
    pub frame_id: u64,
    pub t_mono: f64,
    pub beat_phase: f32,
    pub levels: [f32; 3],
    pub kick: (f32, f32),
    pub snare: (f32, f32),
    pub rythm: (f32, f32),
    pub centroid: f32,
    pub fms: f32,
    pub sms: f32,
    pub triggers_kick: (f32, f32, f32),
    pub triggers_snare: (f32, f32, f32),
    pub dsp_rms: f32,
}

/// Импульс триггера для фазовой очереди OSC.
#[derive(Clone, Debug)]
pub struct TriggerPulse {
    pub address: &'static str,
    pub phase: f32,
    /// Кадр анализа, в котором impulse возник — по нему OSC отбрасывает залежавшиеся.
    pub frame_id: u64,
}

/// Тип OSC-канала: триггерные не шлём при ~0 (чтобы не затирать приёмник).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OscChannelKind {
    Continuous,
    Trigger,
}

#[derive(Clone, Debug)]
pub struct OscChannel {
    pub address: String,
    pub value: f32,
    /// Семантика канала (триггер vs непрерывный); фильтр нулей снят под QLC+.
    #[allow(dead_code)]
    pub kind: OscChannelKind,
}

/// Снимок для OSC-потока.
#[derive(Clone, Debug, Default)]
pub struct OscSnapshot {
    pub frame_id: u64,
    pub t_mono: f64,
    pub beat_phase: f32,
    pub channels: Vec<OscChannel>,
    pub pulses: Vec<TriggerPulse>,
}

impl OscSnapshot {
    pub fn empty() -> Self {
        Self::default()
    }
}

/// Имена импульсных каналов (не отправлять при value ≈ 0).
pub fn is_trigger_address(addr: &str) -> bool {
    matches!(
        addr,
        "kick" | "snare" | "rythm"
            | "trigger4k" | "trigger8k" | "trigger16k"
            | "trigger4s" | "trigger8s" | "trigger16s"
    )
}

/// Собрать OSC-снимок из кадра анализа.
pub fn build_snapshot(frame: &AnalysisFrame, cfg: &Config, prev_triggers: &mut TriggerState) -> OscSnapshot {
    let mut channels = vec![
        OscChannel { address: "low".into(), value: frame.levels[0], kind: OscChannelKind::Continuous },
        OscChannel { address: "mid".into(), value: frame.levels[1], kind: OscChannelKind::Continuous },
        OscChannel { address: "high".into(), value: frame.levels[2], kind: OscChannelKind::Continuous },
        OscChannel { address: "spectralCentroid".into(), value: frame.centroid, kind: OscChannelKind::Continuous },
        OscChannel { address: "fmsd".into(), value: frame.fms, kind: OscChannelKind::Continuous },
        OscChannel { address: "smsd".into(), value: frame.sms, kind: OscChannelKind::Continuous },
        OscChannel { address: "beatPhase".into(), value: frame.beat_phase, kind: OscChannelKind::Continuous },
        OscChannel { address: "kick".into(), value: frame.kick.1, kind: OscChannelKind::Trigger },
        OscChannel { address: "snare".into(), value: frame.snare.1, kind: OscChannelKind::Trigger },
        OscChannel { address: "rythm".into(), value: frame.rythm.1, kind: OscChannelKind::Trigger },
    ];

    let mut pulses = Vec::new();

    let trig_channels: [(&str, f32); 6] = [
        ("trigger4k", frame.triggers_kick.0),
        ("trigger8k", frame.triggers_kick.1),
        ("trigger16k", frame.triggers_kick.2),
        ("trigger4s", frame.triggers_snare.0),
        ("trigger8s", frame.triggers_snare.1),
        ("trigger16s", frame.triggers_snare.2),
    ];

    for (addr, val) in trig_channels {
        let edge = prev_triggers.rising(addr, val);
        channels.push(OscChannel {
            address: addr.into(),
            value: val,
            kind: OscChannelKind::Trigger,
        });
        if edge {
            pulses.push(TriggerPulse {
                address: addr,
                phase: frame.beat_phase,
                frame_id: frame.frame_id,
            });
        }
    }

    if cfg.dsp_rmspower {
        channels.push(OscChannel {
            address: "dsprms".into(),
            value: frame.dsp_rms,
            kind: OscChannelKind::Continuous,
        });
    }

    OscSnapshot {
        frame_id: frame.frame_id,
        t_mono: frame.t_mono,
        beat_phase: frame.beat_phase,
        channels,
        pulses,
    }
}

/// Отслеживание фронтов счётчиков между кадрами.
#[derive(Default)]
pub struct TriggerState {
    prev: std::collections::HashMap<&'static str, f32>,
}

impl TriggerState {
    fn rising(&mut self, addr: &'static str, val: f32) -> bool {
        let prev = self.prev.get(addr).copied().unwrap_or(0.0);
        let edge = val > 0.5 && prev <= 0.5;
        self.prev.insert(addr, val);
        edge
    }
}

/// Каналы для отправки: фильтр по тумблерам + clip low/mid/high.
pub fn channels_for_send(channels: &[OscChannel], osc: &OscCfg) -> Vec<(String, f32)> {
    channels
        .iter()
        .filter(|c| osc.sends(&c.address))
        .map(|c| {
            let mut v = c.value;
            if osc.clip_levels_at_zero && matches!(c.address.as_str(), "low" | "mid" | "high") {
                v = v.max(0.0);
            }
            (c.address.clone(), v)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame() -> AnalysisFrame {
        AnalysisFrame {
            frame_id: 1,
            levels: [-0.2, 0.5, 0.1],
            kick: (1.0, 1.0),
            snare: (0.0, 0.0),
            rythm: (0.0, 0.0),
            triggers_kick: (1.0, 0.0, 0.0),
            triggers_snare: (0.0, 0.0, 0.0),
            ..AnalysisFrame::default()
        }
    }

    #[test]
    fn snapshot_addresses_unique() {
        let mut st = TriggerState::default();
        let snap = build_snapshot(&frame(), &Config::default(), &mut st);
        let mut addrs: Vec<_> = snap.channels.iter().map(|c| c.address.as_str()).collect();
        addrs.sort();
        let n = addrs.len();
        addrs.dedup();
        assert_eq!(addrs.len(), n);
    }

    #[test]
    fn trigger4k_pulse_on_edge_only() {
        let mut st = TriggerState::default();
        let cfg = Config::default();
        let f = frame();
        let a = build_snapshot(&f, &cfg, &mut st);
        assert_eq!(a.pulses.iter().filter(|p| p.address == "trigger4k").count(), 1);
        let b = build_snapshot(&f, &cfg, &mut st);
        assert_eq!(b.pulses.iter().filter(|p| p.address == "trigger4k").count(), 0);
    }

    #[test]
    fn clip_low_at_zero() {
        let mut osc = OscCfg::default();
        osc.clip_levels_at_zero = true;
        let ch = [OscChannel {
            address: "low".into(),
            value: -0.4,
            kind: OscChannelKind::Continuous,
        }];
        let out = channels_for_send(&ch, &osc);
        assert_eq!(out[0].1, 0.0);
    }

    #[test]
    fn send_channels_off() {
        let mut osc = OscCfg::default();
        osc.set_sends("low", false);
        let ch = [OscChannel {
            address: "low".into(),
            value: 1.0,
            kind: OscChannelKind::Continuous,
        }];
        assert!(channels_for_send(&ch, &osc).is_empty());
    }
}
