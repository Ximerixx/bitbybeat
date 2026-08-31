//! Одна лупа «вход / крутилки / выход» и плакат-схема.
//! GUI только вызывает `open_on_right_click` / `mapper_ui` / `popup` / `poster`.

use crate::config::{Config, GainCfg, LagCfg, SigmoidCfg};
use crate::shared::Metrics;
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Points};
use std::collections::VecDeque;

const HIST: usize = 240;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeId {
    DspRms,
    Corr,
    Lag,
    Band(u8),
    GainLow,
    GainMid,
    GainHigh,
    KickMap,
    SnareMap,
    RythmMap,
    KickSig,
    SnareSig,
    RythmSig,
}

impl ProbeId {
    fn title(self) -> &'static str {
        match self {
            Self::DspRms => "/dsprms",
            Self::Corr => "масштаб зала",
            Self::Lag => "инерция",
            Self::Band(0) => "полоса low",
            Self::Band(1) => "полоса mid",
            Self::Band(2) => "полоса high",
            Self::Band(_) => "полоса",
            Self::GainLow => "gain low (адаптив)",
            Self::GainMid => "gain mid (адаптив)",
            Self::GainHigh => "gain high (адаптив)",
            Self::KickMap => "к порогу kick",
            Self::SnareMap => "к порогу snare",
            Self::RythmMap => "к порогу rythm",
            Self::KickSig => "кривая kick",
            Self::SnareSig => "кривая snare",
            Self::RythmSig => "кривая rythm",
        }
    }
}

/// История сырых входов; выход для мапперов пересчитывается текущим конфигом.
pub struct ProbeHistory {
    last_frame: u64,
    input_rms: VecDeque<f32>,
    lag: VecDeque<f32>,
    band_rms: [VecDeque<f32>; 3],
    band_lvl: [VecDeque<f32>; 3],
    kick_x: VecDeque<f32>,
    snare_x: VecDeque<f32>,
    rythm_x: VecDeque<f32>,
}

impl ProbeHistory {
    pub fn new() -> Self {
        Self {
            last_frame: 0,
            input_rms: VecDeque::with_capacity(HIST),
            lag: VecDeque::with_capacity(HIST),
            band_rms: [VecDeque::with_capacity(HIST), VecDeque::with_capacity(HIST), VecDeque::with_capacity(HIST)],
            band_lvl: [VecDeque::with_capacity(HIST), VecDeque::with_capacity(HIST), VecDeque::with_capacity(HIST)],
            kick_x: VecDeque::with_capacity(HIST),
            snare_x: VecDeque::with_capacity(HIST),
            rythm_x: VecDeque::with_capacity(HIST),
        }
    }

    pub fn push(&mut self, m: &Metrics) {
        if m.compute_frame_id == 0 || m.compute_frame_id == self.last_frame {
            return;
        }
        self.last_frame = m.compute_frame_id;
        push_cap(&mut self.input_rms, m.input_rms);
        push_cap(&mut self.lag, m.control.lag_value);
        for i in 0..3 {
            push_cap(&mut self.band_rms[i], m.band_rms[i]);
            push_cap(&mut self.band_lvl[i], m.band_levels[i]);
        }
        push_cap(&mut self.kick_x, m.control.kick_x);
        push_cap(&mut self.snare_x, m.control.snare_x);
        push_cap(&mut self.rythm_x, m.control.rythm_x);
    }
}

fn push_cap(q: &mut VecDeque<f32>, v: f32) {
    if q.len() >= HIST {
        q.pop_front();
    }
    q.push_back(v);
}

