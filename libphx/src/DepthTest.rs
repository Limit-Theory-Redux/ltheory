use ::libc;
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
    RenderState_PushDepthTest(0 as libc::c_int != 0);
}
#[no_mangle]
pub unsafe extern "C" fn DepthTest_PushEnabled() {
    RenderState_PushDepthTest(1 as libc::c_int != 0);
}
