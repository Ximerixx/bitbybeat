//! GUI на egui: тумблеры/крутилки всего тракта + метры/спектр (md_plans/10 R0/R5).

use crate::audio;
use crate::config::{GainCfg, SigmoidCfg, Source};
use crate::shared::Shared;
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Points};
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct App {
    shared: Arc<Shared>,
    devices: Vec<audio::DeviceInfo>,
    pulse_sources: Vec<audio::PulseSource>,
    preset_path: String,
    status: String,
    /// Отдельные окна для сигмоид: [kick, snare, rythm].
    sig_win: [bool; 3],
}

impl App {
    pub fn new(shared: Arc<Shared>) -> Self {
        Self {
            shared,
            devices: audio::list_input_devices(),
            pulse_sources: audio::list_pulse_sources(),
            preset_path: "preset.ron".into(),
            status: String::new(),
            sig_win: [false; 3],
        }
    }
}

/// Большая «лампа» — квадрат, светящийся по значению 0..1 (md_plans/10: индикаторы kick/snare/rythm).
fn lamp(ui: &mut egui::Ui, label: &str, on: f32, color: egui::Color32) {
    let size = egui::vec2(84.0, 84.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter();
    let k = on.clamp(0.0, 1.0);
    let bg = egui::Color32::from_rgb(
        (color.r() as f32 * (0.18 + 0.82 * k)) as u8,
        (color.g() as f32 * (0.18 + 0.82 * k)) as u8,
        (color.b() as f32 * (0.18 + 0.82 * k)) as u8,
    );
    painter.rect_filled(rect, 8.0, bg);
    painter.rect_stroke(rect, 8.0, egui::Stroke::new(2.0_f32, color));
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(16.0),
        egui::Color32::WHITE,
    );
}

/// Крутилки маппера (Math CHOP): x' = (x + preoff)·gain + postoff, затем опц. remap в torange.
fn gain_ui(ui: &mut egui::Ui, label: &str, g: &mut GainCfg) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(&mut g.preoff).speed(0.01).prefix("pre "))
            .on_hover_text("pre-offset: прибавляется к входу ДО умножения");
        ui.add(egui::DragValue::new(&mut g.gain).speed(0.01).prefix("×"))
            .on_hover_text("множитель сигнала (gain)");
        ui.add(egui::DragValue::new(&mut g.postoff).speed(0.01).prefix("post "))
            .on_hover_text("post-offset: прибавляется ПОСЛЕ умножения");
        if let Some((lo, hi)) = g.torange.as_mut() {
            ui.add(egui::DragValue::new(lo).speed(0.01).prefix("→lo "))
                .on_hover_text("нижняя граница выходного диапазона (remap 0..1 → lo..hi)");
            ui.add(egui::DragValue::new(hi).speed(0.01).prefix("→hi "))
                .on_hover_text("верхняя граница выходного диапазона");
        }
    });
}

/// Отрисовать кривую сигмоиды + две точки: вход x (пришло) и выход eval(x) (ушло/примагнитилось).
fn sigmoid_plot(ui: &mut egui::Ui, id: &str, s: &SigmoidCfg, live_x: Option<f32>, height: f32) {
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
    Plot::new(id)
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

/// Компактный редактор сигмоиды (в общей колонке).
fn sigmoid_ctrl(ui: &mut egui::Ui, label: &str, s: &mut SigmoidCfg, live_x: Option<f32>, open_win: &mut bool) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.checkbox(&mut s.enabled, "")
                .on_hover_text("вкл — сигмоида; выкл — значение проходит линейно (как есть)");
            ui.strong(label);
            if ui.small_button("⧉ окно").on_hover_text("открыть в отдельном окне с большим графиком").clicked() {
                *open_win = true;
            }
        });
        sigmoid_params(ui, s);
        sigmoid_plot(ui, label, s, live_x, 90.0);
    });
}

/// Ряд крутилок сигмоиды с подсказками.
fn sigmoid_params(ui: &mut egui::Ui, s: &mut SigmoidCfg) {
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut s.ceil).speed(0.01).prefix("ceil "))
            .on_hover_text("потолок: максимум выхода кривой");
        ui.add(egui::DragValue::new(&mut s.center).speed(0.05).prefix("c "))
            .on_hover_text("центр: значение входа, где кривая на половине высоты");
        let a = ui.checkbox(&mut s.asymmetric, "асимм")
            .on_hover_text("раздельная крутизна левой и правой половины относительно центра");
        // при включении сеем половины из общей k — стартуем от текущей симметричной кривой
        if a.changed() && s.asymmetric {
            s.k_left = s.k;
            s.k_right = s.k;
        }
    });
    ui.horizontal(|ui| {
        if s.asymmetric {
            ui.add(egui::DragValue::new(&mut s.k_left).speed(0.01).prefix("◄k "))
                .on_hover_text("крутизна левой половины (x < центра): круче → резче вход снизу");
            ui.add(egui::DragValue::new(&mut s.k_right).speed(0.01).prefix("k► "))
                .on_hover_text("крутизна правой половины (x > центра): круче → резче насыщение сверху");
        } else {
            ui.add(egui::DragValue::new(&mut s.k).speed(0.01).prefix("k "))
                .on_hover_text("общая крутизна: больше → резче переход");
        }
    });
}

