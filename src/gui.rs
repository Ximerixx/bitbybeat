//! GUI на egui: тумблеры/крутилки всего тракта + метры/спектр (md_plans/10 R0/R5).

use crate::audio;
use crate::config::{OscTransport, Source};
use crate::diag::{self, format_time, LogLevel};
use crate::osc_map::OSC_CHANNEL_LIST;
use crate::preset::{autosave_path, AudioInputKey, UndoStack};
use crate::probe::{self, ProbeHistory, ProbeId, ProbeUi};
use crate::shared::{Metrics, Shared};
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

enum CloseDialog {
    Ask,
}

pub struct App {
    shared: Arc<Shared>,
    cfg_edit: crate::config::Config,
    /// Последний сохранённый на диск (или загруженный) пресет.
    saved_baseline: crate::config::Config,
    config_dirty: bool,
    audio_applied: AudioInputKey,
    undo: UndoStack,
    devices: Vec<audio::DeviceInfo>,
    pulse_sources: Vec<audio::PulseSource>,
    preset_path: String,
    status: String,
    sig_win: [bool; 3],
    show_console: bool,
    log_debug: bool,
    log_info: bool,
    log_warning: bool,
    log_error: bool,
    osc_error_dialog: Option<String>,
    close_dialog: Option<CloseDialog>,
    /// Разрешить закрыть окно даже с несохранённым пресетом.
    allow_exit_dirty: bool,
    autosave_timer: Instant,
    display_metrics: Metrics,
    spectrum_plot: Vec<[f64; 2]>,
    spectrum_plot_frame: u64,
    show_osc_channels: bool,
    show_poster: bool,
    probe: Option<ProbeId>,
    probe_entered: bool,
    probe_hist: ProbeHistory,
}

impl App {
    pub fn new(shared: Arc<Shared>) -> Self {
        let cfg_edit = shared.config.load().as_ref().clone();
        let saved_baseline = cfg_edit.clone();
        let audio_applied = AudioInputKey::from(&cfg_edit.input);
        Self {
            shared,
            cfg_edit,
            saved_baseline,
            config_dirty: false,
            audio_applied,
            undo: UndoStack::new(),
            devices: audio::list_input_devices(),
            pulse_sources: audio::list_pulse_sources(),
            preset_path: "preset.ron".into(),
            status: String::new(),
            sig_win: [false; 3],
            show_console: false,
            log_debug: false,
            log_info: true,
            log_warning: true,
            log_error: true,
            osc_error_dialog: None,
            close_dialog: None,
            allow_exit_dirty: false,
            autosave_timer: Instant::now(),
            display_metrics: Metrics::default(),
            spectrum_plot: Vec::new(),
            spectrum_plot_frame: 0,
            show_osc_channels: false,
            show_poster: false,
            probe: None,
            probe_entered: false,
            probe_hist: ProbeHistory::new(),
        }
    }

    fn preset_dirty(&self) -> bool {
        self.cfg_edit != self.saved_baseline
    }

    fn audio_pending(&self) -> bool {
        AudioInputKey::from(&self.cfg_edit.input) != self.audio_applied
    }

    fn mark_dirty(&mut self) {
        self.config_dirty = true;
    }

    fn commit_config(&mut self) {
        if self.config_dirty {
            self.shared.config.store(self.cfg_edit.clone());
            let v = self.shared.config.version();
            diag::debug("app", format!("config committed v{v}"));
            self.config_dirty = false;
        }
    }

    fn apply_audio_restart(&mut self) {
        self.audio_applied = AudioInputKey::from(&self.cfg_edit.input);
        self.shared.restart_audio.store(true, Ordering::Release);
        self.mark_dirty();
    }

    fn mark_saved_baseline(&mut self) {
        self.saved_baseline = self.cfg_edit.clone();
    }

    fn save_preset_file(&mut self) -> String {
        self.cfg_edit.osc.normalize_known(OSC_CHANNEL_LIST);
        match self.cfg_edit.save_ron(&self.preset_path) {
            Ok(_) => {
                self.mark_saved_baseline();
                format!("сохранено -> {}", self.preset_path)
            }
            Err(e) => format!("ошибка записи: {e}"),
        }
    }

    fn try_autosave(&mut self, dt: f32) {
        if !self.preset_dirty() {
            return;
        }
        if self.autosave_timer.elapsed().as_secs_f32() < 45.0 {
            return;
        }
        let path = autosave_path(&self.preset_path);
        self.cfg_edit.osc.normalize_known(OSC_CHANNEL_LIST);
        if let Err(e) = self.cfg_edit.save_ron(path.to_string_lossy().as_ref()) {
            diag::warn("app", format!("autosave: {e}"));
        } else {
            diag::info("app", format!("autosave -> {}", path.display()));
            self.autosave_timer = Instant::now();
        }
        let _ = dt;
    }