fn live(id: ProbeId, m: &Metrics, cfg: &Config) -> (f32, f32) {
    let rms = m.input_rms;
    let lag = m.control.lag_value;
    match id {
        ProbeId::DspRms => (rms, cfg.dsp_gain.apply(rms)),
        ProbeId::Corr => (rms, cfg.control.corr_gain.apply(rms)),
        ProbeId::Lag => (cfg.control.corr_gain.apply(rms), lag),
        ProbeId::Band(i) => {
            let i = i.min(2) as usize;
            (m.band_rms[i], m.band_levels[i])
        }
        ProbeId::GainLow => (lag, cfg.control.low_gain.apply(lag)),
        ProbeId::GainMid => (lag, cfg.control.mid_gain.apply(lag)),
        ProbeId::GainHigh => {
            let g = if cfg.control.use_high_alt { &cfg.control.high_gain_alt } else { &cfg.control.high_gain };
            (lag, g.apply(lag))
        }
        ProbeId::KickMap => (lag, cfg.control.kick_map.apply(lag)),
        ProbeId::SnareMap => (lag, cfg.control.snare_map.apply(lag)),
        ProbeId::RythmMap => (lag, cfg.control.rythm_map.apply(lag)),
        ProbeId::KickSig => (m.control.kick_x, cfg.control.kick_sigmoid.eval(m.control.kick_x)),
        ProbeId::SnareSig => (m.control.snare_x, cfg.control.snare_sigmoid.eval(m.control.snare_x)),
        ProbeId::RythmSig => (m.control.rythm_x, cfg.control.rythm_sigmoid.eval(m.control.rythm_x)),
    }
}

fn series_in<'a>(id: ProbeId, h: &'a ProbeHistory) -> &'a VecDeque<f32> {
    match id {
        ProbeId::DspRms | ProbeId::Corr => &h.input_rms,
        ProbeId::Lag => &h.input_rms,
        ProbeId::Band(i) => &h.band_rms[i.min(2) as usize],
        ProbeId::GainLow | ProbeId::GainMid | ProbeId::GainHigh | ProbeId::KickMap | ProbeId::SnareMap | ProbeId::RythmMap => &h.lag,
        ProbeId::KickSig => &h.kick_x,
        ProbeId::SnareSig => &h.snare_x,
        ProbeId::RythmSig => &h.rythm_x,
    }
}

fn map_out(id: ProbeId, x: f32, cfg: &Config) -> f32 {
    match id {
        ProbeId::DspRms => cfg.dsp_gain.apply(x),
        ProbeId::Corr => cfg.control.corr_gain.apply(x),
        ProbeId::Lag => cfg.control.corr_gain.apply(x),
        ProbeId::Band(_) => x, // уровни не пересчитать без DSP; берём отдельно
        ProbeId::GainLow => cfg.control.low_gain.apply(x),
        ProbeId::GainMid => cfg.control.mid_gain.apply(x),
        ProbeId::GainHigh => {
            if cfg.control.use_high_alt {
                cfg.control.high_gain_alt.apply(x)
            } else {
                cfg.control.high_gain.apply(x)
            }
        }
        ProbeId::KickMap => cfg.control.kick_map.apply(x),
        ProbeId::SnareMap => cfg.control.snare_map.apply(x),
        ProbeId::RythmMap => cfg.control.rythm_map.apply(x),
        ProbeId::KickSig => cfg.control.kick_sigmoid.eval(x),
        ProbeId::SnareSig => cfg.control.snare_sigmoid.eval(x),
        ProbeId::RythmSig => cfg.control.rythm_sigmoid.eval(x),
    }
}

/// ПКМ над блоком (даже если клик съел слайдер/график).
pub fn open_on_right_click(response: &egui::Response, id: ProbeId, slot: &mut Option<ProbeId>) {
    let over = response.hovered() || response.contains_pointer();
    let rmb = response.ctx.input(|i| i.pointer.secondary_clicked());
    if over && rmb {
        *slot = Some(id);
    }
}

pub fn lupa_button(ui: &mut egui::Ui, id: ProbeId, slot: &mut Option<ProbeId>) {
    if ui
        .small_button("лупа")
        .on_hover_text("график входа/выхода по времени")
        .clicked()
    {
        *slot = Some(id);
    }
}

