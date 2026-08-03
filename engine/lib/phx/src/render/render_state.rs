use crate::render::{RenderCommand, Renderer};

#[luajit_ffi_gen::luajit_ffi]
#[derive(Default, Debug, Copy, Clone)]
pub enum BlendMode {
    #[default]
    Disabled,
    Additive,
    Alpha,
    PreMultAlpha,
}

#[luajit_ffi_gen::luajit_ffi]
#[derive(Default, Debug, Copy, Clone)]
pub enum CullFace {
    #[default]
    None,
    Back,
    Front,
}

pub struct RenderState;

#[luajit_ffi_gen::luajit_ffi]
impl RenderState {
    pub fn push_all_defaults(r: &mut Renderer) {
        Self::push_blend_mode(r, BlendMode::Disabled);
        Self::push_cull_face(r, CullFace::None);
        Self::push_depth_test(r, false);
        Self::push_depth_writable(r, true);
        Self::push_wireframe(r, false);
    }

    pub fn push_blend_mode(r: &mut Renderer, value: BlendMode) {
        r.render_state.push_blend_mode(value);
        r.submit(RenderCommand::SetBlendMode(value));
    }

    pub fn push_cull_face(r: &mut Renderer, value: CullFace) {
        r.render_state.push_cull_face(value);
        r.submit(RenderCommand::SetCullFace(value));
    }

    pub fn push_depth_test(r: &mut Renderer, value: bool) {
        r.render_state.push_depth_test(value);
        r.submit(RenderCommand::SetDepthTest(value));
    }

    pub fn push_depth_writable(r: &mut Renderer, value: bool) {
        r.render_state.push_depth_writable(value);
        r.submit(RenderCommand::SetDepthWritable(value));
    }

    pub fn push_wireframe(r: &mut Renderer, value: bool) {
        r.render_state.push_wireframe(value);
        r.submit(RenderCommand::SetWireframe(value));
    }

    pub fn pop_all(r: &mut Renderer) {
        if let Some(mode) = r.render_state.pop_blend_mode() {
            r.submit(RenderCommand::SetBlendMode(mode));
        }
        if let Some(face) = r.render_state.pop_cull_face() {
            r.submit(RenderCommand::SetCullFace(face));
        }
        if let Some(value) = r.render_state.pop_depth_test() {
            r.submit(RenderCommand::SetDepthTest(value));
        }
        if let Some(value) = r.render_state.pop_depth_writable() {
            r.submit(RenderCommand::SetDepthWritable(value));
        }
        if let Some(value) = r.render_state.pop_wireframe() {
            r.submit(RenderCommand::SetWireframe(value));
        }
    }

    pub fn pop_blend_mode(r: &mut Renderer) {
        if let Some(mode) = r.render_state.pop_blend_mode() {
            r.submit(RenderCommand::SetBlendMode(mode));
        }
    }

    pub fn pop_wireframe(r: &mut Renderer) {
        if let Some(value) = r.render_state.pop_wireframe() {
            r.submit(RenderCommand::SetWireframe(value));
        }
    }

    pub fn pop_depth_test(r: &mut Renderer) {
        if let Some(value) = r.render_state.pop_depth_test() {
            r.submit(RenderCommand::SetDepthTest(value));
        }
    }

    pub fn pop_cull_face(r: &mut Renderer) {
        if let Some(face) = r.render_state.pop_cull_face() {
            r.submit(RenderCommand::SetCullFace(face));
        }
    }

    pub fn pop_depth_writable(r: &mut Renderer) {
        if let Some(value) = r.render_state.pop_depth_writable() {
            r.submit(RenderCommand::SetDepthWritable(value));
        }
    }
}

/// GL state stack, owned by `Renderer` (was `thread_local! RENDER_STATE`) -
/// GPU state has to live with whatever owns the GL context. Push methods
/// return nothing to submit (the new value is always known at the call
/// site); pop methods return the value to restore, or `None` if the stack
/// is now empty, so the caller submits it.
pub struct RenderStateIntern {
    wireframe: [bool; 16],
    wireframe_index: i32,
    depth_test: [bool; 16],
    depth_test_index: i32,
    blend_mode_index: i32,
    blend_mode: [BlendMode; 16],
    cull_face: [CullFace; 16],
    cull_face_index: i32,
    depth_writable: [bool; 16],
    depth_writable_index: i32,
}

