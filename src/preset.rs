//! Пресеты: undo/redo, autosave, сравнение «сохранён / редактируется».

use crate::config::{Config, InputCfg, Source};
use std::path::{Path, PathBuf};

const UNDO_MAX: usize = 32;

/// Ключ аудиовхода — сравниваем только то, что влияет на restart.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AudioInputKey {
    pub source: Source,
    pub device: Option<String>,
    pub pulse_source: Option<String>,
    pub channels_pick: Vec<usize>,
}

impl From<&InputCfg> for AudioInputKey {
    fn from(i: &InputCfg) -> Self {
        Self {
            source: i.source,
            device: i.device.clone(),
            pulse_source: i.pulse_source.clone(),
            channels_pick: i.channels_pick.clone(),
        }
    }
}

pub struct UndoStack {
    past: Vec<Config>,
    future: Vec<Config>,
    /// Снимок на момент нажатия мыши (до drag).
    drag_anchor: Option<Config>,
}

impl UndoStack {
    pub fn new() -> Self {
        Self { past: Vec::new(), future: Vec::new(), drag_anchor: None }
    }

    pub fn on_pointer_pressed(&mut self, cfg: &Config) {
        if self.drag_anchor.is_none() {
            self.drag_anchor = Some(cfg.clone());
        }
    }

    /// Вернуть true, если в стек попало новое состояние.
    pub fn on_pointer_released(&mut self, cfg: &Config) -> bool {
        let Some(before) = self.drag_anchor.take() else {
            return false;
        };
        if before == *cfg {
            return false;
        }
        if self.past.len() >= UNDO_MAX {
            self.past.remove(0);
        }
        self.past.push(before);
        self.future.clear();
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn undo(&mut self, current: &Config) -> Option<Config> {
        let prev = self.past.pop()?;
        self.future.push(current.clone());
        Some(prev)
    }

    pub fn redo(&mut self, current: &Config) -> Option<Config> {
        let next = self.future.pop()?;
        self.past.push(current.clone());
        Some(next)
    }

    pub fn clear(&mut self) {
        self.past.clear();
        self.future.clear();
        self.drag_anchor = None;
    }
}

pub fn autosave_path(preset_path: &str) -> PathBuf {
    let p = Path::new(preset_path);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("preset");
    let parent = p.parent().unwrap_or(Path::new("."));
    parent.join(format!("{stem}_autosave.ron"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_with_rate(hz: f32) -> Config {
        let mut c = Config::default();
        c.osc_rate_hz = hz;
        c
    }

    #[test]
    fn release_without_press_records_nothing() {
        let mut s = UndoStack::new();
        assert!(!s.on_pointer_released(&cfg_with_rate(60.0)));
        assert!(!s.can_undo());
    }

    #[test]
    fn drag_without_change_records_nothing() {
        let mut s = UndoStack::new();
        let cfg = cfg_with_rate(60.0);
        s.on_pointer_pressed(&cfg);
        assert!(!s.on_pointer_released(&cfg));
        assert!(!s.can_undo());
    }

    #[test]
    fn undo_returns_state_from_before_the_drag() {
        let mut s = UndoStack::new();
        let before = cfg_with_rate(60.0);
        let after = cfg_with_rate(90.0);
        s.on_pointer_pressed(&before);
        assert!(s.on_pointer_released(&after));

        let undone = s.undo(&after).expect("undo available");
        assert_eq!(undone.osc_rate_hz, 60.0);
        assert!(!s.can_undo());
        assert!(s.can_redo());

        let redone = s.redo(&undone).expect("redo available");
        assert_eq!(redone.osc_rate_hz, 90.0);
    }

    #[test]
    fn nested_press_keeps_the_first_anchor() {
        // Второй pressed без released не должен затирать точку отсчёта drag.
        let mut s = UndoStack::new();
        s.on_pointer_pressed(&cfg_with_rate(60.0));
        s.on_pointer_pressed(&cfg_with_rate(75.0));
        assert!(s.on_pointer_released(&cfg_with_rate(90.0)));
        assert_eq!(s.undo(&cfg_with_rate(90.0)).unwrap().osc_rate_hz, 60.0);
    }

    #[test]
    fn new_edit_clears_the_redo_branch() {
        let mut s = UndoStack::new();
        s.on_pointer_pressed(&cfg_with_rate(60.0));
        s.on_pointer_released(&cfg_with_rate(90.0));
        s.undo(&cfg_with_rate(90.0));
        assert!(s.can_redo());

        s.on_pointer_pressed(&cfg_with_rate(60.0));
        s.on_pointer_released(&cfg_with_rate(120.0));
        assert!(!s.can_redo());
    }

    #[test]
    fn stack_is_bounded_and_drops_the_oldest_state() {
        let mut s = UndoStack::new();
        for i in 0..(UNDO_MAX + 5) {
            s.on_pointer_pressed(&cfg_with_rate(i as f32));
            s.on_pointer_released(&cfg_with_rate(i as f32 + 1000.0));
        }
        assert_eq!(s.past.len(), UNDO_MAX);
        // Самое старое состояние (0.0) вытеснено.
        assert!(s.past.iter().all(|c| c.osc_rate_hz >= 5.0));
    }

    #[test]
    fn clear_drops_history_and_pending_drag() {
        let mut s = UndoStack::new();
        s.on_pointer_pressed(&cfg_with_rate(60.0));
        s.on_pointer_released(&cfg_with_rate(90.0));
        s.on_pointer_pressed(&cfg_with_rate(90.0));
        s.clear();
        assert!(!s.can_undo());
        assert!(!s.can_redo());
        assert!(!s.on_pointer_released(&cfg_with_rate(120.0)));
    }

    #[test]
    fn undo_on_empty_stack_returns_none() {
        let mut s = UndoStack::new();
        assert!(s.undo(&cfg_with_rate(60.0)).is_none());
        assert!(s.redo(&cfg_with_rate(60.0)).is_none());
    }

    #[test]
    fn autosave_sits_next_to_the_preset() {
        assert_eq!(autosave_path("presets/hall.ron"), Path::new("presets/hall_autosave.ron"));
        assert_eq!(autosave_path("preset.ron"), Path::new("preset_autosave.ron"));
    }

    #[test]
    fn autosave_handles_paths_without_a_usable_stem() {
        // У пустого пути нет ни stem, ни родителя — падаем на «preset» в текущем каталоге.
        assert_eq!(autosave_path(""), Path::new(".").join("preset_autosave.ron"));
    }

    #[test]
    fn audio_key_ignores_fields_that_do_not_need_a_restart() {
        let mut a = InputCfg::default();
        let mut b = InputCfg::default();
        b.prefer_monitor = !a.prefer_monitor;
        assert_eq!(AudioInputKey::from(&a), AudioInputKey::from(&b));

        a.device = Some("hw:1".into());
        assert_ne!(AudioInputKey::from(&a), AudioInputKey::from(&b));
    }
}