    fn handle_undo_input(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.pointer.any_pressed()) {
            self.undo.on_pointer_pressed(&self.cfg_edit);
        }
        if ctx.input(|i| i.pointer.any_released()) {
            if self.undo.on_pointer_released(&self.cfg_edit) {
                self.mark_dirty();
            }
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
            if let Some(prev) = self.undo.undo(&self.cfg_edit) {
                self.cfg_edit = prev;
                self.mark_dirty();
            }
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Y)) {
            if let Some(next) = self.undo.redo(&self.cfg_edit) {
                self.cfg_edit = next;
                self.mark_dirty();
            }
        }
    }
}

/// Большая "лампа" - квадрат, светящийся по значению 0..1 (md_plans/10: индикаторы kick/snare/rythm).
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

fn meter(ui: &mut egui::Ui, label: &str, v: f32, max: f32) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}: {v:6.3}"));
        let frac = (v / max).clamp(0.0, 1.0);
        ui.add(egui::ProgressBar::new(frac).desired_width(160.0));
    });
}

/// Значение на входе детектора + порог, с которым оно сравнивается.
fn meter_detect(ui: &mut egui::Ui, label: &str, v: f32, thr: f32, max: f32) {
    ui.horizontal(|ui| {
        let over = v > thr;
        ui.label(format!("{label}: {v:6.3}"));
        let frac = if max > 1e-9 { (v / max).clamp(0.0, 1.0) } else { 0.0 };
        ui.add(egui::ProgressBar::new(frac).desired_width(120.0));
        let col = if over {
            egui::Color32::from_rgb(120, 255, 120)
        } else {
            egui::Color32::GRAY
        };
        ui.colored_label(col, format!("порог {thr:.3}"));
    });
}

/// Gain полосы: при адаптиве показываем базу (серую) и живое значение.
fn band_gain_ui(ui: &mut egui::Ui, manual: &mut f32, live: f32, adaptive_on: bool) {
    ui.horizontal(|ui| {
        if adaptive_on {
            let base = *manual;
            ui.add_enabled(false, egui::Slider::new(manual, 0.0..=10.0).text(format!("база {base:.2}")))
                .on_hover_text("значение из пресета. Сейчас его подменяет адаптив");
            ui.label("->");
            ui.colored_label(egui::Color32::from_rgb(255, 210, 80), format!("{live:.2}"));
            let frac = (live / 10.0).clamp(0.0, 1.0);
            ui.add(
                egui::ProgressBar::new(frac)
                    .fill(egui::Color32::from_rgb(255, 180, 60))
                    .desired_width(72.0),
            );
        } else {
            ui.add(egui::Slider::new(manual, 0.0..=10.0).text("крутизна после порога"))
                .on_hover_text("после вычитания порога: больше - полоса громче в OSC и легче срабатывает детектор");
        }
    });
}

fn latency_budget_ui(
    ui: &mut egui::Ui,
    ring: f32,
    compute_ms: f32,
    osc_jitter_ms: f32,
    osc_send_latency_ms: f32,
) {
    ui.label(format!(
        "latency: ring {:.0}%  compute {:.1}ms  OSC jitter {:.1}ms  send lag {:.1}ms",
        ring * 100.0,
        compute_ms,
        osc_jitter_ms,
        osc_send_latency_ms,
    ));
    ui.horizontal(|ui| {
        let ring_c = if ring > 0.7 {
            egui::Color32::RED
        } else if ring > 0.4 {
            egui::Color32::YELLOW
        } else {
            egui::Color32::GREEN
        };
        ui.colored_label(ring_c, "* ring");
        let cmp_c = if compute_ms > 12.0 {
            egui::Color32::RED
        } else if compute_ms > 8.0 {
            egui::Color32::YELLOW
        } else {
            egui::Color32::GREEN
        };
        ui.colored_label(cmp_c, "* compute");
        let jit_c = if osc_jitter_ms > 3.0 {
            egui::Color32::RED
        } else if osc_jitter_ms > 1.0 {
            egui::Color32::YELLOW
        } else {
            egui::Color32::GREEN
        };
        ui.colored_label(jit_c, "* osc");
    });
}

fn restart_button(ui: &mut egui::Ui, pending: bool) -> egui::Response {
    let t = ui.ctx().input(|i| i.time);
    let blink_on = pending && ((t * 4.0).sin() > 0.0);
    let mut btn = egui::Button::new("применить (restart)");
    if pending {
        btn = btn.fill(if blink_on {
            egui::Color32::from_rgb(255, 200, 50)
        } else {
            egui::Color32::from_rgb(220, 70, 50)
        });
    }
    ui.add(btn).on_hover_text(if pending {
        "источник изменён - нажми, чтобы переподключить аудио"
    } else {
        "переподключить аудио с текущими настройками"
    })
}

