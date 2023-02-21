use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    fn RenderState_PushCullFace(_: CullFace);
    fn RenderState_PopCullFace();
}
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
