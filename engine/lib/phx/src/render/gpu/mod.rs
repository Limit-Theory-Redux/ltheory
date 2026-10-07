//! wgpu-shaped render objects shared by both renderer backends
//! (see `doc/engine/render-api-v2.md`, §1). S2 adds attachment views and
//! render passes.

mod render_pass;
mod tex_view;

pub use render_pass::*;
pub use tex_view::*;
