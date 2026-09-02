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

/// Захват mutex внутри лога с восстановлением после отравления.
///
/// Нельзя звать [`warn`] при ошибке — это сам лог, получилась бы рекурсия; пишем прямо в stderr.
fn lock_or_recover<'a, T>(m: &'a Mutex<T>, name: &str) -> std::sync::MutexGuard<'a, T> {
    match m.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            eprintln!("[WARN] diag: mutex '{name}' poisoned, recovering");
            poisoned.into_inner()
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
            *lock_or_recover(&self.osc_error_msg, "osc_error_msg") = Some(msg.clone());
            self.osc_error_dialog.store(true, Ordering::Release);
        }
        // Дублируем в stderr только значимое: debug/info на 120 Гц забили бы консоль
        // и добавили блокирующий I/O в compute-цикл.
        if level >= LogLevel::Warning {
            eprintln!("[{}] {}: {}", level.label(), target, msg);
        }
        let mut q = lock_or_recover(&self.entries, "entries");
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

    pub fn take_osc_error_dialog(&self) -> Option<String> {
        if self.osc_error_dialog.swap(false, Ordering::AcqRel) {
            lock_or_recover(&self.osc_error_msg, "osc_error_msg").take()
        } else {
            None
        }
    }

    pub fn snapshot(&self) -> Vec<LogEntry> {
        lock_or_recover(&self.entries, "entries").iter().cloned().collect()
    }

    pub fn clear(&self) {
        lock_or_recover(&self.entries, "entries").clear();
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

/// Записать в глобальную шину; до её инициализации (ранний старт, тесты) — в stderr,
/// иначе такие сообщения терялись бы полностью.
fn log(level: LogLevel, target: &str, message: impl Into<String>) {
    match bus() {
        Some(b) => b.push(level, target, message),
        None => eprintln!("[{}] {}: {}", level.label(), target, message.into()),
    }
}

pub fn debug(target: &str, message: impl Into<String>) {
    log(LogLevel::Debug, target, message);
}

pub fn info(target: &str, message: impl Into<String>) {
    log(LogLevel::Info, target, message);
}

pub fn warn(target: &str, message: impl Into<String>) {
    log(LogLevel::Warning, target, message);
}

pub fn error(target: &str, message: impl Into<String>) {
    log(LogLevel::Error, target, message);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn osc_error_raises_dialog_once() {
        let bus = LogBus::new(10);
        bus.push(LogLevel::Error, "osc", "port busy");
        assert_eq!(bus.take_osc_error_dialog(), Some("port busy".to_string()));
        assert_eq!(bus.take_osc_error_dialog(), None);
    }

    #[test]
    fn non_osc_error_does_not_raise_dialog() {
        let bus = LogBus::new(10);
        bus.push(LogLevel::Error, "audio", "device lost");
        assert_eq!(bus.take_osc_error_dialog(), None);
    }

    #[test]
    fn ring_buffer_drops_oldest_entries() {
        let bus = LogBus::new(2);
        bus.push(LogLevel::Info, "t1", "m1");
        bus.push(LogLevel::Info, "t2", "m2");
        bus.push(LogLevel::Info, "t3", "m3");
        let targets: Vec<_> = bus.snapshot().iter().map(|e| e.target.clone()).collect();
        assert_eq!(targets, vec!["t2", "t3"]);
    }

    #[test]
    fn snapshot_keeps_level_and_message() {
        let bus = LogBus::new(4);
        bus.push(LogLevel::Warning, "engine", "slow tick");
        let snap = bus.snapshot();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap[0].level, LogLevel::Warning);
        assert_eq!(snap[0].message, "slow tick");
    }

    #[test]
    fn clear_empties_the_ring() {
        let bus = LogBus::new(4);
        bus.push(LogLevel::Info, "t", "m");
        bus.clear();
        assert!(bus.snapshot().is_empty());
    }

    #[test]
    fn push_survives_poisoned_entries_lock() {
        let bus = LogBus::new(4);
        let b = Arc::clone(&bus);
        let _ = std::thread::spawn(move || {
            let _g = b.entries.lock().unwrap();
            panic!("poison the log");
        })
        .join();

        bus.push(LogLevel::Info, "after", "still works");
        assert!(bus.snapshot().iter().any(|e| e.target == "after"));
    }
}
