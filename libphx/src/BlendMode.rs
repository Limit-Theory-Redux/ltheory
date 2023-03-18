use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use crate::RenderState::*;
use libc;

pub type BlendMode = i32;

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
    RenderState_PushBlendMode(0);
}

#[no_mangle]
pub unsafe extern "C" fn BlendMode_PushAlpha() {
    RenderState_PushBlendMode(1);
}

#[no_mangle]
pub unsafe extern "C" fn BlendMode_PushDisabled() {
    RenderState_PushBlendMode(2);
}

#[no_mangle]
pub unsafe extern "C" fn BlendMode_PushPreMultAlpha() {
    RenderState_PushBlendMode(3);
}
