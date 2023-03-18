use crate::internal::Memory::*;
use crate::RenderState::*;
use glam::Vec3;
use libc;

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
    RenderState_PushDepthTest(false);
}

#[no_mangle]
pub unsafe extern "C" fn DepthTest_PushEnabled() {
    RenderState_PushDepthTest(true);
}