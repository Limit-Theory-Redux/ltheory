use super::*;

pub type BlendMode = i32;

pub const BlendMode_Additive: BlendMode = 0;
pub const BlendMode_Alpha: BlendMode = 1;
pub const BlendMode_Disabled: BlendMode = 2;
pub const BlendMode_PreMultAlpha: BlendMode = 3;

#[no_mangle]
pub unsafe extern "C" fn BlendMode_Pop() {
    RenderState_PopBlendMode();
}

#[no_mangle]
pub unsafe extern "C" fn BlendMode_Push(blendMode: BlendMode) {
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
