use std::path::PathBuf;

use crate::internal::static_string;

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

    pub fn update_dropped(&mut self, file: PathBuf) {
        self.dropped_file = Some(file);
    }

    pub fn update_hovered(&mut self, file: PathBuf) {
        self.hovered_file = Some(file);
    }

    pub fn update_cancelled(&mut self) {
        self.hovered_file_cancelled = true;
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl DragAndDropState {
    pub fn get_dropped_file(&self) -> Option<&str> {
        self.dropped_file
            .as_ref()
            .map(|file| file.to_str())
            .flatten()
    }

    pub fn get_hovered_file(&self) -> Option<&str> {
        self.hovered_file
            .as_ref()
            .map(|file| file.to_str())
            .flatten()
    }

    pub fn if_hovered_file_cancelled(&self) -> bool {
        self.hovered_file_cancelled
    }
}
