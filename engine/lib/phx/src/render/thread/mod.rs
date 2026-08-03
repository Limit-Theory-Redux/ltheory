mod batch_stats;
mod camera_render_data;
mod command_executor;
mod config;
mod entity_render_data;
mod error;
mod render_batch;
mod render_command;
mod renderer;
mod renderer_queue;
mod shader_reload_result;
mod ubo;

// `Renderer`'s two backends: identical public API and FFI surface, selected
// at compile time. See `renderer.rs` for the shared pieces.
#[cfg(not(feature = "immediate"))]
mod render_thread;
#[cfg(feature = "immediate")]
mod renderer_immediate;
#[cfg(not(feature = "immediate"))]
mod renderer_threaded;

pub use batch_stats::*;
pub use camera_render_data::*;
pub use command_executor::*;
pub use config::*;
pub use entity_render_data::*;
pub use error::*;
pub use render_batch::*;
pub use render_command::*;
#[cfg(not(feature = "immediate"))]
pub use render_thread::*;
pub use renderer::*;
#[cfg(feature = "immediate")]
pub use renderer_immediate::*;
pub use renderer_queue::*;
#[cfg(not(feature = "immediate"))]
pub use renderer_threaded::*;
pub use shader_reload_result::*;
pub use ubo::*;
