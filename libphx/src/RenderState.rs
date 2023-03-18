use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use crate::GL::gl;
use libc;

extern "C" {
    static mut __glewBlendFuncSeparate: PFNGLBLENDFUNCSEPARATEPROC;
}
pub type BlendMode = i32;
pub type CullFace = i32;
pub type GLenum = u32;
pub type GLboolean = libc::c_uchar;
pub type PFNGLBLENDFUNCSEPARATEPROC =
    Option<unsafe extern "C" fn(GLenum, GLenum, GLenum, GLenum) -> ()>;

static mut wireframe: [bool; 16] = [false; 16];

static mut wireframeIndex: i32 = -1_i32;

static mut depthTest: [bool; 16] = [false; 16];

static mut depthTestIndex: i32 = -1_i32;

static mut blendModeIndex: i32 = -1_i32;

static mut blendMode: [BlendMode; 16] = [0; 16];

static mut cullFace: [CullFace; 16] = [0; 16];

static mut cullFaceIndex: i32 = -1_i32;

static mut depthWritable: [bool; 16] = [false; 16];

static mut depthWritableIndex: i32 = -1_i32;

#[inline]
unsafe extern "C" fn RenderState_SetBlendMode(mut mode: BlendMode) {
    match mode {
        0 => {
            __glewBlendFuncSeparate.expect("non-null function pointer")(
                1_i32 as GLenum,
                1_i32 as GLenum,
                1_i32 as GLenum,
                1_i32 as GLenum,
            );
        }
        1 => {
            __glewBlendFuncSeparate.expect("non-null function pointer")(
                0x302_i32 as GLenum,
                0x303_i32 as GLenum,
                1_i32 as GLenum,
                0x303_i32 as GLenum,
            );
        }
        3 => {
            gl::BlendFunc(1_i32 as GLenum, 0x303_i32 as GLenum);
        }
        2 => {
            gl::BlendFunc(1_i32 as GLenum, 0_i32 as GLenum);
        }
        _ => {}
    }
}

#[inline]
unsafe extern "C" fn RenderState_SetCullFace(mut mode: CullFace) {
    match mode {
        0 => {
            gl::Disable(0xb44_i32 as GLenum);
        }
        1 => {
            gl::Enable(0xb44_i32 as GLenum);
            gl::CullFace(0x405_i32 as GLenum);
        }
        2 => {
            gl::Enable(0xb44_i32 as GLenum);
            gl::CullFace(0x404_i32 as GLenum);
        }
        _ => {}
    }
}

#[inline]
unsafe extern "C" fn RenderState_SetDepthTest(mut enabled: bool) {
    if enabled {
        gl::Enable(0xb71_i32 as GLenum);
    } else {
        gl::Disable(0xb71_i32 as GLenum);
    };
}

#[inline]
unsafe extern "C" fn RenderState_SetDepthWritable(mut enabled: bool) {
    gl::DepthMask(enabled as GLboolean);
}

#[inline]
unsafe extern "C" fn RenderState_SetWireframe(mut enabled: bool) {
    if enabled {
        gl::PolygonMode(0x408_i32 as GLenum, 0x1b01_i32 as GLenum);
    } else {
        gl::PolygonMode(0x408_i32 as GLenum, 0x1b02_i32 as GLenum);
    };
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushAllDefaults() {
    RenderState_PushBlendMode(2_i32);
    RenderState_PushCullFace(0_i32);
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
    if blendModeIndex < 0_i32 {
        Fatal(
            b"RenderState_PopBlendMode: Attempting to pop an empty state stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    blendModeIndex -= 1;
    if blendModeIndex >= 0_i32 {
        RenderState_SetBlendMode(blendMode[blendModeIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopWireframe() {
    if wireframeIndex < 0_i32 {
        Fatal(
            b"RenderState_PopWireframe: Attempting to pop an empty state stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    wireframeIndex -= 1;
    if wireframeIndex >= 0_i32 {
        RenderState_SetWireframe(wireframe[wireframeIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushBlendMode(mut value: BlendMode) {
    if blendModeIndex + 1_i32 >= 16_i32 {
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
    if depthTestIndex < 0_i32 {
        Fatal(
            b"RenderState_PopDepthTest: Attempting to pop an empty state stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    depthTestIndex -= 1;
    if depthTestIndex >= 0_i32 {
        RenderState_SetDepthTest(depthTest[depthTestIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopCullFace() {
    if cullFaceIndex < 0_i32 {
        Fatal(
            b"RenderState_PopCullFace: Attempting to pop an empty state stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    cullFaceIndex -= 1;
    if cullFaceIndex >= 0_i32 {
        RenderState_SetCullFace(cullFace[cullFaceIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PopDepthWritable() {
    if depthWritableIndex < 0_i32 {
        Fatal(
            b"RenderState_PopDepthWritable: Attempting to pop an empty state stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    depthWritableIndex -= 1;
    if depthWritableIndex >= 0_i32 {
        RenderState_SetDepthWritable(depthWritable[depthWritableIndex as usize]);
    }
}

#[no_mangle]
pub unsafe extern "C" fn RenderState_PushCullFace(mut value: CullFace) {
    if cullFaceIndex + 1_i32 >= 16_i32 {
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
pub unsafe extern "C" fn RenderState_PushDepthTest(mut value: bool) {
    if depthTestIndex + 1_i32 >= 16_i32 {
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
pub unsafe extern "C" fn RenderState_PushDepthWritable(mut value: bool) {
    if depthWritableIndex + 1_i32 >= 16_i32 {
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
pub unsafe extern "C" fn RenderState_PushWireframe(mut value: bool) {
    if wireframeIndex + 1_i32 >= 16_i32 {
        Fatal(
            b"RenderState_PushWireframe: Maximum state stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    wireframeIndex += 1;
    wireframe[wireframeIndex as usize] = value;
    RenderState_SetWireframe(value);
}
