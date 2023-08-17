use std::path::{Path, PathBuf};

use crate::internal::static_string;

#[luajit_ffi_gen::luajit_ffi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemEvent {
    Exit,
}

#[derive(Default)]
pub struct SystemEventState {
    exit: bool,
}

impl SystemEventState {
    pub fn set_exit(&mut self, exit: bool) -> bool {
        self.exit = exit;
        true
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl SystemEventState {
    pub fn is_exit(&self) -> bool {
        self.exit
    }
}
