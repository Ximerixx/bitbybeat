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

/// Открыть вход на несколько секунд и показать, идёт ли звук. Нужно, когда в GUI «тишина»
/// и непонятно, виноват источник, устройство или сам тракт.
fn probe_input(device: Option<&str>) {
    use ringbuf::traits::{Consumer, Observer, Split};

    let pulse = device.filter(|d| d.starts_with("pulse:")).map(|d| &d["pulse:".len()..]);
    let cpal_dev = if pulse.is_some() { None } else { device };
    let rb = ringbuf::HeapRb::<f32>::new(48000 * 4);
    let (prod, mut cons) = rb.split();

    let input = match audio::AudioInput::open(cpal_dev, pulse, &[], prod) {
        Ok(i) => i,
        Err(e) => {
            println!("не удалось открыть вход: {e}");
            return;
        }
    };
    println!("вход: {} @ {} Гц, {} кан.", input.device_name, input.sample_rate, input.channels);
    println!("слушаю 3 с...");

    let mut total = 0usize;
    let mut peak = 0.0f32;
    let mut sum_sq = 0.0f64;
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_secs(3) {
        std::thread::sleep(std::time::Duration::from_millis(50));
        while let Some(s) = cons.try_pop() {
            total += 1;
            peak = peak.max(s.abs());
            sum_sq += (s as f64) * (s as f64);
        }
    }
    let rms = if total > 0 { (sum_sq / total as f64).sqrt() } else { 0.0 };
    println!("сэмплов: {total} (ожидалось ~{})", (input.sample_rate * 3.0) as usize);
    println!("пик: {peak:.4}   RMS: {rms:.4}");
    println!("ringbuf ёмкость: {}", cons.capacity());
    if total == 0 {
        println!("ВЕРДИКТ: данные не идут — устройство открылось, но сэмплов нет");
    } else if peak < 1e-5 {
        println!("ВЕРДИКТ: идёт тишина — проверьте, что на этом устройстве реально играет звук");
    } else {
        println!("ВЕРДИКТ: сигнал есть");
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--probe-input") {
        diag::init(LogBus::new(64));
        probe_input(args.get(i + 1).map(|s| s.as_str()));
        return Ok(());
    }

    if std::env::args().any(|a| a == "--list-devices") {
        println!("== cpal input devices ==");
        for d in audio::list_input_devices() {
            let tag = if d.is_loopback {
                " [loopback: системный вывод]"
            } else if audio::is_monitor(&d.name) {
                " [monitor]"
            } else {
                ""
            };
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
