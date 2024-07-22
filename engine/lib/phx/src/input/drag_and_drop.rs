use std::path::{Path, PathBuf};

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragAndDropEvent {
    DroppedFile,
    HoveredFile,
    HoveredFileCancelled,
}

#[derive(Default)]
pub struct DragAndDropState {
    dropped_file: Option<PathBuf>,
    hovered_file: Option<PathBuf>,
    hovered_file_cancelled: bool,
}

impl DragAndDropState {
    pub fn reset(&mut self) {
        self.dropped_file = None;
        self.hovered_file = None;
        self.hovered_file_cancelled = false;
    }

    pub fn update_dropped(&mut self, file: &Path) -> bool {
        self.dropped_file = Some(file.into());
        true
    }

    pub fn update_hovered(&mut self, file: &Path) -> bool {
        self.hovered_file = Some(file.into());
        true
    }

    pub fn update_cancelled(&mut self) -> bool {
        self.hovered_file_cancelled = true;
        true
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl DragAndDropState {
    pub fn get_dropped_file(&self) -> Option<&str> {
        self.dropped_file.as_ref().and_then(|file| file.to_str())
    }

    pub fn get_hovered_file(&self) -> Option<&str> {
        self.hovered_file.as_ref().and_then(|file| file.to_str())
    }

    pub fn if_hovered_file_cancelled(&self) -> bool {
        self.hovered_file_cancelled
    }
}
