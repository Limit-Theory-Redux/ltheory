use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    fn Fatal(_: cstr, _: ...);
    fn glBlendFunc(sfactor: GLenum, dfactor: GLenum);
    fn glCullFace(mode: GLenum);
    fn glDepthMask(flag: GLboolean);
    fn glDisable(cap: GLenum);
    fn glEnable(cap: GLenum);
    fn glPolygonMode(face: GLenum, mode: GLenum);
    static mut __glewBlendFuncSeparate: PFNGLBLENDFUNCSEPARATEPROC;
}
pub type int32_t = libc::c_int;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type BlendMode = int32;
pub type CullFace = int32;
pub type GLenum = libc::c_uint;
pub type GLboolean = libc::c_uchar;
pub type PFNGLBLENDFUNCSEPARATEPROC = Option::<
    unsafe extern "C" fn(GLenum, GLenum, GLenum, GLenum) -> (),
>;
static mut wireframe: [bool; 16] = [false; 16];
static mut wireframeIndex: libc::c_int = -(1 as libc::c_int);
static mut depthTest: [bool; 16] = [false; 16];
static mut depthTestIndex: libc::c_int = -(1 as libc::c_int);
static mut blendModeIndex: libc::c_int = -(1 as libc::c_int);
static mut blendMode: [BlendMode; 16] = [0; 16];
static mut cullFace: [CullFace; 16] = [0; 16];
static mut cullFaceIndex: libc::c_int = -(1 as libc::c_int);
static mut depthWritable: [bool; 16] = [false; 16];
static mut depthWritableIndex: libc::c_int = -(1 as libc::c_int);
#[inline]
unsafe extern "C" fn RenderState_SetBlendMode(mut mode: BlendMode) {
    match mode {
        0 => {
            __glewBlendFuncSeparate
                .expect(
                    "non-null function pointer",
                )(
                1 as libc::c_int as GLenum,
                1 as libc::c_int as GLenum,
                1 as libc::c_int as GLenum,
                1 as libc::c_int as GLenum,
            );
            return;
        }
        1 => {
            __glewBlendFuncSeparate
                .expect(
                    "non-null function pointer",
                )(
                0x302 as libc::c_int as GLenum,
                0x303 as libc::c_int as GLenum,
                1 as libc::c_int as GLenum,
                0x303 as libc::c_int as GLenum,
            );
            return;
        }
        3 => {
            glBlendFunc(1 as libc::c_int as GLenum, 0x303 as libc::c_int as GLenum);
            return;
        }
        2 => {
            glBlendFunc(1 as libc::c_int as GLenum, 0 as libc::c_int as GLenum);
            return;
        }
        _ => {}
    };
}
#[inline]
unsafe extern "C" fn RenderState_SetCullFace(mut mode: CullFace) {
    match mode {
        0 => {
            glDisable(0xb44 as libc::c_int as GLenum);
            return;
        }
        1 => {
            glEnable(0xb44 as libc::c_int as GLenum);
            glCullFace(0x405 as libc::c_int as GLenum);
            return;
        }
        2 => {
            glEnable(0xb44 as libc::c_int as GLenum);
            glCullFace(0x404 as libc::c_int as GLenum);
            return;
        }
        _ => {}
    };
}
#[inline]
unsafe extern "C" fn RenderState_SetDepthTest(mut enabled: bool) {
    if enabled {
        glEnable(0xb71 as libc::c_int as GLenum);
    } else {
        glDisable(0xb71 as libc::c_int as GLenum);
    };
}
#[inline]
unsafe extern "C" fn RenderState_SetDepthWritable(mut enabled: bool) {
    glDepthMask(enabled as GLboolean);
}
#[inline]
unsafe extern "C" fn RenderState_SetWireframe(mut enabled: bool) {
    if enabled {
        glPolygonMode(0x408 as libc::c_int as GLenum, 0x1b01 as libc::c_int as GLenum);
    } else {
        glPolygonMode(0x408 as libc::c_int as GLenum, 0x1b02 as libc::c_int as GLenum);
    };
}
#[no_mangle]
pub unsafe extern "C" fn RenderState_PushAllDefaults() {
    RenderState_PushBlendMode(2 as libc::c_int);
    RenderState_PushCullFace(0 as libc::c_int);
    RenderState_PushDepthTest(0 as libc::c_int != 0);
    RenderState_PushDepthWritable(1 as libc::c_int != 0);
    RenderState_PushWireframe(0 as libc::c_int != 0);
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
    if blendModeIndex < 0 as libc::c_int {
        Fatal(
            b"RenderState_PopBlendMode: Attempting to pop an empty state stack\0"
                as *const u8 as *const libc::c_char,
        );
    }
    blendModeIndex -= 1;
    if blendModeIndex >= 0 as libc::c_int {
        RenderState_SetBlendMode(blendMode[blendModeIndex as usize]);
    }
}
#[no_mangle]
pub unsafe extern "C" fn RenderState_PopWireframe() {
    if wireframeIndex < 0 as libc::c_int {
        Fatal(
            b"RenderState_PopWireframe: Attempting to pop an empty state stack\0"
                as *const u8 as *const libc::c_char,
        );
    }
    wireframeIndex -= 1;
    if wireframeIndex >= 0 as libc::c_int {
        RenderState_SetWireframe(wireframe[wireframeIndex as usize]);
    }
}
#[no_mangle]
pub unsafe extern "C" fn RenderState_PushBlendMode(mut value: BlendMode) {
    if blendModeIndex + 1 as libc::c_int >= 16 as libc::c_int {
        Fatal(
            b"RenderState_PushBlendMode: Maximum state stack depth exceeded\0"
                as *const u8 as *const libc::c_char,
        );
    }
    blendModeIndex += 1;
    blendMode[blendModeIndex as usize] = value;
    RenderState_SetBlendMode(value);
}
#[no_mangle]
pub unsafe extern "C" fn RenderState_PopDepthTest() {
    if depthTestIndex < 0 as libc::c_int {
        Fatal(
            b"RenderState_PopDepthTest: Attempting to pop an empty state stack\0"
                as *const u8 as *const libc::c_char,
        );
    }
    depthTestIndex -= 1;
    if depthTestIndex >= 0 as libc::c_int {
        RenderState_SetDepthTest(depthTest[depthTestIndex as usize]);
    }
}
#[no_mangle]
pub unsafe extern "C" fn RenderState_PopCullFace() {
    if cullFaceIndex < 0 as libc::c_int {
        Fatal(
            b"RenderState_PopCullFace: Attempting to pop an empty state stack\0"
                as *const u8 as *const libc::c_char,
        );
    }
    cullFaceIndex -= 1;
    if cullFaceIndex >= 0 as libc::c_int {
        RenderState_SetCullFace(cullFace[cullFaceIndex as usize]);
    }
}
#[no_mangle]
pub unsafe extern "C" fn RenderState_PopDepthWritable() {
    if depthWritableIndex < 0 as libc::c_int {
        Fatal(
            b"RenderState_PopDepthWritable: Attempting to pop an empty state stack\0"
                as *const u8 as *const libc::c_char,
        );
    }
    depthWritableIndex -= 1;
    if depthWritableIndex >= 0 as libc::c_int {
        RenderState_SetDepthWritable(depthWritable[depthWritableIndex as usize]);
    }
}
#[no_mangle]
pub unsafe extern "C" fn RenderState_PushCullFace(mut value: CullFace) {
    if cullFaceIndex + 1 as libc::c_int >= 16 as libc::c_int {
        Fatal(
            b"RenderState_PushCullFace: Maximum state stack depth exceeded\0"
                as *const u8 as *const libc::c_char,
        );
    }
    cullFaceIndex += 1;
    cullFace[cullFaceIndex as usize] = value;
    RenderState_SetCullFace(value);
}
#[no_mangle]
pub unsafe extern "C" fn RenderState_PushDepthTest(mut value: bool) {
    if depthTestIndex + 1 as libc::c_int >= 16 as libc::c_int {
        Fatal(
            b"RenderState_PushDepthTest: Maximum state stack depth exceeded\0"
                as *const u8 as *const libc::c_char,
        );
    }
    depthTestIndex += 1;
    depthTest[depthTestIndex as usize] = value;
    RenderState_SetDepthTest(value);
}
#[no_mangle]
pub unsafe extern "C" fn RenderState_PushDepthWritable(mut value: bool) {
    if depthWritableIndex + 1 as libc::c_int >= 16 as libc::c_int {
        Fatal(
            b"RenderState_PushDepthWritable: Maximum state stack depth exceeded\0"
                as *const u8 as *const libc::c_char,
        );
    }
    depthWritableIndex += 1;
    depthWritable[depthWritableIndex as usize] = value;
    RenderState_SetDepthWritable(value);
}
#[no_mangle]
pub unsafe extern "C" fn RenderState_PushWireframe(mut value: bool) {
    if wireframeIndex + 1 as libc::c_int >= 16 as libc::c_int {
        Fatal(
            b"RenderState_PushWireframe: Maximum state stack depth exceeded\0"
                as *const u8 as *const libc::c_char,
        );
    }
    wireframeIndex += 1;
    wireframe[wireframeIndex as usize] = value;
    RenderState_SetWireframe(value);
}
