use super::*;
use crate::common::*;

static mut wireframe: [bool; 16] = [false; 16];

static mut wireframeIndex: i32 = -1;

static mut depthTest: [bool; 16] = [false; 16];

static mut depthTestIndex: i32 = -1;

static mut blendModeIndex: i32 = -1;

static mut blendMode: [BlendMode; 16] = [0; 16];

static mut cullFace: [CullFace; 16] = [0; 16];

static mut cullFaceIndex: i32 = -1;

static mut depthWritable: [bool; 16] = [false; 16];

static mut depthWritableIndex: i32 = -1;

#[inline]
extern "C" fn RenderState_SetBlendMode(mode: BlendMode) {
    match mode {
        BlendMode_Additive => {
            gl_blend_func_separate(gl::ONE, gl::ONE, gl::ONE, gl::ONE);
        }
        BlendMode_Alpha => {
            gl_blend_func_separate(
                gl::SRC_ALPHA,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
            );
        }
        BlendMode_PreMultAlpha => {
            gl_blend_func(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
        }
        BlendMode_Disabled => {
            gl_blend_func(gl::ONE, gl::ZERO);
        }
        _ => {}
    }
}

#[inline]
extern "C" fn RenderState_SetCullFace(mode: CullFace) {
    match mode {
        CullFace_None => {
            gl_disable(gl::CULL_FACE);
        }
        CullFace_Back => {
            gl_enable(gl::CULL_FACE);
            gl_cull_face(gl::BACK);
        }
        CullFace_Front => {
            gl_enable(gl::CULL_FACE);
            gl_cull_face(gl::FRONT);
        }
        _ => {}
    }
}

#[inline]
extern "C" fn RenderState_SetDepthTest(enabled: bool) {
    if enabled {
        gl_enable(gl::DEPTH_TEST);
    } else {
        gl_disable(gl::DEPTH_TEST);
    };
}

#[inline]
extern "C" fn RenderState_SetDepthWritable(enabled: bool) {
    gl_depth_mask(enabled as gl::types::GLboolean);
}

#[inline]
extern "C" fn RenderState_SetWireframe(enabled: bool) {
    if enabled {
        gl_polygon_mode(gl::FRONT_AND_BACK, gl::LINE);
    } else {
        gl_polygon_mode(gl::FRONT_AND_BACK, gl::FILL);
    };
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushAllDefaults() {
    RenderState_PushBlendMode(2);
    RenderState_PushCullFace(0);
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
