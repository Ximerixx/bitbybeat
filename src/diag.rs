//! Централизованный лог: ring-buffer + флаги для GUI (диалог ошибок OSC, консоль).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}

impl LogLevel {
    pub fn label(self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    pub fn color_rgb(self) -> (u8, u8, u8) {
        match self {
            LogLevel::Debug => (160, 160, 160),
            LogLevel::Info => (120, 180, 255),
            LogLevel::Warning => (255, 220, 80),
            LogLevel::Error => (255, 80, 80),
        }
    }
}

#[derive(Clone)]
pub struct LogEntry {
    pub at: SystemTime,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
}

pub struct LogBus {
    entries: Mutex<VecDeque<LogEntry>>,
    max_entries: usize,
    /// Показать модальный диалог с последней OSC-ошибкой.
    pub osc_error_dialog: AtomicBool,
    osc_error_msg: Mutex<Option<String>>,
}

impl LogBus {
    pub fn new(max_entries: usize) -> Arc<Self> {
        Arc::new(Self {
            entries: Mutex::new(VecDeque::with_capacity(max_entries.min(512))),
            max_entries,
            osc_error_dialog: AtomicBool::new(false),
            osc_error_msg: Mutex::new(None),
        })
    }

    pub fn push(&self, level: LogLevel, target: &str, message: impl Into<String>) {
        let msg = message.into();
        if level == LogLevel::Error && target == "osc" {
            *self.osc_error_msg.lock().unwrap() = Some(msg.clone());
            self.osc_error_dialog.store(true, Ordering::Release);
        }
        let mut q = self.entries.lock().unwrap();
        if q.len() >= self.max_entries {
            q.pop_front();
        }
        q.push_back(LogEntry {
            at: SystemTime::now(),
            level,
            target: target.into(),
            message: msg,
        });
    }

    pub fn debug(&self, target: &str, message: impl Into<String>) {
        self.push(LogLevel::Debug, target, message);
    }

    pub fn info(&self, target: &str, message: impl Into<String>) {
        self.push(LogLevel::Info, target, message);
    }

    pub fn warn(&self, target: &str, message: impl Into<String>) {
        self.push(LogLevel::Warning, target, message);
    }

    pub fn error(&self, target: &str, message: impl Into<String>) {
        self.push(LogLevel::Error, target, message);
    }

    pub fn take_osc_error_dialog(&self) -> Option<String> {
        if self.osc_error_dialog.swap(false, Ordering::AcqRel) {
            self.osc_error_msg.lock().unwrap().take()
        } else {
            None
        }
    }

    pub fn snapshot(&self) -> Vec<LogEntry> {
        self.entries.lock().unwrap().iter().cloned().collect()
    }

    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }
}

/// Глобальный bus (устанавливается при старте приложения).
static LOG: std::sync::OnceLock<Arc<LogBus>> = std::sync::OnceLock::new();

pub fn init(bus: Arc<LogBus>) {
    let _ = LOG.set(bus);
}

pub fn bus() -> Option<Arc<LogBus>> {
    LOG.get().cloned()
}

pub fn debug(target: &str, message: impl Into<String>) {
    if let Some(b) = bus() {
        b.debug(target, message);
    }
}

pub fn info(target: &str, message: impl Into<String>) {
    if let Some(b) = bus() {
        b.info(target, message);
    }
}

pub fn warn(target: &str, message: impl Into<String>) {
    if let Some(b) = bus() {
        b.warn(target, message);
    }
}

pub fn error(target: &str, message: impl Into<String>) {
    if let Some(b) = bus() {
        b.error(target, message);
    }
}

pub fn format_time(t: SystemTime) -> String {
    match t.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => {
            let secs = d.as_secs() % 86_400;
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            let ms = d.subsec_millis();
            format!("{h:02}:{m:02}:{s:02}.{ms:03}")
        }
        Err(_) => "??:??:??.???".into(),
    }
}
