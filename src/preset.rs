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