/// Блок маппера + ПКМ / кнопка лупа. Возвращает, менялся ли конфиг.
pub fn mapper_ui(
    ui: &mut egui::Ui,
    title: &str,
    hint: &str,
    g: &mut GainCfg,
    id: ProbeId,
    slot: &mut Option<ProbeId>,
) -> bool {
    let mut dirty = false;
    let r = ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.strong(title);
            lupa_button(ui, id, slot);
        });
        ui.weak(hint);
        dirty |= gain_knobs(ui, g);
    })
    .response;
    open_on_right_click(&r, id, slot);
    dirty
}

pub fn gain_knobs(ui: &mut egui::Ui, g: &mut GainCfg) -> bool {
    let mut dirty = false;
    ui.horizontal(|ui| {
        dirty |= ui.add(egui::DragValue::new(&mut g.preoff).speed(0.01).prefix("до +")).changed();
        dirty |= ui.add(egui::DragValue::new(&mut g.gain).speed(0.01).prefix("x")).changed();
        dirty |= ui.add(egui::DragValue::new(&mut g.postoff).speed(0.01).prefix("после +")).changed();
        if let Some((lo, hi)) = g.torange.as_mut() {
            dirty |= ui.add(egui::DragValue::new(lo).speed(0.01).prefix("мин ")).changed();
            dirty |= ui.add(egui::DragValue::new(hi).speed(0.01).prefix("макс ")).changed();
        }
    });
    dirty
}

/// Кривая сигмоиды + точки: голубая = вход x, жёлтая = выход (порог).
fn sigmoid_plot(ui: &mut egui::Ui, plot_id: &str, s: &SigmoidCfg, live_x: Option<f32>, height: f32) {
    let xmax = (s.center * 2.0)
        .max(1.0)
        .max(live_x.unwrap_or(0.0) * 1.15)
        .max(0.5) as f64;
    let curve: PlotPoints = (0..=200)
        .map(|i| {
            let x = xmax * i as f64 / 200.0;
            [x, s.eval(x as f32) as f64]
        })
        .collect();
    Plot::new(plot_id)
        .height(height)
        .include_y(0.0)
        .show(ui, |p| {
            p.line(Line::new(curve).name("сигмоида"));
            if let Some(x) = live_x {
                let y = s.eval(x);
                p.points(
                    Points::new(vec![[x as f64, 0.0]])
                        .radius(4.0_f32)
                        .color(egui::Color32::LIGHT_BLUE)
                        .name("вход x (пришло)"),
                );
                p.points(
                    Points::new(vec![[x as f64, y as f64]])
                        .radius(6.0_f32)
                        .color(egui::Color32::YELLOW)
                        .name("выход (порог)"),
                );
            }
        });
}

/// Как раньше в колонке: кривая на месте, «окно» увеличивает график, ПКМ = лупа.
pub fn sigmoid_ui(
    ui: &mut egui::Ui,
    label: &str,
    s: &mut SigmoidCfg,
    live_x: f32,
    id: ProbeId,
    slot: &mut Option<ProbeId>,
    open_win: &mut bool,
) -> bool {
    let mut dirty = false;
    let r = ui.group(|ui| {
        ui.horizontal(|ui| {
            dirty |= ui
                .checkbox(&mut s.enabled, "")
                .on_hover_text("вкл - сигмоида; выкл - значение проходит линейно (как есть)")
                .changed();
            ui.strong(label);
            lupa_button(ui, id, slot);
            if ui
                .small_button("окно")
                .on_hover_text("открыть в отдельном окне с большим графиком")
                .clicked()
            {
                *open_win = true;
            }
        });
        dirty |= sigmoid_knobs(ui, s);
        sigmoid_plot(ui, label, s, Some(live_x), 90.0);
    })
    .response;
    open_on_right_click(&r, id, slot);
    dirty
}