impl RenderStateIntern {
    pub fn new() -> Self {
        Self {
            wireframe: [false; 16],
            wireframe_index: -1,
            depth_test: [false; 16],
            depth_test_index: -1,
            blend_mode_index: -1,
            blend_mode: [BlendMode::Additive; 16],
            cull_face: [CullFace::None; 16],
            cull_face_index: -1,
            depth_writable: [false; 16],
            depth_writable_index: -1,
        }
    }

    fn push_blend_mode(&mut self, value: BlendMode) {
        if self.blend_mode_index + 1 >= 16 {
            panic!("RenderState_PushBlendMode: Maximum state stack depth exceeded");
        }
        self.blend_mode_index += 1;
        self.blend_mode[self.blend_mode_index as usize] = value;
    }

    fn push_cull_face(&mut self, value: CullFace) {
        if self.cull_face_index + 1 >= 16 {
            panic!("RenderState_PushCullFace: Maximum state stack depth exceeded");
        }
        self.cull_face_index += 1;
        self.cull_face[self.cull_face_index as usize] = value;
    }

    fn push_depth_test(&mut self, value: bool) {
        if self.depth_test_index + 1 >= 16 {
            panic!("RenderState_PushDepthTest: Maximum state stack depth exceeded");
        }
        self.depth_test_index += 1;
        self.depth_test[self.depth_test_index as usize] = value;
    }

    fn push_depth_writable(&mut self, value: bool) {
        if self.depth_writable_index + 1 >= 16 {
            panic!("RenderState_PushDepthWritable: Maximum state stack depth exceeded");
        }
        self.depth_writable_index += 1;
        self.depth_writable[self.depth_writable_index as usize] = value;
    }

    fn push_wireframe(&mut self, value: bool) {
        if self.wireframe_index + 1 >= 16 {
            panic!("RenderState_PushWireframe: Maximum state stack depth exceeded");
        }
        self.wireframe_index += 1;
        self.wireframe[self.wireframe_index as usize] = value;
    }

    fn pop_blend_mode(&mut self) -> Option<BlendMode> {
        if self.blend_mode_index < 0 {
            panic!("RenderState_PopBlendMode: Attempting to pop an empty state stack");
        }
        self.blend_mode_index -= 1;
        (self.blend_mode_index >= 0).then(|| self.blend_mode[self.blend_mode_index as usize])
    }

    fn pop_wireframe(&mut self) -> Option<bool> {
        if self.wireframe_index < 0 {
            panic!("RenderState_PopWireframe: Attempting to pop an empty state stack");
        }
        self.wireframe_index -= 1;
        (self.wireframe_index >= 0).then(|| self.wireframe[self.wireframe_index as usize])
    }

    fn pop_depth_test(&mut self) -> Option<bool> {
        if self.depth_test_index < 0 {
            panic!("RenderState_PopDepthTest: Attempting to pop an empty state stack");
        }
        self.depth_test_index -= 1;
        (self.depth_test_index >= 0).then(|| self.depth_test[self.depth_test_index as usize])
    }

    fn pop_cull_face(&mut self) -> Option<CullFace> {
        if self.cull_face_index < 0 {
            panic!("RenderState_PopCullFace: Attempting to pop an empty state stack");
        }
        self.cull_face_index -= 1;
        (self.cull_face_index >= 0).then(|| self.cull_face[self.cull_face_index as usize])
    }

    fn pop_depth_writable(&mut self) -> Option<bool> {
        if self.depth_writable_index < 0 {
            panic!("RenderState_PopDepthWritable: Attempting to pop an empty state stack");
        }
        self.depth_writable_index -= 1;
        (self.depth_writable_index >= 0)
            .then(|| self.depth_writable[self.depth_writable_index as usize])
    }
}

impl Default for RenderStateIntern {
    fn default() -> Self {
        Self::new()
    }
}
