#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::explicit_auto_deref)]
#![allow(clippy::new_without_default)]
#![allow(clippy::module_inception)] // Allow nested module with the same name as parent

pub mod audio;
pub mod common;
pub mod engine;
pub mod error;
pub mod event_bus;
pub mod input;
pub mod logging;
pub mod math;
pub mod physics;
pub mod render;
pub mod rf;
pub mod system;
pub mod ui;
pub mod window;
