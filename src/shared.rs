//! Разделяемое состояние между GUI, движком и OSC.

use crate::config::Config;
use crate::control::ControlOut;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

/// Снимок метрик для GUI/мониторинга (движок пишет, GUI читает).
#[derive(Clone, Default)]
pub struct Metrics {
    pub device_name: String,
    pub sample_rate: f32,
    pub input_rms: f32,
    pub band_levels: [f32; 3],   // low, mid, high (после сглаживания)
    pub band_rms: [f32; 3],      // pre-gain rms полос
    pub kick: (f32, f32),        // (gate, trigger)
    pub snare: (f32, f32),
    pub rythm: (f32, f32),
    pub kick_env: f32,           // огибающие для ламп (импульс + плавный спад)
    pub snare_env: f32,
    pub rythm_env: f32,
    pub control: ControlOut,
    pub centroid: f32,
    pub fms: f32,
    pub sms: f32,
    pub flux: f32,
    pub dsp_rms: f32,
    pub spectrum: Vec<f32>,      // магнитуды (усечённые для отрисовки)
    pub osc_channels: usize,
    pub error: Option<String>,
}

pub struct Shared {
    pub config: Mutex<Config>,
    pub metrics: Mutex<Metrics>,
    pub running: AtomicBool,
    pub restart_audio: AtomicBool,
}

impl Shared {
    pub fn new(config: Config) -> Arc<Self> {
        Arc::new(Self {
            config: Mutex::new(config),
            metrics: Mutex::new(Metrics::default()),
            running: AtomicBool::new(true),
            restart_audio: AtomicBool::new(false),
        })
    }
}
