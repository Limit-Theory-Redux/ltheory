use crate::internal::Memory::*;
use crate::BlendMode::*;
use crate::Common::*;
use crate::CullFace::*;
use crate::Math::Vec3;
use crate::GL::gl;
use libc;

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
unsafe extern "C" fn RenderState_SetBlendMode(mode: BlendMode) {
    match mode {
        BlendMode_Additive => {
            gl::BlendFuncSeparate(gl::ONE, gl::ONE, gl::ONE, gl::ONE);
        }
        BlendMode_Alpha => {
            gl::BlendFuncSeparate(
                gl::SRC_ALPHA,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
            );
        }
        BlendMode_PreMultAlpha => {
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
        }
        BlendMode_Disabled => {
            gl::BlendFunc(gl::ONE, gl::ZERO);
        }
        _ => {}
    }
}

#[inline]
unsafe extern "C" fn RenderState_SetCullFace(mode: CullFace) {
    match mode {
        CullFace_None => {
            gl::Disable(gl::CULL_FACE);
        }
        CullFace_Back => {
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
        }
        CullFace_Front => {
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::FRONT);
        }
        _ => {}
    }
}

#[inline]
unsafe extern "C" fn RenderState_SetDepthTest(enabled: bool) {
    if enabled {
        gl::Enable(gl::DEPTH_TEST);
    } else {
        gl::Disable(gl::DEPTH_TEST);
    };
}

#[inline]
unsafe extern "C" fn RenderState_SetDepthWritable(enabled: bool) {
    gl::DepthMask(enabled as gl::types::GLboolean);
}

#[inline]
unsafe extern "C" fn RenderState_SetWireframe(enabled: bool) {
    if enabled {
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
    } else {
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
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
        Fatal(
            b"RenderState_PopBlendMode: Attempting to pop an empty state stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    blendModeIndex -= 1;
    if blendModeIndex >= 0 {
        RenderState_SetBlendMode(blendMode[blendModeIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopWireframe() {
    if wireframeIndex < 0 {
        Fatal(
            b"RenderState_PopWireframe: Attempting to pop an empty state stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    wireframeIndex -= 1;
    if wireframeIndex >= 0 {
        RenderState_SetWireframe(wireframe[wireframeIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushBlendMode(value: BlendMode) {
    if blendModeIndex + 1 >= 16 {
        Fatal(
            b"RenderState_PushBlendMode: Maximum state stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    blendModeIndex += 1;
    blendMode[blendModeIndex as usize] = value;
    RenderState_SetBlendMode(value);
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopDepthTest() {
    if depthTestIndex < 0 {
        Fatal(
            b"RenderState_PopDepthTest: Attempting to pop an empty state stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    depthTestIndex -= 1;
    if depthTestIndex >= 0 {
        RenderState_SetDepthTest(depthTest[depthTestIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopCullFace() {
    if cullFaceIndex < 0 {
        Fatal(
            b"RenderState_PopCullFace: Attempting to pop an empty state stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    cullFaceIndex -= 1;
    if cullFaceIndex >= 0 {
        RenderState_SetCullFace(cullFace[cullFaceIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopDepthWritable() {
    if depthWritableIndex < 0 {
        Fatal(
            b"RenderState_PopDepthWritable: Attempting to pop an empty state stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    depthWritableIndex -= 1;
    if depthWritableIndex >= 0 {
        RenderState_SetDepthWritable(depthWritable[depthWritableIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushCullFace(value: CullFace) {
    if cullFaceIndex + 1 >= 16 {
        Fatal(
            b"RenderState_PushCullFace: Maximum state stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    cullFaceIndex += 1;
    cullFace[cullFaceIndex as usize] = value;
    RenderState_SetCullFace(value);
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushDepthTest(value: bool) {
    if depthTestIndex + 1 >= 16 {
        Fatal(
            b"RenderState_PushDepthTest: Maximum state stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    depthTestIndex += 1;
    depthTest[depthTestIndex as usize] = value;
    RenderState_SetDepthTest(value);
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushDepthWritable(value: bool) {
    if depthWritableIndex + 1 >= 16 {
        Fatal(
            b"RenderState_PushDepthWritable: Maximum state stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    depthWritableIndex += 1;
    depthWritable[depthWritableIndex as usize] = value;
    RenderState_SetDepthWritable(value);
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushWireframe(value: bool) {
    if wireframeIndex + 1 >= 16 {
        Fatal(
            b"RenderState_PushWireframe: Maximum state stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    wireframeIndex += 1;
    wireframe[wireframeIndex as usize] = value;
    RenderState_SetWireframe(value);
}