/// Большое окно той же кривой (кнопка «окно»).
pub fn sigmoid_window(
    ctx: &egui::Context,
    title: &str,
    open: &mut bool,
    s: &mut SigmoidCfg,
    live_x: f32,
) -> bool {
    if !*open {
        return false;
    }
    let mut dirty = false;
    let mut keep = *open;
    egui::Window::new(title)
        .open(&mut keep)
        .default_size([420.0, 380.0])
        .resizable(true)
        .show(ctx, |ui| {
            dirty |= ui.checkbox(&mut s.enabled, "сигмоида вкл").changed();
            dirty |= sigmoid_knobs(ui, s);
            let h = (ui.available_height() - 10.0).max(120.0);
            sigmoid_plot(ui, &format!("win_{title}"), s, Some(live_x), h);
        });
    *open = keep;
    dirty
}

fn sigmoid_knobs(ui: &mut egui::Ui, s: &mut SigmoidCfg) -> bool {
    let mut dirty = false;
    ui.horizontal(|ui| {
        dirty |= ui
            .add(egui::DragValue::new(&mut s.ceil).speed(0.01).prefix("ceil "))
            .on_hover_text("потолок: максимум выхода кривой")
            .changed();
        dirty |= ui
            .add(egui::DragValue::new(&mut s.center).speed(0.05).prefix("c "))
            .on_hover_text("центр: значение входа, где кривая на половине высоты")
            .changed();
        let a = ui
            .checkbox(&mut s.asymmetric, "асимм")
            .on_hover_text("раздельная крутизна левой и правой половины относительно центра");
        if a.changed() {
            dirty = true;
            if s.asymmetric {
                s.k_left = s.k;
                s.k_right = s.k;
            }
        }
    });
    ui.horizontal(|ui| {
        if s.asymmetric {
            dirty |= ui
                .add(egui::DragValue::new(&mut s.k_left).speed(0.01).prefix("<k "))
                .on_hover_text("крутизна левой половины (x < центра): круче -> резче вход снизу")
                .changed();
            dirty |= ui
                .add(egui::DragValue::new(&mut s.k_right).speed(0.01).prefix("k> "))
                .on_hover_text("крутизна правой половины (x > центра): круче -> резче насыщение сверху")
                .changed();
        } else {
            dirty |= ui
                .add(egui::DragValue::new(&mut s.k).speed(0.01).prefix("k "))
                .on_hover_text("общая крутизна: больше -> резче переход")
                .changed();
        }
    });
    dirty
}

fn lag_knobs(ui: &mut egui::Ui, l: &mut LagCfg) -> bool {
    let mut dirty = false;
    dirty |= ui.add(egui::Slider::new(&mut l.lag_up, 0.0..=10.0).text("вверх")).changed();
    dirty |= ui.add(egui::Slider::new(&mut l.lag_dn, 0.0..=10.0).text("вниз")).changed();
    dirty |= ui.add(egui::Slider::new(&mut l.accel_up, 0.1..=10.0).text("ускор. вверх")).changed();
    dirty |= ui.add(egui::Slider::new(&mut l.accel_dn, 0.1..=10.0).text("ускор. вниз")).changed();
    dirty
}

fn band_knobs(ui: &mut egui::Ui, cfg: &mut Config, i: usize, adaptive: bool, live_gain: f32) -> bool {
    let Some(b) = cfg.bands.get_mut(i) else { return false };
    let mut dirty = false;
    dirty |= ui.checkbox(&mut b.active, "полоса вкл").changed();
    dirty |= ui.add(egui::DragValue::new(&mut b.cutoff_hz).speed(1.0).suffix(" Hz")).changed();
    dirty |= ui.add(egui::Slider::new(&mut b.pregain, 0.0..=8.0).text("до порога x")).changed();
    dirty |= ui.add(egui::Slider::new(&mut b.threshold, 0.0..=1.0).text("порог тишины")).changed();
    if adaptive {
        ui.label(format!("gain с адаптива: {live_gain:.2}"));
    } else {
        dirty |= ui.add(egui::Slider::new(&mut b.gain, 0.0..=10.0).text("крутизна")).changed();
    }
    dirty |= ui.add(egui::Slider::new(&mut b.add, -1.0..=1.0).text("сдвиг после")).changed();
    dirty |= ui.add(egui::Slider::new(&mut b.smooth_s, 0.0..=1.0).text("сглаживание")).changed();
    dirty
}

