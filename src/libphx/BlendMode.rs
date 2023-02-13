use ::libc;
use super::internal::Memory::*;

extern "C" {
    fn RenderState_PushBlendMode(_: BlendMode);
    fn RenderState_PopBlendMode();
}

pub type int32_t = libc::c_int;
pub type int32 = int32_t;
pub type BlendMode = int32;

#[no_mangle]
pub unsafe extern "C" fn BlendMode_Pop() {
    RenderState_PopBlendMode();
}
#[no_mangle]
pub unsafe extern "C" fn BlendMode_Push(mut blendMode: BlendMode) {
    RenderState_PushBlendMode(blendMode);
}
#[no_mangle]
pub unsafe extern "C" fn BlendMode_PushAdditive() {
    RenderState_PushBlendMode(0 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn BlendMode_PushAlpha() {
    RenderState_PushBlendMode(1 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn BlendMode_PushDisabled() {
    RenderState_PushBlendMode(2 as libc::c_int);
}
#[no_mangle]
pub unsafe extern "C" fn BlendMode_PushPreMultAlpha() {
    RenderState_PushBlendMode(3 as libc::c_int);
}
