#![allow(non_upper_case_globals)] // TODO: fix this
#![allow(unsafe_code)] // TODO: remove

use super::*;

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

static mut wireframe: [bool; 16] = [false; 16];

static mut wireframeIndex: i32 = -1;

static mut depthTest: [bool; 16] = [false; 16];

static mut depthTestIndex: i32 = -1;

static mut blendModeIndex: i32 = -1;

static mut blendMode: [BlendMode; 16] = [BlendMode::Additive; 16];

static mut cullFace: [CullFace; 16] = [CullFace::None; 16];

static mut cullFaceIndex: i32 = -1;

static mut depthWritable: [bool; 16] = [false; 16];

static mut depthWritableIndex: i32 = -1;

#[inline]
extern "C" fn RenderState_SetBlendMode(mode: BlendMode) {
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
extern "C" fn RenderState_SetCullFace(mode: CullFace) {
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
extern "C" fn RenderState_SetDepthTest(enabled: bool) {
    if enabled {
        glcheck!(gl::Enable(gl::DEPTH_TEST));
    } else {
        glcheck!(gl::Disable(gl::DEPTH_TEST));
    };
}

#[inline]
extern "C" fn RenderState_SetDepthWritable(enabled: bool) {
    glcheck!(gl::DepthMask(enabled as gl::types::GLboolean));
}

#[inline]
extern "C" fn RenderState_SetWireframe(enabled: bool) {
    if enabled {
        glcheck!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE));
    } else {
        glcheck!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL));
    };
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushAllDefaults() {
    RenderState_PushBlendMode(BlendMode::Disabled);
    RenderState_PushCullFace(CullFace::None);
    RenderState_PushDepthTest(false);
    RenderState_PushDepthWritable(true);
    RenderState_PushWireframe(false);
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopAll() {
    RenderState_PopBlendMode();
    RenderState_PopCullFace();
    RenderState_PopDepthTest();
    RenderState_PopDepthWritable();
    RenderState_PopWireframe();
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopBlendMode() {
    if blendModeIndex < 0 {
        panic!("RenderState_PopBlendMode: Attempting to pop an empty state stack");
    }
    blendModeIndex -= 1;
    if blendModeIndex >= 0 {
        RenderState_SetBlendMode(blendMode[blendModeIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopWireframe() {
    if wireframeIndex < 0 {
        panic!("RenderState_PopWireframe: Attempting to pop an empty state stack");
    }
    wireframeIndex -= 1;
    if wireframeIndex >= 0 {
        RenderState_SetWireframe(wireframe[wireframeIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushBlendMode(value: BlendMode) {
    if blendModeIndex + 1 >= 16 {
        panic!("RenderState_PushBlendMode: Maximum state stack depth exceeded");
    }
    blendModeIndex += 1;
    blendMode[blendModeIndex as usize] = value;
    RenderState_SetBlendMode(value);
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopDepthTest() {
    if depthTestIndex < 0 {
        panic!("RenderState_PopDepthTest: Attempting to pop an empty state stack");
    }
    depthTestIndex -= 1;
    if depthTestIndex >= 0 {
        RenderState_SetDepthTest(depthTest[depthTestIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopCullFace() {
    if cullFaceIndex < 0 {
        panic!("RenderState_PopCullFace: Attempting to pop an empty state stack");
    }
    cullFaceIndex -= 1;
    if cullFaceIndex >= 0 {
        RenderState_SetCullFace(cullFace[cullFaceIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopDepthWritable() {
    if depthWritableIndex < 0 {
        panic!("RenderState_PopDepthWritable: Attempting to pop an empty state stack");
    }
    depthWritableIndex -= 1;
    if depthWritableIndex >= 0 {
        RenderState_SetDepthWritable(depthWritable[depthWritableIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushCullFace(value: CullFace) {
    if cullFaceIndex + 1 >= 16 {
        panic!("RenderState_PushCullFace: Maximum state stack depth exceeded");
    }
    cullFaceIndex += 1;
    cullFace[cullFaceIndex as usize] = value;
    RenderState_SetCullFace(value);
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushDepthTest(value: bool) {
    if depthTestIndex + 1 >= 16 {
        panic!("RenderState_PushDepthTest: Maximum state stack depth exceeded");
    }
    depthTestIndex += 1;
    depthTest[depthTestIndex as usize] = value;
    RenderState_SetDepthTest(value);
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushDepthWritable(value: bool) {
    if depthWritableIndex + 1 >= 16 {
        panic!("RenderState_PushDepthWritable: Maximum state stack depth exceeded");
    }
    depthWritableIndex += 1;
    depthWritable[depthWritableIndex as usize] = value;
    RenderState_SetDepthWritable(value);
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushWireframe(value: bool) {
    if wireframeIndex + 1 >= 16 {
        panic!("RenderState_PushWireframe: Maximum state stack depth exceeded");
    }
    wireframeIndex += 1;
    wireframe[wireframeIndex as usize] = value;
    RenderState_SetWireframe(value);
}
