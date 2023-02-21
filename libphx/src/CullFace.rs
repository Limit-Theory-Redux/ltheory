use crate::internal::Memory::*;
use crate::RenderState::*;
use glam::Vec3;
use libc;

pub type CullFace = i32;
#[no_mangle]
pub unsafe extern "C" fn CullFace_Pop() {
    RenderState_PopCullFace();
}
#[no_mangle]
pub unsafe extern "C" fn CullFace_Push(mut cullFace: CullFace) {
    RenderState_PushCullFace(cullFace);
}
#[no_mangle]
pub unsafe extern "C" fn CullFace_PushNone() {
    RenderState_PushCullFace(0 as i32);
}
#[no_mangle]
pub unsafe extern "C" fn CullFace_PushBack() {
    RenderState_PushCullFace(1 as i32);
}
#[no_mangle]
pub unsafe extern "C" fn CullFace_PushFront() {
    RenderState_PushCullFace(2 as i32);
}