fn knobs_for(ui: &mut egui::Ui, id: ProbeId, cfg: &mut Config, m: &Metrics) -> bool {
    match id {
        ProbeId::DspRms => {
            let mut d = ui.checkbox(&mut cfg.dsp_rmspower, "считать /dsprms").changed();
            d |= gain_knobs(ui, &mut cfg.dsp_gain);
            d
        }
        ProbeId::Corr => gain_knobs(ui, &mut cfg.control.corr_gain),
        ProbeId::Lag => lag_knobs(ui, &mut cfg.control.lag),
        ProbeId::Band(i) => {
            let i = i.min(2) as usize;
            let live = [m.control.low_gain, m.control.mid_gain, m.control.high_gain][i];
            band_knobs(ui, cfg, i, cfg.control.enabled, live)
        }
        ProbeId::GainLow => gain_knobs(ui, &mut cfg.control.low_gain),
        ProbeId::GainMid => gain_knobs(ui, &mut cfg.control.mid_gain),
        ProbeId::GainHigh => {
            let mut d = ui.checkbox(&mut cfg.control.use_high_alt, "запасной маппер").changed();
            if cfg.control.use_high_alt {
                d |= gain_knobs(ui, &mut cfg.control.high_gain_alt);
            } else {
                d |= gain_knobs(ui, &mut cfg.control.high_gain);
            }
            d
        }
        ProbeId::KickMap => gain_knobs(ui, &mut cfg.control.kick_map),
        ProbeId::SnareMap => gain_knobs(ui, &mut cfg.control.snare_map),
        ProbeId::RythmMap => gain_knobs(ui, &mut cfg.control.rythm_map),
        ProbeId::KickSig => {
            let mut d = ui.checkbox(&mut cfg.control.kick_sigmoid.enabled, "кривая вкл").changed();
            d |= sigmoid_knobs(ui, &mut cfg.control.kick_sigmoid);
            d
        }
        ProbeId::SnareSig => {
            let mut d = ui.checkbox(&mut cfg.control.snare_sigmoid.enabled, "кривая вкл").changed();
            d |= sigmoid_knobs(ui, &mut cfg.control.snare_sigmoid);
            d
        }
        ProbeId::RythmSig => {
            let mut d = ui.checkbox(&mut cfg.control.rythm_sigmoid.enabled, "кривая вкл").changed();
            d |= sigmoid_knobs(ui, &mut cfg.control.rythm_sigmoid);
            d
        }
    }
}

fn plot_io(ui: &mut egui::Ui, id: ProbeId, hist: &ProbeHistory, cfg: &Config, vin: f32, vout: f32) {
    ui.label(format!("вход {vin:.3}    выход {vout:.3}"));
    let ins = series_in(id, hist);
    let (yin, yout): (PlotPoints, PlotPoints) = if matches!(id, ProbeId::Band(_)) {
        let i = match id {
            ProbeId::Band(i) => i.min(2) as usize,
            _ => 0,
        };
        let a = ins.iter().enumerate().map(|(t, v)| [t as f64, *v as f64]).collect();
        let b = hist.band_lvl[i]
            .iter()
            .enumerate()
            .map(|(t, v)| [t as f64, *v as f64])
            .collect();
        (a, b)
    } else if id == ProbeId::Lag {
        let a = hist
            .input_rms
            .iter()
            .enumerate()
            .map(|(t, v)| [t as f64, cfg.control.corr_gain.apply(*v) as f64])
            .collect();
        let b = hist.lag.iter().enumerate().map(|(t, v)| [t as f64, *v as f64]).collect();
        (a, b)
    } else {
        let a = ins.iter().enumerate().map(|(t, v)| [t as f64, *v as f64]).collect();
        let b = ins
            .iter()
            .enumerate()
            .map(|(t, v)| [t as f64, map_out(id, *v, cfg) as f64])
            .collect();
        (a, b)
    };
    Plot::new(format!("probe_in_{id:?}"))
        .height(72.0)
        .show(ui, |p| p.line(Line::new(yin).name("вход").color(egui::Color32::from_rgb(120, 180, 255))));
    Plot::new(format!("probe_out_{id:?}"))
        .height(72.0)
        .show(ui, |p| p.line(Line::new(yout).name("выход").color(egui::Color32::from_rgb(255, 200, 80))));
}

