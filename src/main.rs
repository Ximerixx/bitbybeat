//! bitbybeat — конфигурируемый аудиоанализатор → OSC.
//! Порт TouchDesigner-прототипа `Analysis 2.2`. Документация тракта — в `md_plans/`.

mod audio;
mod config;
mod control;
mod detect;
mod dsp;
mod engine;
mod gui;
mod osc;
mod shared;

use config::Config;
use shared::Shared;
use std::sync::atomic::Ordering;

fn main() -> eframe::Result<()> {
    // Диагностика источников без запуска окна.
    if std::env::args().any(|a| a == "--list-devices") {
        println!("== cpal input devices ==");
        for d in audio::list_input_devices() {
            let tag = if audio::is_monitor(&d) { " [monitor]" } else { "" };
            println!("  {d}{tag}");
        }
        println!("== pulse sources (pactl) ==");
        for s in audio::list_pulse_sources() {
            let tag = if s.is_monitor { " [monitor]" } else { "" };
            println!("  {}{}", s.name, tag);
        }
        return Ok(());
    }

    let config = Config::load_ron("preset.ron").unwrap_or_default();
    let shared = Shared::new(config);

    let engine_handle = engine::spawn(shared.clone());

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1200.0, 820.0]),
        ..Default::default()
    };
    let app = gui::App::new(shared.clone());
    let result = eframe::run_native(
        "bitbybeat",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    );

    // остановить движок и дождаться
    shared.running.store(false, Ordering::Relaxed);
    let _ = engine_handle.join();
    result
}