fn meter(ui: &mut egui::Ui, label: &str, v: f32, max: f32) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}: {v:6.3}"));
        let frac = (v / max).clamp(0.0, 1.0);
        ui.add(egui::ProgressBar::new(frac).desired_width(160.0));
    });
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint(); // живые метры
        let shared = self.shared.clone();

        // ─── Левая панель: вход + пресеты ───
        egui::SidePanel::left("left").resizable(true).default_width(320.0).show(ctx, |ui| {
            ui.heading("Вход / Источник");
            let mut cfg = shared.config.lock().unwrap();

            egui::ComboBox::from_label("source")
                .selected_text(match cfg.input.source { Source::Device => "Устройство", Source::File => "Файл (опц.)" })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut cfg.input.source, Source::Device, "Устройство");
                    ui.selectable_value(&mut cfg.input.source, Source::File, "Файл (опц.)");
                });

            let pm = ui.checkbox(&mut cfg.input.prefer_monitor, "предпочитать monitor-источники")
                .on_hover_text("при включении автоматически выбирает первый monitor выхода (системный звук)");
            if pm.changed() && cfg.input.prefer_monitor {
                if let Some(mon) = self.pulse_sources.iter().find(|s| s.is_monitor) {
                    cfg.input.pulse_source = Some(mon.name.clone());
                    shared.restart_audio.store(true, Ordering::Relaxed);
                }
            }

            // PulseAudio-источники (в т.ч. .monitor выходов) — захват через parec (мимо ALSA-плагина).
            let cur_pulse = match &cfg.input.pulse_source {
                Some(n) => self.pulse_sources.iter().find(|s| &s.name == n)
                    .map(|s| s.label().to_string()).unwrap_or_else(|| n.clone()),
                None => "— нет (ALSA-устройство) —".into(),
            };
            egui::ComboBox::from_label("pulse source (monitor)")
                .selected_text(cur_pulse)
                .width(280.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut cfg.input.pulse_source, None, "— нет (ALSA-устройство) —");
                    for s in &self.pulse_sources {
                        let tag = if s.is_monitor { format!("🔁 {}", s.label()) } else { s.label().to_string() };
                        ui.selectable_value(&mut cfg.input.pulse_source, Some(s.name.clone()), tag);
                    }
                })
                .response
                .on_hover_text("захват системного звука: выберите 🔁 monitor вашего выхода (через parec, без паник cpal)");

            let cur = cfg.input.device.clone().unwrap_or_else(|| "— default —".into());
            ui.add_enabled_ui(cfg.input.pulse_source.is_none(), |ui| {
                egui::ComboBox::from_label("ALSA device")
                    .selected_text(cur)
                    .width(280.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut cfg.input.device, None, "— default —");
                        for d in &self.devices {
                            let base = format!("{} [{}ch]", d.name, d.channels);
                            let tag = if audio::is_monitor(&d.name) { format!("🔁 {base}") } else { base };
                            ui.selectable_value(&mut cfg.input.device, Some(d.name.clone()), tag);
                        }
                    });
            });

            // Выбор каналов для многоканальных устройств (микшер/интерфейс).
            if cfg.input.pulse_source.is_none() {
                let nch = match &cfg.input.device {
                    Some(name) => self.devices.iter().find(|d| &d.name == name).map(|d| d.channels).unwrap_or(0),
                    None => 0,
                };
                if nch > 1 {
                    ui.label(format!("каналы устройства: {nch} — выбери нужные (1–2)"));
                    egui::ScrollArea::horizontal().max_height(64.0).show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            for i in 0..nch as usize {
                                let mut on = cfg.input.channels_pick.contains(&i);
                                if ui.toggle_value(&mut on, format!("{i}"))
                                    .on_hover_text(format!("канал {i}"))
                                    .changed()
                                {
                                    if on { cfg.input.channels_pick.push(i); }
                                    else { cfg.input.channels_pick.retain(|&x| x != i); }
                                    cfg.input.channels_pick.sort_unstable();
                                }
                            }
                        });
                    });
                    ui.weak("пусто = моно-даунмикс всех; выбранные усредняются в моно");
                }
            }
            ui.horizontal(|ui| {
                if ui.button("обновить списки").clicked() {
                    self.devices = audio::list_input_devices();
                    self.pulse_sources = audio::list_pulse_sources();
                    if cfg.input.prefer_monitor && cfg.input.pulse_source.is_none() {
                        if let Some(mon) = self.pulse_sources.iter().find(|s| s.is_monitor) {
                            cfg.input.pulse_source = Some(mon.name.clone());
                        }
                    }
                }
                if ui.button("применить (restart)").clicked() {
                    shared.restart_audio.store(true, Ordering::Relaxed);
                }
            });

            ui.separator();
            ui.heading("Пре-обработка");
            ui.checkbox(&mut cfg.compressor.enabled, "компрессор (audiodyna)")
                .on_hover_text("downward-компрессор входа; в эталоне был обойдён (dead node)");
            if cfg.compressor.enabled {
                let c = &mut cfg.compressor.cfg;
                ui.add(egui::Slider::new(&mut c.threshold_db, -60.0..=0.0).text("thr dB"))
                    .on_hover_text("порог: выше него сигнал сжимается");
                ui.add(egui::Slider::new(&mut c.ratio, 0.05..=4.0).text("ratio"))
                    .on_hover_text("коэффициент сжатия");
                ui.add(egui::Slider::new(&mut c.makeup_db, -12.0..=24.0).text("makeup dB"))
                    .on_hover_text("компенсация громкости после сжатия");
            }
            ui.checkbox(&mut cfg.dsp_rmspower, "RMS-power во входную DSP-ветвь (R2)")
                .on_hover_text("опциональная нода RMS в ветви анализа (вставляется по ситуации)");
            gain_ui(ui, "DSP-гейн (math1)", &mut cfg.dsp_gain);

            ui.separator();
            ui.heading("Частоты");
            ui.add(egui::Slider::new(&mut cfg.compute_rate_hz, 30.0..=480.0).text("обсчёт, Гц"))
                .on_hover_text("частота DSP/детекторов; выше = меньше задержка отклика (CPU почти не растёт)");
            ui.add(egui::Slider::new(&mut cfg.osc_rate_hz, 1.0..=480.0).text("OSC, Гц"))
                .on_hover_text("частота отправки OSC; отдельный таймер, шлёт последний посчитанный снимок");

            ui.separator();
            ui.heading("OSC");
            ui.checkbox(&mut cfg.osc.enabled, "включён");
            ui.add(egui::DragValue::new(&mut cfg.osc.port).prefix("port "))
                .on_hover_text("UDP-порт вывода (эталон: 7700)");
            ui.checkbox(&mut cfg.osc.bundle, "bundle")
                .on_hover_text("слать одним OSC-бандлом вместо отдельных сообщений");

            ui.separator();
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.preset_path);
            });
            ui.horizontal(|ui| {
                if ui.button("сохранить пресет").clicked() {
                    self.status = match cfg.save_ron(&self.preset_path) {
                        Ok(_) => "сохранено".into(),
                        Err(e) => format!("ошибка: {e}"),
                    };
                }
                if ui.button("загрузить").clicked() {
                    match crate::config::Config::load_ron(&self.preset_path) {
                        Ok(new) => { *cfg = new; self.status = "загружено".into(); shared.restart_audio.store(true, Ordering::Relaxed); }
                        Err(e) => self.status = format!("ошибка: {e}"),
                    }
                }
            });
            ui.label(&self.status);
        });

        // ─── Правая панель: метры/спектр ───
        egui::SidePanel::right("right").resizable(true).default_width(340.0).show(ctx, |ui| {
            let m = shared.metrics.lock().unwrap().clone();
            ui.heading("Метры");
            ui.label(format!("устройство: {}", m.device_name));
            ui.label(format!("sr: {:.0} Гц   OSC-каналов: {}", m.sample_rate, m.osc_channels));
            if let Some(e) = &m.error { ui.colored_label(egui::Color32::RED, e); }
            meter(ui, "input rms", m.input_rms, 1.0);
            ui.separator();
            meter(ui, "low",  m.band_levels[0], 2.0);
            meter(ui, "mid",  m.band_levels[1], 2.0);
            meter(ui, "high", m.band_levels[2], 2.0);
            ui.separator();
            ui.label(format!("kick  gate {:.0} trig {:.0}", m.kick.0, m.kick.1));
            ui.label(format!("snare gate {:.0} trig {:.0}", m.snare.0, m.snare.1));
            ui.label(format!("rythm trig {:.0}  flux {:.4}", m.rythm.1, m.flux));
            ui.separator();
            meter(ui, "centroid", m.centroid, 1.0);
            meter(ui, "fms", m.fms, 1.0);
            meter(ui, "sms", m.sms, 1.0);
            ui.separator();
            ui.label("спектр");
            if !m.spectrum.is_empty() {
                let pts: PlotPoints = m.spectrum.iter().enumerate().map(|(i, v)| [i as f64, *v as f64]).collect();
                Plot::new("spec").height(120.0).show(ui, |p| p.line(Line::new(pts)));
            }
            ui.separator();
            ui.label(format!("control: L{:.2} M{:.2} H{:.2}", m.control.low_gain, m.control.mid_gain, m.control.high_gain));
            ui.label(format!("thr: kick {:.3} snare {:.3} rythm {:.3}", m.control.kick_thresh, m.control.snare_thresh, m.control.rythm_thresh));
            ui.label(format!("lag: {:.4}", m.control.lag_value));
        });

        // ─── Верхняя панель: большие лампы сигналов (по огибающим — импульсы видны) ───
        egui::TopBottomPanel::top("lamps").show(ctx, |ui| {
            let m = shared.metrics.lock().unwrap().clone();
            ui.horizontal(|ui| {
                lamp(ui, "KICK", m.kick_env, egui::Color32::from_rgb(255, 80, 60));
                lamp(ui, "SNARE", m.snare_env, egui::Color32::from_rgb(80, 160, 255));
                lamp(ui, "RYTHM", m.rythm_env, egui::Color32::from_rgb(120, 255, 120));
                ui.separator();
                lamp(ui, "LOW", (m.band_levels[0] / 1.5).clamp(0.0, 1.0), egui::Color32::from_rgb(255, 180, 60));
                lamp(ui, "MID", (m.band_levels[1] / 1.5).clamp(0.0, 1.0), egui::Color32::from_rgb(255, 230, 90));
                lamp(ui, "HIGH", (m.band_levels[2] / 1.5).clamp(0.0, 1.0), egui::Color32::from_rgb(200, 120, 255));
            });
        });

        // ─── Центр: полосы / детекторы / адаптив ───
        let m = shared.metrics.lock().unwrap().clone();
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut cfg = shared.config.lock().unwrap();
            let control_on = cfg.control.enabled;
            egui::ScrollArea::vertical().show(ui, |ui| {
                // ── Полосы: side by side ──
                ui.heading("Полосы");
                if control_on {
                    ui.weak("gain полос управляется адаптивом → крутилка gain неактивна");
                }
                let bands = &mut cfg.bands;
                ui.columns(bands.len(), |cols| {
                    for (i, b) in bands.iter_mut().enumerate() {
                        let ui = &mut cols[i];
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut b.active, "").on_hover_text("вкл/выкл полосу");
                                ui.strong(&b.name);
                            });
                            ui.add(egui::DragValue::new(&mut b.cutoff_hz).speed(1.0).suffix(" Hz"))
                                .on_hover_text("частота среза фильтра полосы");
                            ui.add(egui::DragValue::new(&mut b.rolloff_db_oct).speed(0.5).prefix("roll "))
                                .on_hover_text("крутизна спада фильтра, дБ/окт");
                            ui.add(egui::DragValue::new(&mut b.resonance).speed(0.01).prefix("Q "))
                                .on_hover_text("добротность (резонанс) фильтра");
                            ui.add(egui::Slider::new(&mut b.pregain, 0.0..=8.0).text("pregain"))
                                .on_hover_text("усиление ДО измерения RMS полосы");
                            ui.add(egui::Slider::new(&mut b.threshold, 0.0..=1.0).text("thr"))
                                .on_hover_text("порог отсечки уровня полосы");
                            ui.add_enabled(!control_on, egui::Slider::new(&mut b.gain, 0.0..=10.0).text("gain"))
                                .on_hover_text("ручной выходной гейн (неактивен при адаптиве)");
                            ui.add(egui::Slider::new(&mut b.add, -1.0..=1.0).text("add"))
                                .on_hover_text("смещение уровня (add)");
                            ui.add(egui::Slider::new(&mut b.smooth_s, 0.0..=1.0).text("smooth s"))
                                .on_hover_text("окно сглаживания уровня, сек");
                        });
                    }
                });

                ui.separator();
                ui.heading("Детекторы");
                // kick/snare пороги при адаптиве правятся сигмоидами → неактивны; rythm всегда ручной.
                for (idx, d) in cfg.detectors.iter_mut().enumerate() {
                    let manual = !control_on || idx == 2;
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut d.active, "").on_hover_text("вкл/выкл детектор");
                        ui.strong(&d.name);
                        ui.add_enabled(manual, egui::DragValue::new(&mut d.threshold).speed(0.01).prefix("thr "))
                            .on_hover_text(if idx == 2 {
                                "порог onset по нормированному flux (0..1)"
                            } else {
                                "порог; при адаптиве задаётся сигмоидой (неактивен)"
                            });
                        ui.add(egui::DragValue::new(&mut d.retrigger_s).speed(0.005).prefix("retrig "))
                            .on_hover_text("минимальная пауза между срабатываниями, сек");
                    });
                }

                ui.separator();
                ui.heading("Адаптивное управление");
                let c = &mut cfg.control;
                ui.checkbox(&mut c.enabled, "включено (правит гейны полос и пороги kick/snare)")
                    .on_hover_text("RMS входа → lag → мапперы/сигмоиды → гейны и пороги");
                ui.checkbox(&mut c.control_rms, "RMS в control-ветви (R3)")
                    .on_hover_text("считать RMS сигнала control-ветви перед мапперами");
                gain_ui(ui, "corr gain (math2)", &mut c.corr_gain);
                ui.collapsing("lag (stateful, без bypass — R3)", |ui| {
                    ui.label("асимметричное сглаживание с ограничением скорости; помнит прошлые значения");
                    ui.add(egui::Slider::new(&mut c.lag.lag_up, 0.0..=10.0).text("lag up"))
                        .on_hover_text("сглаживание на росте (больше → медленнее вверх)");
                    ui.add(egui::Slider::new(&mut c.lag.lag_dn, 0.0..=10.0).text("lag dn"))
                        .on_hover_text("сглаживание на спаде (больше → медленнее вниз)");
                    ui.add(egui::Slider::new(&mut c.lag.accel_up, 0.1..=10.0).text("accel up"))
                        .on_hover_text("предел ускорения вверх");
                    ui.add(egui::Slider::new(&mut c.lag.accel_dn, 0.1..=10.0).text("accel dn"))
                        .on_hover_text("предел ускорения вниз");
                });
                ui.collapsing("мапперы гейнов", |ui| {
                    gain_ui(ui, "low",  &mut c.low_gain);
                    gain_ui(ui, "mid",  &mut c.mid_gain);
                    gain_ui(ui, "high (highControlGain1)", &mut c.high_gain);
                    ui.checkbox(&mut c.use_high_alt, "использовать highControlGain (альт, R1)");
                    gain_ui(ui, "high alt", &mut c.high_gain_alt);
                });
                ui.collapsing("мапперы порогов + сигмоиды (R4)", |ui| {
                    ui.label("маппер даёт вход x, сигмоида → порог. На графике: голубая точка — x (пришло), жёлтая — порог (ушло).");
                    gain_ui(ui, "kick map",  &mut c.kick_map);
                    sigmoid_ctrl(ui, "kick sigmoid", &mut c.kick_sigmoid, Some(m.control.kick_x), &mut self.sig_win[0]);
                    gain_ui(ui, "snare map", &mut c.snare_map);
                    sigmoid_ctrl(ui, "snare sigmoid", &mut c.snare_sigmoid, Some(m.control.snare_x), &mut self.sig_win[1]);
                    gain_ui(ui, "rythm map", &mut c.rythm_map);
                    sigmoid_ctrl(ui, "rythm sigmoid", &mut c.rythm_sigmoid, Some(m.control.rythm_x), &mut self.sig_win[2]);
                });
            });

            // ── Всплывающие окна сигмоид с увеличенным графиком ──
            let ctrl = &mut cfg.control;
            let sigs: [(&str, &mut SigmoidCfg, f32); 3] = [
                ("kick sigmoid", &mut ctrl.kick_sigmoid, m.control.kick_x),
                ("snare sigmoid", &mut ctrl.snare_sigmoid, m.control.snare_x),
                ("rythm sigmoid", &mut ctrl.rythm_sigmoid, m.control.rythm_x),
            ];
            for (i, (label, s, live_x)) in sigs.into_iter().enumerate() {
                let mut open = self.sig_win[i];
                egui::Window::new(label)
                    .open(&mut open)
                    .default_size([420.0, 380.0])
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut s.enabled, "сигмоида вкл");
                        });
                        sigmoid_params(ui, s);
                        let h = (ui.available_height() - 10.0).max(120.0);
                        sigmoid_plot(ui, &format!("win_{label}"), s, Some(live_x), h);
                    });
                self.sig_win[i] = open;
            }
        });
    }
}
