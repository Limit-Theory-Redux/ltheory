//! wgpu-shaped render objects shared by both renderer backends
//! (see `doc/engine/render-api-v2.md`, section 1). S2 added attachment views
//! and render passes; S3 the binding model: shader layouts, pipelines,
//! samplers, bind groups, the pass encoder and the per-pass view block; S4 the
//! recycled uniform and vertex rings, the draw block, material parameter
//! arenas, `Material` and the `SceneList`.

mod bind_group;
mod buffer;
mod draw_block;
mod r#gen;
mod imm;
mod layout;
mod material;
mod pass_encoder;
mod pipeline;
mod render_pass;
mod sampler;
mod scene_list;
mod tex_desc;
mod tex_view;
mod uniform_ring;
mod view_block;

pub use bind_group::*;
pub use buffer::*;
pub use draw_block::*;
pub use r#gen::*;
pub use imm::*;
pub use layout::*;
pub use material::*;
pub use pass_encoder::*;
pub use pipeline::*;
pub use render_pass::*;
pub use sampler::*;
pub use scene_list::*;
pub use tex_desc::*;
pub use tex_view::*;
pub use uniform_ring::*;
pub use view_block::*;
