//! bitbybeat — конфигурируемый аудиоанализатор → OSC.
//! Порт TouchDesigner-прототипа `Analysis 2.2`. Документация тракта — в `md_plans/`.

#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]


mod audio;
mod config;
mod control;
mod detect;
mod dsp;
mod engine;
mod gui;
mod diag;
mod osc;
mod osc_map;
mod preset;
mod probe;
mod shared;

use config::Config;
use diag::LogBus;
use shared::Shared;
use std::sync::atomic::Ordering;

fn main() -> eframe::Result<()> {
    if std::env::args().any(|a| a == "--list-devices") {
        println!("== cpal input devices ==");
        for d in audio::list_input_devices() {
            let tag = if audio::is_monitor(&d.name) { " [monitor]" } else { "" };
            println!("  {} [{}ch]{tag}", d.name, d.channels);
        }
        println!("== pulse sources (pactl) ==");
        for s in audio::list_pulse_sources() {
            let tag = if s.is_monitor { " [monitor]" } else { "" };
            println!("  {}  ({}){}", s.name, s.label(), tag);
        }
        return Ok(());
    }

    let logs = LogBus::new(2000);
    diag::init(logs.clone());

    let config = match Config::load_ron("preset.ron") {
        Ok(c) => {
            diag::info("app", "пресет preset.ron загружен");
            c
        }
        Err(e) => {
            diag::warn("app", format!("preset.ron не загружен ({e}), дефолт"));
            Config::default()
        }
    };

    let shared = Shared::new(config, logs.clone());

    let engine_handle = engine::spawn(shared.clone());
    let osc_handle = osc::spawn(shared.clone());

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

    shared.running.store(false, Ordering::Release);
    let _ = engine_handle.join();
    let _ = osc_handle.join();
    result
}
