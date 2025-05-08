use std::cell::RefCell;

use crate::render::{gl, glcheck};

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
    pub fn push_all_defaults() {
        RENDER_STATE.with_borrow_mut(|rs| rs.push_all_defaults())
    }

    pub fn push_blend_mode(value: BlendMode) {
        RENDER_STATE.with_borrow_mut(|rs| rs.push_blend_mode(value))
    }

    pub fn push_cull_face(value: CullFace) {
        RENDER_STATE.with_borrow_mut(|rs| rs.push_cull_face(value))
    }

    pub fn push_depth_test(value: bool) {
        RENDER_STATE.with_borrow_mut(|rs| rs.push_depth_test(value))
    }

    pub fn push_depth_writable(value: bool) {
        RENDER_STATE.with_borrow_mut(|rs| rs.push_depth_writable(value))
    }

    pub fn push_wireframe(value: bool) {
        RENDER_STATE.with_borrow_mut(|rs| rs.push_wireframe(value))
    }

    pub fn pop_all() {
        RENDER_STATE.with_borrow_mut(|rs| rs.pop_all())
    }

    pub fn pop_blend_mode() {
        RENDER_STATE.with_borrow_mut(|rs| rs.pop_blend_mode())
    }

    pub fn pop_wireframe() {
        RENDER_STATE.with_borrow_mut(|rs| rs.pop_wireframe())
    }

    pub fn pop_depth_test() {
        RENDER_STATE.with_borrow_mut(|rs| rs.pop_depth_test())
    }

    pub fn pop_cull_face() {
        RENDER_STATE.with_borrow_mut(|rs| rs.pop_cull_face())
    }

    pub fn pop_depth_writable() {
        RENDER_STATE.with_borrow_mut(|rs| rs.pop_depth_writable())
    }
}

thread_local! { static RENDER_STATE: RefCell<RenderStateIntern> = RefCell::new(RenderStateIntern::new()); }

struct RenderStateIntern {
    wireframe: [bool; 16],       // = [false; 16];
    wireframe_index: i32,        // = -1;
    depth_test: [bool; 16],      // = [false; 16];
    depth_test_index: i32,       // = -1;
    blend_mode_index: i32,       // = -1;
    blend_mode: [BlendMode; 16], // = [BlendMode::Additive; 16];
    cull_face: [CullFace; 16],   // = [CullFace::None; 16];
    cull_face_index: i32,        // = -1;
    depth_writable: [bool; 16],  // = [false; 16];
    depth_writable_index: i32,   // = -1;
}