fn console_window(app: &mut App, ctx: &egui::Context) {
    let mut open = app.show_console;
    egui::Window::new("Консоль")
        .open(&mut open)
        .default_size([720.0, 400.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Фильтр:");
                if ui.checkbox(&mut app.log_debug, "debug").changed() {}
                if ui.checkbox(&mut app.log_info, "info").changed() {}
                if ui.checkbox(&mut app.log_warning, "warning").changed() {}
                if ui.checkbox(&mut app.log_error, "error").changed() {}
                if ui.button("очистить").clicked() {
                    app.shared.logs.clear();
                    diag::info("app", "консоль очищена");
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                for e in app.shared.logs.snapshot() {
                    let pass = match e.level {
                        LogLevel::Debug => app.log_debug,
                        LogLevel::Info => app.log_info,
                        LogLevel::Warning => app.log_warning,
                        LogLevel::Error => app.log_error,
                    };
                    if !pass {
                        continue;
                    }
                    ui.horizontal(|ui| {
                        ui.monospace(format_time(e.at));
                    let (r, g, b) = e.level.color_rgb();
                        ui.colored_label(egui::Color32::from_rgb(r, g, b), e.level.label());
                        ui.label(format!("[{}]", e.target));
                        ui.label(&e.message);
                    });
                }
            });
        });
    app.show_console = open;
}

