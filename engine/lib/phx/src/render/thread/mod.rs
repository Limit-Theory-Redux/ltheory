mod batch_stats;
mod camera_render_data;
mod command_category;
mod command_executor;
mod command_executor_gl;
mod config;
mod entity_render_data;
mod error;
mod instance_batch;
mod instance_data;
mod render_batch;
mod render_command;
mod renderer_data;
mod renderer_ffi;
mod renderer_stats;
mod resource_handle;
mod shader_reload_result;
// Both files carry an inner `#![cfg(feature = "stats-server")]`, so they
// compile to nothing without the feature - no outer gate needed here.
#[cfg(feature = "stats-server")]
mod stats_server;
#[cfg(feature = "stats-server")]
mod stats_snapshot;
mod ubo;
mod vertex_format;

// `Renderer`'s two backends: identical public API and FFI surface, selected
// at compile time. See `renderer_shared.rs` for the shared pieces.
#[cfg(not(feature = "immediate"))]
mod render_thread;
#[cfg(feature = "immediate")]
mod renderer_immediate;
#[cfg(not(feature = "immediate"))]
mod renderer_threaded;

pub use batch_stats::*;
pub use camera_render_data::*;
pub use command_category::*;
pub use command_executor::*;
pub use config::*;
pub use entity_render_data::*;
pub use error::*;
pub use instance_batch::*;
pub use instance_data::*;
pub use render_batch::*;
pub use render_command::*;
#[cfg(not(feature = "immediate"))]
pub use render_thread::*;
pub use renderer_data::*;
pub use renderer_ffi::*;
#[cfg(feature = "immediate")]
pub use renderer_immediate::*;
pub use renderer_stats::*;
#[cfg(not(feature = "immediate"))]
pub use renderer_threaded::*;
pub use resource_handle::*;
pub use shader_reload_result::*;
#[cfg(feature = "stats-server")]
pub use stats_server::*;
#[cfg(feature = "stats-server")]
pub use stats_snapshot::*;
pub use ubo::*;
pub use vertex_format::*;