pub struct ProbeUi<'a> {
    pub slot: &'a mut Option<ProbeId>,
    pub entered: &'a mut bool,
    pub hist: &'a ProbeHistory,
    pub metrics: &'a Metrics,
}

fn close_probe(slot: &mut Option<ProbeId>, entered: &mut bool) {
    *slot = None;
    *entered = false;
}

/// Лупа: сверху вход, в центре крутилки, снизу выход.
/// Крестик / Esc всегда закрывают. Уход курсора - только после того, как курсор уже был внутри.
pub fn popup(ctx: &egui::Context, ui_state: ProbeUi<'_>, cfg: &mut Config) -> bool {
    let Some(id) = *ui_state.slot else { return false };
    let (vin, vout) = live(id, ui_state.metrics, cfg);
    let mut dirty = false;
    let mut open = true;
    let inner = egui::Window::new(id.title())
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .default_size([340.0, 360.0])
        .show(ctx, |ui| {
            plot_io(ui, id, ui_state.hist, cfg, vin, vout);
            ui.separator();
            dirty |= knobs_for(ui, id, cfg, ui_state.metrics);
            ui.separator();
            ui.weak("кнопка лупа или ПКМ по блоку. Esc / крестик - закрыть.");
        });
    if !open || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close_probe(ui_state.slot, ui_state.entered);
        return dirty;
    }
    let hovered = inner.as_ref().is_some_and(|i| i.response.hovered() || i.response.contains_pointer());
    if hovered {
        *ui_state.entered = true;
    }
    let dragging = ctx.input(|i| i.pointer.any_down());
    if *ui_state.entered && !hovered && !dragging {
        close_probe(ui_state.slot, ui_state.entered);
    }
    dirty
}

fn node(ui: &mut egui::Ui, title: &str, id: ProbeId, slot: &mut Option<ProbeId>, entered: &mut bool, body: impl FnOnce(&mut egui::Ui) -> bool) -> bool {
    let mut dirty = false;
    let was = *slot;
    let r = ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.strong(title);
            lupa_button(ui, id, slot);
        });
        dirty |= body(ui);
    })
    .response;
    open_on_right_click(&r, id, slot);
    r.on_hover_text("лупа или ПКМ - вход/выход");
    if *slot == Some(id) && was != Some(id) {
        *entered = false;
    }
    dirty
}

