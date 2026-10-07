//! Pass recording (doc/engine/render-api-v2.md, sections 1.4 and 1.5).
//!
//! Between `beginPass` and `finish` the main thread records `PassCmd`s into
//! the `PassEncoder`. They are flushed to the render thread as one
//! `RenderCommand::PassCommands` (together with the uniform ring bytes they
//! reference) at `PASS_FLUSH_LIMIT` commands, at `finish`, and before any
//! non-pass command, so old and new commands in one pass execute in order.

use std::sync::Arc;

use super::{
    BindGroupId, CameraState, PipelineId, RingChunk, RingOffset, SamplerId, TexView, ViewBlock,
};
use crate::render::ResourceId;

/// Commands recorded between flushes.
pub const PASS_FLUSH_LIMIT: usize = 512;
/// Pass-input slots (group 3 texture units 12..15).
pub const MAX_INPUTS: usize = 4;

pub type PassInputs = [Option<(TexView, SamplerId)>; MAX_INPUTS];

/// One recorded pass command.
#[derive(Debug, Clone)]
pub enum PassCmd {
    SetPipeline(PipelineId),
    /// Bind a material-style bind group (group 1 or any other group).
    SetBindGroup {
        group: u8,
        id: BindGroupId,
    },
    /// Group 0 block: bind `size` bytes at `block` to the view binding.
    SetView {
        block: RingOffset,
        size: u32,
    },
    /// Group 0 textures. `None` unbinds the unit.
    SetEnvironment {
        env_map: Option<ResourceId>,
        ir_map: Option<ResourceId>,
    },
    /// Group 2 block: bind `size` bytes at `at` to the draw binding.
    SetDraw {
        at: RingOffset,
        size: u32,
    },
    /// Group 3 textures (pass inputs).
    SetInputs(Box<PassInputs>),
    SetViewport([i32; 4]),
    SetScissor(Option<[i32; 4]>),
    DrawMesh {
        mesh: ResourceId,
        first_index: u32,
        index_count: u32,
    },
    /// The built-in unit quad (`VertexLayout::Fullscreen`).
    DrawFullscreen,
    /// Instanced draw whose per-instance attributes (`InstanceData`, 84
    /// bytes each, locations 4..9) were written into the vertex ring at
    /// `instances` (see `instanced.glsl`).
    DrawMeshInstanced {
        mesh: ResourceId,
        index_count: u32,
        instances: RingOffset,
        count: u32,
    },
    /// Texture-fetch instancing: `count` u32 indices (attribute location 10)
    /// into a static data texture, written into the vertex ring at `indices`.
    DrawInstancedIndices {
        mesh: ResourceId,
        index_count: u32,
        indices: RingOffset,
        count: u32,
    },
}

impl PassCmd {
    pub fn is_draw(&self) -> bool {
        matches!(
            self,
            PassCmd::DrawMesh { .. }
                | PassCmd::DrawFullscreen
                | PassCmd::DrawMeshInstanced { .. }
                | PassCmd::DrawInstancedIndices { .. }
        )
    }
}

/// What `RenderCommand::PassCommands` carries.
#[derive(Debug, Clone)]
pub struct PassCommands {
    /// Frame slot the uniform chunks belong to.
    pub slot: u8,
    /// Uniform uploads that must happen before `cmds` run.
    pub uniforms: Vec<RingChunk>,
    /// Vertex-ring uploads (instance data, instanced index lists).
    pub vertices: Vec<RingChunk>,
    pub cmds: Vec<PassCmd>,
}

/// Main-thread recording state of the open pass.
#[derive(Default)]
pub struct PassEncoder {
    pub(crate) cmds: Vec<PassCmd>,
    /// Staged group-3 inputs; sent as one `SetInputs` before the next draw.
    pub(crate) inputs: PassInputs,
    pub(crate) inputs_dirty: bool,
}

impl PassEncoder {
    pub fn new() -> Self {
        Self {
            cmds: Vec::with_capacity(PASS_FLUSH_LIMIT + 8),
            inputs: Default::default(),
            inputs_dirty: false,
        }
    }

    pub fn len(&self) -> usize {
        self.cmds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cmds.is_empty()
    }

    pub fn push(&mut self, cmd: PassCmd) {
        self.cmds.push(cmd);
    }

    pub fn take(&mut self) -> Vec<PassCmd> {
        std::mem::replace(&mut self.cmds, Vec::with_capacity(PASS_FLUSH_LIMIT + 8))
    }

    /// Stage input `slot`; `None` clears it.
    pub fn set_input(&mut self, slot: usize, input: Option<(TexView, SamplerId)>) {
        assert!(
            slot < MAX_INPUTS,
            "pass input slot {slot} out of range (max {MAX_INPUTS})"
        );
        self.inputs[slot] = input;
        self.inputs_dirty = true;
    }
}

/// The open pass (one at a time) and the target size last drawn to.
pub struct PassState {
    /// Label of the open pass, if any.
    pub open: Option<Arc<str>>,
    /// Size of the last pass's attachments (kept after `finish`).
    pub extent: [u32; 2],
    pub is_window: bool,
    /// `[x, y, w, h]` of the current viewport.
    pub viewport: [i32; 4],
    /// The pass's current group-0 block; re-allocated when it changes.
    pub view: ViewBlock,
}

impl Default for PassState {
    fn default() -> Self {
        Self {
            open: None,
            extent: [0, 0],
            is_window: false,
            viewport: [0, 0, 0, 0],
            view: ViewBlock::new(&CameraState::default(), [0, 0, 1, 1], false),
        }
    }
}

/// The environment cube maps of group 0. The `TexCube`s are held so the
/// textures outlive the passes that sample them.
#[derive(Default)]
pub struct Environment {
    pub env_map: Option<crate::render::TexCube>,
    pub ir_map: Option<crate::render::TexCube>,
}
