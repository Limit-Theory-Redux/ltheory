use crate::render_state::*;

pub type CullFace = i32;

pub const CullFace_None: CullFace = 0;
pub const CullFace_Back: CullFace = 1;
pub const CullFace_Front: CullFace = 2;

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
    RenderState_PushCullFace(CullFace_None);
}

#[no_mangle]
pub unsafe extern "C" fn CullFace_PushBack() {
    RenderState_PushCullFace(CullFace_Back);
}

#[no_mangle]
pub unsafe extern "C" fn CullFace_PushFront() {
    RenderState_PushCullFace(CullFace_Front);
}