fn osc_channels_window(app: &mut App, ctx: &egui::Context) {
    let mut open = app.show_osc_channels;
    let mut dirty = false;
    let mut do_save = false;
    let enabled_n = OSC_CHANNEL_LIST
        .iter()
        .filter(|(addr, _)| app.cfg_edit.osc.sends(addr))
        .count();
    egui::Window::new("OSC-каналы")
        .open(&mut open)
        .default_size([420.0, 560.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(format!(
                "Включено {enabled_n} из {}. Выкл - адрес не уходит в QLC.",
                OSC_CHANNEL_LIST.len()
            ));
            ui.weak(format!(
                "В файл попадает по кнопке сохранить пресет -> {}. Автосейв ({}) при старте не читается.",
                app.preset_path,
                autosave_path(&app.preset_path).display()
            ));
            ui.horizontal(|ui| {
                if ui.button("все вкл").clicked() {
                    for (addr, _) in OSC_CHANNEL_LIST {
                        app.cfg_edit.osc.set_sends(addr, true);
                    }
                    dirty = true;
                }
                if ui.button("все выкл").clicked() {
                    for (addr, _) in OSC_CHANNEL_LIST {
                        app.cfg_edit.osc.set_sends(addr, false);
                    }
                    dirty = true;
                }
                if ui.button("сохранить пресет").clicked() {
                    do_save = true;
                    dirty = true;
                }
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (addr, hint) in OSC_CHANNEL_LIST {
                    let mut on = app.cfg_edit.osc.sends(addr);
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut on, format!("/{addr}")).changed() {
                            app.cfg_edit.osc.set_sends(addr, on);
                            dirty = true;
                        }
                        ui.weak(*hint);
                    });
                }
            });
        });
    app.show_osc_channels = open;
    if do_save {
        app.status = app.save_preset_file();
    }
    if dirty {
        app.mark_dirty();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.preset_dirty() && !self.allow_exit_dirty {
                self.close_dialog = Some(CloseDialog::Ask);
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            }
        }

        ctx.request_repaint();
        let shared = self.shared.clone();
        self.handle_undo_input(ctx);
        self.try_autosave(ctx.input(|i| i.stable_dt));

        if let Some(msg) = shared.logs.take_osc_error_dialog() {
            self.osc_error_dialog = Some(msg);
        }

        if let Some(msg) = self.osc_error_dialog.clone() {
            let mut open = true;
            egui::Window::new("Ошибка OSC")
                .open(&mut open)
                .collapsible(false)
                .resizable(true)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, &msg);
                    ui.horizontal(|ui| {
                        if ui.button("OK").clicked() {
                            self.osc_error_dialog = None;
                        }
                        if ui.button("Консоль").clicked() {
                            self.show_console = true;
                            self.osc_error_dialog = None;
                        }
                    });
                });
            if !open {
                self.osc_error_dialog = None;
            }
        }

        if matches!(self.close_dialog, Some(CloseDialog::Ask)) {
            let mut open = true;
            egui::Window::new("Сохранить пресет?")
                .open(&mut open)
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Есть несохранённые изменения. Сохранить перед выходом?");
                    ui.horizontal(|ui| {
                        if ui.button("Сохранить").clicked() {
                            let msg = self.save_preset_file();
                            if msg.starts_with("сохранено") {
                                self.status = msg;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            } else {
                                self.status = msg;
                            }
                            self.close_dialog = None;
                        }
                        if ui.button("Выйти без сохранения").clicked() {
                            self.allow_exit_dirty = true;
                            self.close_dialog = None;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Отмена").clicked() {
                            self.close_dialog = None;
                        }
                    });
                });
            if !open {
                self.close_dialog = None;
            }
        }

        shared.metrics.copy_latest(&mut self.display_metrics);
        if self.display_metrics.compute_frame_id != self.spectrum_plot_frame {
            self.spectrum_plot = spectrum_plot_points(
                &self.display_metrics.spectrum,
                self.display_metrics.spectrum_len,
            );
            self.spectrum_plot_frame = self.display_metrics.compute_frame_id;
        }
        let m = self.display_metrics.clone();
        self.probe_hist.push(&self.display_metrics);
        let mut probe_hit: Option<ProbeId> = None;
        let preset_dirty = self.preset_dirty();

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Консоль").clicked() {
                    self.show_console = true;
                }
                if ui.button("OSC каналы").clicked() {
                    self.show_osc_channels = true;
                }
                if ui.button("Схема").clicked() {
                    self.show_poster = true;
                }
                ui.separator();
                if ui.add_enabled(self.undo.can_undo(), egui::Button::new("undo")).clicked() {
                    if let Some(prev) = self.undo.undo(&self.cfg_edit) {
                        self.cfg_edit = prev;
                        self.mark_dirty();
                    }
                }
                if ui.add_enabled(self.undo.can_redo(), egui::Button::new("redo")).clicked() {
                    if let Some(next) = self.undo.redo(&self.cfg_edit) {
                        self.cfg_edit = next;
                        self.mark_dirty();
                    }
                }
                ui.separator();
                if preset_dirty {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 200, 80),
                        "* пресет изменён (не сохранён)",
                    );
                }
                ui.separator();
                latency_budget_ui(
                    ui,
                    m.ringbuf_fill,
                    m.compute_dt_ms,
                    m.osc_jitter_ms,
                    m.osc_send_latency_ms,
                );
                ui.separator();
                ui.label(format!(
                    "frame {}  bundleSeq {}  phase {:.2}  OSC ok/err {}/{}",
                    m.compute_frame_id, m.osc_bundle_seq, m.beat_phase, m.osc_send_ok, m.osc_send_err
                ));
                if let Some(e) = &m.osc_last_error {
                    ui.colored_label(egui::Color32::RED, format!("OSC: {e}"));
                }
            });
        });

        if self.show_console {
            console_window(self, ctx);
        }
        if self.show_osc_channels {
            osc_channels_window(self, ctx);
        }

        // ─── Левая панель: вход + пресеты ───
        egui::SidePanel::left("left").resizable(true).default_width(320.0).show(ctx, |ui| {
            ui.heading("Вход / Источник");
            let mut panel_dirty = false;
            let mut do_restart = false;
            let mut open_osc_channels = false;
            let audio_pending = self.audio_pending();
            {
            let cfg = &mut self.cfg_edit;

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
                    panel_dirty = true;
                }
            }

            // PulseAudio-источники
            let cur_pulse = match &cfg.input.pulse_source {
                Some(n) => self.pulse_sources.iter().find(|s| &s.name == n)
                    .map(|s| s.label().to_string()).unwrap_or_else(|| n.clone()),
                None => "- нет (ALSA-устройство) -".into(),
            };
            egui::ComboBox::from_label("pulse source (monitor)")
                .selected_text(cur_pulse)
                .width(280.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut cfg.input.pulse_source, None, "- нет (ALSA-устройство) -");
                    for s in &self.pulse_sources {
                        let tag = if s.is_monitor { format!("[mon] {}", s.label()) } else { s.label().to_string() };
                        ui.selectable_value(&mut cfg.input.pulse_source, Some(s.name.clone()), tag);
                    }
                })
                .response
                .on_hover_text("захват системного звука: выберите [mon] monitor вашего выхода (через parec, без паник cpal)");

            let cur = cfg.input.device.clone().unwrap_or_else(|| "- default -".into());
            ui.add_enabled_ui(cfg.input.pulse_source.is_none(), |ui| {
                egui::ComboBox::from_label("ALSA device")
                    .selected_text(cur)
                    .width(280.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut cfg.input.device, None, "- default -");
                        for d in &self.devices {
                            let base = format!("{} [{}ch]", d.name, d.channels);
                            let tag = if audio::is_monitor(&d.name) { format!("[mon] {base}") } else { base };
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
                    ui.label(format!("каналы устройства: {nch} - выбери нужные (1-2)"));
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
                if restart_button(ui, audio_pending).clicked() {
                    do_restart = true;
                    panel_dirty = true;
                }
            });

            ui.separator();
            ui.heading("Пре-обработка");
            ui.weak("Цепочка: устройство -> моно -> (компрессор) -> полосы / спектр / адаптив.");
            ui.checkbox(&mut cfg.compressor.enabled, "компрессор на входе")
                .on_hover_text("сжимает громкость всего сигнала до фильтров полос; выкл - сырой звук в анализ");
            if cfg.compressor.enabled {
                let c = &mut cfg.compressor.cfg;
                ui.add(egui::Slider::new(&mut c.threshold_db, -60.0..=0.0).text("порог, дБ"))
                    .on_hover_text("тише порога не трогаем; громче - сжимаем");
                ui.add(egui::Slider::new(&mut c.ratio, 0.05..=4.0).text("ratio"))
                    .on_hover_text(
                        "как в дампе audiodyna, не классический 4:1.\n\
                         1 = почти не трогает. Больше 1 - давит пики.\n\
                         Меньше 1 - пики раздувает (формула 1/ratio - 1).",
                    );
                if c.ratio < 1.0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 160, 60),
                        format!("ratio {:.2} < 1: пики громче, не тише", c.ratio),
                    );
                }
                ui.add(egui::Slider::new(&mut c.makeup_db, -12.0..=24.0).text("компенсация, дБ"))
                    .on_hover_text("громкость после сжатия, чтобы уровень не просел");
            }
            ui.checkbox(&mut cfg.dsp_rmspower, "считать /dsprms")
                .on_hover_text("отдельный канал OSC: RMS всего кадра после компрессора. На полосы и детекторы не влияет");
            if probe::mapper_ui(
                ui,
                "Громкость для /dsprms",
                "вход: RMS всего кадра -> этот маппер -> OSC /dsprms. Полосы, kick/snare и адаптив сюда не смотрят.",
                &mut cfg.dsp_gain,
                ProbeId::DspRms,
                &mut probe_hit,
            ) {
                panel_dirty = true;
            }

            ui.separator();
            ui.heading("Частоты");
            ui.add(egui::Slider::new(&mut cfg.compute_rate_hz, 30.0..=480.0).text("анализ, Гц"))
                .on_hover_text("как часто считаем полосы и детекторы. Выше - быстрее отклик ламп и hold");
            ui.add(egui::Slider::new(&mut cfg.osc_rate_hz, 1.0..=480.0).text("отправка OSC, Гц"))
                .on_hover_text("как часто шлём последний снимок в QLC. Не обязана совпадать с анализом");
            if probe::spectral_ui(ui, cfg, &m, &mut probe_hit) {
                panel_dirty = true;
            }

            ui.separator();
            ui.heading("OSC");
            if ui.checkbox(&mut cfg.osc.enabled, "включён").changed() {
                panel_dirty = true;
            }
            ui.horizontal(|ui| {
                ui.label("транспорт");
                if ui.selectable_label(cfg.osc.transport == OscTransport::Udp, "UDP").clicked() {
                    cfg.osc.transport = OscTransport::Udp;
                    panel_dirty = true;
                }
                if ui.selectable_label(cfg.osc.transport == OscTransport::Tcp, "TCP").clicked() {
                    cfg.osc.transport = OscTransport::Tcp;
                    panel_dirty = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("host");
                if ui.text_edit_singleline(&mut cfg.osc.host).changed() {
                    panel_dirty = true;
                }
            });
            if ui.add(egui::DragValue::new(&mut cfg.osc.port).prefix("port ")).changed() {
                panel_dirty = true;
            }
            if ui.checkbox(&mut cfg.osc.bundle, "bundle").changed() {
                panel_dirty = true;
            }
            if ui.checkbox(&mut cfg.osc.bundle_meta, "meta: bundleSeq / bundleTime / bundleFrame")
                .on_hover_text("счётчик и таймстамп в каждом bundle - приёмник выбирает новее по bundleSeq")
                .changed()
            {
                panel_dirty = true;
            }
            if ui.checkbox(&mut cfg.osc.clip_levels_at_zero, "low/mid/high >= 0 на OSC")
                .on_hover_text("при отправке: отрицательные low/mid/high обрезаются до 0; внутри приложения значения без изменений")
                .changed()
            {
                panel_dirty = true;
            }
            if ui.button("какие каналы слать...").clicked() {
                open_osc_channels = true;
            }
            ui.collapsing("фаза / синхронизация", |ui| {
                let p = &mut cfg.osc.phase;
                if ui.checkbox(&mut p.sync_timeline, "общая временная шкала (anti-drift)")
                    .on_hover_text("compute и OSC якорят sleep к одному origin")
                    .changed()
                {
                    panel_dirty = true;
                }
                if ui.checkbox(&mut p.quantize_triggers, "квантовать триггеры к фазе")
                    .on_hover_text(
                        "Импульсы (kick/snare/trigger*) шлются на ближайшей отметке сетки фазы,\n\
                         а не в момент детекта. Снижает разброс относительно такта,\n\
                         но добавляет задержку до 1 шага сетки. Выкл - минимальная задержка, больше джиттер.",
                    )
                    .changed()
                {
                    panel_dirty = true;
                }
                if ui.add(egui::Slider::new(&mut p.phase_grid, 0.0625..=1.0).text("сетка фазы"))
                    .changed()
                {
                    panel_dirty = true;
                }
                if ui.checkbox(&mut p.immediate_triggers, "немедленные импульсы (без очереди)")
                    .changed()
                {
                    panel_dirty = true;
                }
            });

            }
            if open_osc_channels {
                self.show_osc_channels = true;
            }
            if do_restart {
                self.apply_audio_restart();
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.preset_path);
            });
            ui.horizontal(|ui| {
                if ui.button("сохранить пресет").clicked() {
                    self.status = self.save_preset_file();
                }
                if ui.button("загрузить").clicked() {
                    match crate::config::Config::load_ron(&self.preset_path) {
                        Ok(new) => {
                            self.cfg_edit = new;
                            self.mark_saved_baseline();
                            self.undo.clear();
                            self.status = "загружено".into();
                            self.apply_audio_restart();
                            panel_dirty = true;
                        }
                        Err(e) => self.status = format!("ошибка: {e}"),
                    }
                }
            });
            if preset_dirty {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 200, 80),
                    "не сохранено в файл пресета (autosave - запасная копия, при старте не грузится)",
                );
                ui.weak(format!("копия: {}", autosave_path(&self.preset_path).display()));
            }
            ui.label(&self.status);
            if panel_dirty {
                self.config_dirty = true;
            }
        });

        // ─── Правая панель: метры/спектр ───
        egui::SidePanel::right("right").resizable(true).default_width(340.0).show(ctx, |ui| {
            ui.heading("Метры");
            ui.label(format!("устройство: {}", m.device_name));
            ui.label(format!("sr: {:.0} Гц   OSC-каналов: {}", m.sample_rate, m.osc_channels));
            ui.label(format!("beat phase: {:.3}", m.beat_phase));
            if m.kick_bar_pos > 0 || m.snare_bar_pos > 0 {
                ui.label(format!(
                    "такт: kick {}/4  snare {}/4",
                    m.kick_bar_pos, m.snare_bar_pos
                ));
            }
            if let Some(e) = &m.error { ui.colored_label(egui::Color32::RED, e); }
            if let Some(e) = &m.osc_last_error { ui.colored_label(egui::Color32::RED, format!("OSC: {e}")); }
            meter(ui, "input rms", m.input_rms, 1.0);
            ui.separator();
            meter(ui, "low",  m.band_levels[0], 2.0);
            meter(ui, "mid",  m.band_levels[1], 2.0);
            meter(ui, "high", m.band_levels[2], 2.0);
            ui.weak("OSC /low /mid /high (после gain, add, сглаживания)");
            ui.separator();
            ui.weak("вход детекторов: с этим сравнивается порог. kick=low*pregain, snare=high*pregain, rythm=flux 0..1");
            meter_detect(ui, "kick in", m.detect[0], m.detect_thr[0], 1.0);
            meter_detect(ui, "snare in", m.detect[1], m.detect_thr[1], 2.0);
            meter_detect(ui, "rythm in", m.detect[2], m.detect_thr[2], 1.0);
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
            if m.spectrum_len > 0 {
                let pts = PlotPoints::new(self.spectrum_plot.clone());
                Plot::new("spec").height(120.0).show(ui, |p| p.line(Line::new(pts)));
            }
            ui.separator();
            ui.label(format!("control: L{:.2} M{:.2} H{:.2}", m.control.low_gain, m.control.mid_gain, m.control.high_gain));
            ui.label(format!("thr: kick {:.3} snare {:.3} rythm {:.3}", m.control.kick_thresh, m.control.snare_thresh, m.control.rythm_thresh));
            ui.label(format!("lag: {:.4}", m.control.lag_value));
        });

        // ─── Верхняя панель: большие лампы сигналов ───
        egui::TopBottomPanel::top("lamps").show(ctx, |ui| {
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
        egui::CentralPanel::default().show(ctx, |ui| {
            let control_on = self.cfg_edit.control.enabled;
            egui::ScrollArea::vertical().show(ui, |ui| {
                // ── Полосы: side by side ──
                ui.heading("Полосы");
                ui.weak("После компрессора звук режется на 3 фильтра. Дальше: RMS -> pregain -> порог -> gain -> add -> сглаживание -> OSC /low /mid /high и детекторы.");
                if control_on {
                    ui.weak("Адаптив включён: gain полосы берётся из громкости зала, крутилка ниже только база в пресете.");
                }
                let bands = &mut self.cfg_edit.bands;
                let live_gains = [m.control.low_gain, m.control.mid_gain, m.control.high_gain];
                ui.columns(bands.len(), |cols| {
                    for (i, b) in bands.iter_mut().enumerate() {
                        let ui = &mut cols[i];
                        let band_r = ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut b.active, "").on_hover_text("выкл - полоса даёт 0, детектор на ней молчит");
                                ui.strong(&b.name);
                                probe::lupa_button(ui, ProbeId::Band(i as u8), &mut probe_hit);
                            });
                            ui.add(egui::DragValue::new(&mut b.cutoff_hz).speed(1.0).suffix(" Hz"))
                                .on_hover_text("частота среза фильтра этой полосы");
                            ui.add(egui::DragValue::new(&mut b.rolloff_db_oct).speed(0.5).prefix("спад "))
                                .on_hover_text("крутизна фильтра, дБ/октава: круче - меньше соседних частот");
                            ui.add(egui::DragValue::new(&mut b.resonance).speed(0.01).prefix("Q "))
                                .on_hover_text("горбик на частоте среза");
                            ui.add(egui::Slider::new(&mut b.pregain, 0.0..=8.0).text("до порога x"))
                                .on_hover_text("умножает RMS полосы до вычитания порога. Mid/high обычно выше, чтобы слабый RMS дотягивал");
                            ui.add(egui::Slider::new(&mut b.threshold, 0.0..=1.0).text("порог тишины"))
                                .on_hover_text("вычитается из RMS. Ниже порога уровень полосы = 0");
                            band_gain_ui(ui, &mut b.gain, live_gains[i], control_on);
                            ui.add(egui::Slider::new(&mut b.add, -1.0..=1.0).text("сдвиг после"))
                                .on_hover_text("после clamp 0..100, до сглаживания. Отрицательный - /low /mid /high уходят ниже 0. Для QLC есть тумблер clip >= 0.");
                            if b.add < 0.0 {
                                ui.colored_label(
                                    egui::Color32::from_rgb(255, 160, 60),
                                    format!("add {:+.2}: полоса может быть < 0", b.add),
                                );
                            }
                            ui.add(egui::Slider::new(&mut b.smooth_s, 0.0..=1.0).text("сглаживание, с"))
                                .on_hover_text("инерция огибающей. Больше - лампы и OSC менее дёрганые, удар размазан");
                        })
                        .response;
                        probe::open_on_right_click(&band_r, ProbeId::Band(i as u8), &mut probe_hit);
                    }
                });

                ui.separator();
                ui.heading("Детекторы");
                ui.weak("Kick смотрит RMS low, snare - high, rythm - спектральный flux. Выход 0/1 идёт в OSC (/kick /snare /rythm) и в счётчики долей.");
                for (idx, d) in self.cfg_edit.detectors.iter_mut().enumerate() {
                    let manual = !control_on || idx == 2;
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut d.active, "").on_hover_text("выкл - нет импульсов в OSC");
                        ui.strong(&d.name);
                        ui.add_enabled(manual, egui::DragValue::new(&mut d.threshold).speed(0.01).prefix("порог "))
                            .on_hover_text(if idx == 2 {
                                "порог по flux (0..1). Kick/snare при адаптиве задаются кривой ниже"
                            } else {
                                "когда RMS полосы выше - gate. При адаптиве крутилка серая, порог считает кривая"
                            });
                        ui.add(egui::DragValue::new(&mut d.retrigger_s).speed(0.005).prefix("пауза "))
                            .on_hover_text("после импульса столько секунд новый удар игнорируется. 0 - можно дребезжать");
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut d.hysteresis_enabled, "гистерезис")
                            .on_hover_text("gate гаснет не на том же пороге, а ниже на d - меньше дрожи на границе");
                        ui.add_enabled(
                            d.hysteresis_enabled,
                            egui::DragValue::new(&mut d.hysteresis).speed(0.005).prefix("d "),
                        )
                        .on_hover_text("gate выключается при уровне < порог - d");
                        ui.checkbox(&mut d.trigger_hold_enabled, "удержать 1")
                            .on_hover_text("OSC-триггер остаётся 1 после удара, не один кадр");
                        ui.add_enabled(
                            d.trigger_hold_enabled,
                            egui::DragValue::new(&mut d.trigger_hold_s).speed(0.005).prefix("сек "),
                        )
                        .on_hover_text("гасить в 0, если столько секунд не было нового удара");
                    });
                }

                ui.separator();
                ui.heading("Адаптив (подстройка под громкость)");
                ui.weak("Цепочка: RMS всего входа -> масштаб -> инерция -> отсюда гейны полос и пороги kick/snare.");
                let c = &mut self.cfg_edit.control;
                ui.checkbox(&mut c.enabled, "включён")
                    .on_hover_text("подменяет gain полос и пороги kick/snare. Выкл - всё с крутилок полос/детекторов");
                ui.checkbox(&mut c.control_rms, "брать RMS входа")
                    .on_hover_text("вход этой ветки: RMS кадра. Выкл - модуль того же числа (почти то же)");
                let _ = probe::mapper_ui(
                    ui,
                    "Масштаб громкости зала",
                    "вход: RMS после компрессора -> этот маппер -> инерция. Отсюда живёт весь адаптив. На прямой OSC /dsprms не влияет.",
                    &mut c.corr_gain,
                    ProbeId::Corr,
                    &mut probe_hit,
                );
                ui.collapsing("инерция громкости", |ui| {
                    ui.weak("После масштаба, до гейнов полос и порогов. Помнит прошлое: зал не прыгает каждый кадр.");
                    let lag_r = ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong("инерция");
                            probe::lupa_button(ui, ProbeId::Lag, &mut probe_hit);
                        });
                        ui.add(egui::Slider::new(&mut c.lag.lag_up, 0.0..=10.0).text("вверх, медленнее"))
                            .on_hover_text("как быстро растём, когда стало громче");
                        ui.add(egui::Slider::new(&mut c.lag.lag_dn, 0.0..=10.0).text("вниз, медленнее"))
                            .on_hover_text("как быстро падаем, когда стало тише");
                        ui.add(egui::Slider::new(&mut c.lag.accel_up, 0.1..=10.0).text("ускорение вверх"))
                            .on_hover_text("предел рывка вверх");
                        ui.add(egui::Slider::new(&mut c.lag.accel_dn, 0.1..=10.0).text("ускорение вниз"))
                            .on_hover_text("предел рывка вниз");
                    })
                    .response;
                    probe::open_on_right_click(&lag_r, ProbeId::Lag, &mut probe_hit);
                });
                ui.collapsing("гейны полос от громкости", |ui| {
                    ui.weak("После инерции. Результат подставляется как gain low/mid/high вместо крутилки на полосе.");
                    let _ = probe::mapper_ui(ui, "-> gain low", "вход: инерция громкости -> gain полосы low", &mut c.low_gain, ProbeId::GainLow, &mut probe_hit);
                    let _ = probe::mapper_ui(ui, "-> gain mid", "вход: инерция громкости -> gain полосы mid", &mut c.mid_gain, ProbeId::GainMid, &mut probe_hit);
                    let _ = probe::mapper_ui(ui, "-> gain high", "вход: инерция громкости -> gain полосы high", &mut c.high_gain, ProbeId::GainHigh, &mut probe_hit);
                    ui.checkbox(&mut c.use_high_alt, "другой маппер для high")
                        .on_hover_text("вместо основного gain high взять запасной набор крутилок");
                    let _ = probe::mapper_ui(ui, "-> gain high (запасной)", "тот же вход (инерция), другой масштаб для high", &mut c.high_gain_alt, ProbeId::GainHigh, &mut probe_hit);
                });
                ui.collapsing("пороги kick / snare / rythm", |ui| {
                    ui.weak("После инерции: линейный маппер даёт x, кривая сжимает его в порог детектора. Голубая точка - x, жёлтая - порог. ПКМ - лупа по времени.");
                    let _ = probe::mapper_ui(ui, "к порогу kick", "вход: инерция -> x для кривой kick", &mut c.kick_map, ProbeId::KickMap, &mut probe_hit);
                    let _ = probe::sigmoid_ui(ui, "кривая порога kick", &mut c.kick_sigmoid, m.control.kick_x, ProbeId::KickSig, &mut probe_hit, &mut self.sig_win[0]);
                    let _ = probe::mapper_ui(ui, "к порогу snare", "вход: инерция -> x для кривой snare", &mut c.snare_map, ProbeId::SnareMap, &mut probe_hit);
                    let _ = probe::sigmoid_ui(ui, "кривая порога snare", &mut c.snare_sigmoid, m.control.snare_x, ProbeId::SnareSig, &mut probe_hit, &mut self.sig_win[1]);
                    let _ = probe::mapper_ui(ui, "к порогу rythm", "вход: инерция -> x для кривой rythm (в эталоне часто выкл.)", &mut c.rythm_map, ProbeId::RythmMap, &mut probe_hit);
                    let _ = probe::sigmoid_ui(ui, "кривая порога rythm", &mut c.rythm_sigmoid, m.control.rythm_x, ProbeId::RythmSig, &mut probe_hit, &mut self.sig_win[2]);
                });
            });
        });

        {
            let ctrl = &mut self.cfg_edit.control;
            if probe::sigmoid_window(ctx, "кривая порога kick", &mut self.sig_win[0], &mut ctrl.kick_sigmoid, m.control.kick_x) {
                self.config_dirty = true;
            }
        }
        {
            let ctrl = &mut self.cfg_edit.control;
            if probe::sigmoid_window(ctx, "кривая порога snare", &mut self.sig_win[1], &mut ctrl.snare_sigmoid, m.control.snare_x) {
                self.config_dirty = true;
            }
        }
        {
            let ctrl = &mut self.cfg_edit.control;
            if probe::sigmoid_window(ctx, "кривая порога rythm", &mut self.sig_win[2], &mut ctrl.rythm_sigmoid, m.control.rythm_x) {
                self.config_dirty = true;
            }
        }

        if let Some(id) = probe_hit {
            self.probe = Some(id);
            self.probe_entered = false;
        }
        if probe::poster(
            ctx,
            &mut self.show_poster,
            &mut self.cfg_edit,
            &m,
            &mut self.probe,
            &mut self.probe_entered,
        ) {
            self.mark_dirty();
        }
        if probe::popup(
            ctx,
            ProbeUi {
                slot: &mut self.probe,
                entered: &mut self.probe_entered,
                hist: &self.probe_hist,
                metrics: &m,
            },
            &mut self.cfg_edit,
        ) {
            self.mark_dirty();
        }

        // Короткий write-lock: коммитим локальный конфиг при изменениях или во время drag.
        if self.config_dirty || ctx.input(|i| i.pointer.any_down()) {
            self.commit_config();
        }
    }
}

fn spectrum_plot_points(spectrum: &[f32; crate::shared::SPECTRUM_DRAW_BINS], len: usize) -> Vec<[f64; 2]> {
    (0..len.min(crate::shared::SPECTRUM_DRAW_BINS))
        .map(|i| [i as f64, spectrum[i] as f64])
        .collect()
}
