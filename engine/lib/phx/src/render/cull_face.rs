use super::*;

pub type CullFace = i32;

pub const CULL_FACE_NONE: CullFace = 0;
pub const CULL_FACE_BACK: CullFace = 1;
pub const CULL_FACE_FRONT: CullFace = 2;

#[no_mangle]
pub unsafe extern "C" fn CullFace_Pop() {
    RenderState_PopCullFace();
}

#[no_mangle]
pub unsafe extern "C" fn CullFace_Push(cullFace: CullFace) {
    RenderState_PushCullFace(cullFace);
}

#[no_mangle]
pub unsafe extern "C" fn CullFace_PushNone() {
    RenderState_PushCullFace(CULL_FACE_NONE);
}

#[no_mangle]
pub unsafe extern "C" fn CullFace_PushBack() {
    RenderState_PushCullFace(CULL_FACE_BACK);
}

#[no_mangle]
pub unsafe extern "C" fn CullFace_PushFront() {
    RenderState_PushCullFace(CULL_FACE_FRONT);
}
