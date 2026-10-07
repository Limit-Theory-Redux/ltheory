//! wgpu-shaped render objects shared by both renderer backends
//! (see `doc/engine/render-api-v2.md`, section 1). S2 added attachment views
//! and render passes; S3 adds the binding model: shader layouts, pipelines,
//! samplers, bind groups, the pass encoder and the per-pass view block.

mod bind_group;
mod draw_block;
mod layout;
mod pass_encoder;
mod pipeline;
mod render_pass;
mod sampler;
mod tex_view;
mod uniform_ring;
mod view_block;

pub use bind_group::*;
pub use draw_block::*;
pub use layout::*;
pub use pass_encoder::*;
pub use pipeline::*;
pub use render_pass::*;
pub use sampler::*;
pub use tex_view::*;
pub use uniform_ring::*;
pub use view_block::*;
