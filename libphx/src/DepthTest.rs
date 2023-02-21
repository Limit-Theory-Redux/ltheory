use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    fn RenderState_PushDepthTest(_: bool);
    fn RenderState_PopDepthTest();
}
#[no_mangle]
pub unsafe extern "C" fn DepthTest_Pop() {
    RenderState_PopDepthTest();
}
#[no_mangle]
pub unsafe extern "C" fn DepthTest_Push(mut depthTest: bool) {
    RenderState_PushDepthTest(depthTest);
}
#[no_mangle]
pub unsafe extern "C" fn DepthTest_PushDisabled() {
    RenderState_PushDepthTest(0 as i32 != 0);
}
#[no_mangle]
pub unsafe extern "C" fn DepthTest_PushEnabled() {
    RenderState_PushDepthTest(1 as i32 != 0);
}