impl RenderStateIntern {
    fn new() -> Self {
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

    fn push_all_defaults(&mut self) {
        self.push_blend_mode(BlendMode::Disabled);
        self.push_cull_face(CullFace::None);
        self.push_depth_test(false);
        self.push_depth_writable(true);
        self.push_wireframe(false);
    }

    fn push_blend_mode(&mut self, value: BlendMode) {
        if self.blend_mode_index + 1 >= 16 {
            panic!("RenderState_PushBlendMode: Maximum state stack depth exceeded");
        }
        self.blend_mode_index += 1;
        self.blend_mode[self.blend_mode_index as usize] = value;
        set_blend_mode(value);
    }

    fn push_cull_face(&mut self, value: CullFace) {
        if self.cull_face_index + 1 >= 16 {
            panic!("RenderState_PushCullFace: Maximum state stack depth exceeded");
        }
        self.cull_face_index += 1;
        self.cull_face[self.cull_face_index as usize] = value;
        set_cull_face(value);
    }

    fn push_depth_test(&mut self, value: bool) {
        if self.depth_test_index + 1 >= 16 {
            panic!("RenderState_PushDepthTest: Maximum state stack depth exceeded");
        }
        self.depth_test_index += 1;
        self.depth_test[self.depth_test_index as usize] = value;
        set_depth_test(value);
    }

    fn push_depth_writable(&mut self, value: bool) {
        if self.depth_writable_index + 1 >= 16 {
            panic!("RenderState_PushDepthWritable: Maximum state stack depth exceeded");
        }
        self.depth_writable_index += 1;
        self.depth_writable[self.depth_writable_index as usize] = value;
        set_depth_writable(value);
    }

    fn push_wireframe(&mut self, value: bool) {
        if self.wireframe_index + 1 >= 16 {
            panic!("RenderState_PushWireframe: Maximum state stack depth exceeded");
        }
        self.wireframe_index += 1;
        self.wireframe[self.wireframe_index as usize] = value;
        set_wireframe(value);
    }

    fn pop_all(&mut self) {
        self.pop_blend_mode();
        self.pop_cull_face();
        self.pop_depth_test();
        self.pop_depth_writable();
        self.pop_wireframe();
    }

    fn pop_blend_mode(&mut self) {
        if self.blend_mode_index < 0 {
            panic!("RenderState_PopBlendMode: Attempting to pop an empty state stack");
        }
        self.blend_mode_index -= 1;
        if self.blend_mode_index >= 0 {
            set_blend_mode(self.blend_mode[self.blend_mode_index as usize]);
        }
    }

    fn pop_wireframe(&mut self) {
        if self.wireframe_index < 0 {
            panic!("RenderState_PopWireframe: Attempting to pop an empty state stack");
        }
        self.wireframe_index -= 1;
        if self.wireframe_index >= 0 {
            set_wireframe(self.wireframe[self.wireframe_index as usize]);
        }
    }

    fn pop_depth_test(&mut self) {
        if self.depth_test_index < 0 {
            panic!("RenderState_PopDepthTest: Attempting to pop an empty state stack");
        }
        self.depth_test_index -= 1;
        if self.depth_test_index >= 0 {
            set_depth_test(self.depth_test[self.depth_test_index as usize]);
        }
    }

    fn pop_cull_face(&mut self) {
        if self.cull_face_index < 0 {
            panic!("RenderState_PopCullFace: Attempting to pop an empty state stack");
        }
        self.cull_face_index -= 1;
        if self.cull_face_index >= 0 {
            set_cull_face(self.cull_face[self.cull_face_index as usize]);
        }
    }

    fn pop_depth_writable(&mut self) {
        if self.depth_writable_index < 0 {
            panic!("RenderState_PopDepthWritable: Attempting to pop an empty state stack");
        }
        self.depth_writable_index -= 1;
        if self.depth_writable_index >= 0 {
            set_depth_writable(self.depth_writable[self.depth_writable_index as usize]);
        }
    }
}

#[inline]
fn set_blend_mode(mode: BlendMode) {
    match mode {
        BlendMode::Additive => {
            glcheck!(gl::BlendFuncSeparate(gl::ONE, gl::ONE, gl::ONE, gl::ONE));
        }
        BlendMode::Alpha => {
            glcheck!(gl::BlendFuncSeparate(
                gl::SRC_ALPHA,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
            ));
        }
        BlendMode::PreMultAlpha => {
            glcheck!(gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA));
        }
        BlendMode::Disabled => {
            glcheck!(gl::BlendFunc(gl::ONE, gl::ZERO));
        }
    }
}

#[inline]
fn set_cull_face(mode: CullFace) {
    match mode {
        CullFace::None => {
            glcheck!(gl::Disable(gl::CULL_FACE));
        }
        CullFace::Back => {
            glcheck!(gl::Enable(gl::CULL_FACE));
            glcheck!(gl::CullFace(gl::BACK));
        }
        CullFace::Front => {
            glcheck!(gl::Enable(gl::CULL_FACE));
            glcheck!(gl::CullFace(gl::FRONT));
        }
    }
}

#[inline]
fn set_depth_test(enabled: bool) {
    if enabled {
        glcheck!(gl::Enable(gl::DEPTH_TEST));
    } else {
        glcheck!(gl::Disable(gl::DEPTH_TEST));
    };
}

#[inline]
fn set_depth_writable(enabled: bool) {
    glcheck!(gl::DepthMask(enabled as gl::types::GLboolean));
}

#[inline]
fn set_wireframe(enabled: bool) {
    if enabled {
        glcheck!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE));
    } else {
        glcheck!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL));
    };
}