/// Плакат: ноды без перетаскивания, крутилки на месте, ПКМ = лупа.
pub fn poster(
    ctx: &egui::Context,
    open: &mut bool,
    cfg: &mut Config,
    m: &Metrics,
    slot: &mut Option<ProbeId>,
    entered: &mut bool,
) -> bool {
    let mut dirty = false;
    egui::Window::new("Схема тракта")
        .open(open)
        .default_size([640.0, 860.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.weak("Плакат: только смотреть цепочку и крутить. ПКМ по блоку - график входа/выхода.");
            egui::ScrollArea::both().show(ui, |ui| {
                ui.horizontal(|ui| {
                    dirty |= node(ui, "1. RMS входа", ProbeId::Corr, slot, entered, |ui| {
                        ui.label(format!("{:.4}", m.input_rms));
                        false
                    });
                    ui.label("->");
                    dirty |= node(ui, "2. /dsprms", ProbeId::DspRms, slot, entered, |ui| {
                        let mut d = ui.checkbox(&mut cfg.dsp_rmspower, "слать").changed();
                        d |= gain_knobs(ui, &mut cfg.dsp_gain);
                        ui.label(format!("сейчас {:.3}", m.dsp_rms));
                        d
                    });
                });
                ui.label("v  тот же RMS идёт в адаптив и в полосы");
                ui.horizontal(|ui| {
                    dirty |= node(ui, "3. масштаб зала", ProbeId::Corr, slot, entered, |ui| {
                        gain_knobs(ui, &mut cfg.control.corr_gain)
                    });
                    ui.label("->");
                    dirty |= node(ui, "4. инерция", ProbeId::Lag, slot, entered, |ui| {
                        let d = lag_knobs(ui, &mut cfg.control.lag);
                        ui.label(format!("L={:.3}", m.control.lag_value));
                        d
                    });
                });
                ui.label("v  L правит gain полос и пороги");
                ui.vertical(|ui| {
                    dirty |= node(ui, "5a. gain low", ProbeId::GainLow, slot, entered, |ui| {
                        gain_knobs(ui, &mut cfg.control.low_gain)
                    });
                    dirty |= node(ui, "5b. gain mid", ProbeId::GainMid, slot, entered, |ui| {
                        gain_knobs(ui, &mut cfg.control.mid_gain)
                    });
                    dirty |= node(ui, "5c. gain high", ProbeId::GainHigh, slot, entered, |ui| {
                        gain_knobs(ui, &mut cfg.control.high_gain)
                    });
                });
                ui.separator();
                ui.vertical(|ui| {
                    for i in 0..3u8 {
                        let name = ["6. low", "6. mid", "6. high"][i as usize];
                        dirty |= node(ui, name, ProbeId::Band(i), slot, entered, |ui| {
                            let live = [m.control.low_gain, m.control.mid_gain, m.control.high_gain][i as usize];
                            band_knobs(ui, cfg, i as usize, cfg.control.enabled, live)
                        });
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        dirty |= node(ui, "7. порог kick", ProbeId::KickMap, slot, entered, |ui| {
                            gain_knobs(ui, &mut cfg.control.kick_map)
                        });
                        dirty |= node(ui, "кривая kick", ProbeId::KickSig, slot, entered, |ui| {
                            sigmoid_knobs(ui, &mut cfg.control.kick_sigmoid)
                        });
                    });
                    ui.horizontal(|ui| {
                        dirty |= node(ui, "7. порог snare", ProbeId::SnareMap, slot, entered, |ui| {
                            gain_knobs(ui, &mut cfg.control.snare_map)
                        });
                        dirty |= node(ui, "кривая snare", ProbeId::SnareSig, slot, entered, |ui| {
                            sigmoid_knobs(ui, &mut cfg.control.snare_sigmoid)
                        });
                    });
                    ui.horizontal(|ui| {
                        dirty |= node(ui, "7. порог rythm", ProbeId::RythmMap, slot, entered, |ui| {
                            gain_knobs(ui, &mut cfg.control.rythm_map)
                        });
                        dirty |= node(ui, "кривая rythm", ProbeId::RythmSig, slot, entered, |ui| {
                            sigmoid_knobs(ui, &mut cfg.control.rythm_sigmoid)
                        });
                    });
                });
                ui.separator();
                ui.label(format!(
                    "OSC <-  low {:+.3}  mid {:+.3}  high {:+.3}   kick {:.0}  snare {:.0}",
                    m.band_levels[0], m.band_levels[1], m.band_levels[2], m.kick.1, m.snare.1
                ));
            });
        });
    dirty
}
