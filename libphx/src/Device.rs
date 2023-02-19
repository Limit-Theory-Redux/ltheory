use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use crate::DeviceType::*;
extern "C" {
    fn DeviceType_ToString(_: DeviceType) -> cstr;
    fn snprintf(
        _: *mut libc::c_char,
        _: libc::size_t,
        _: *const libc::c_char,
        _: ...
    ) -> libc::c_int;
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Device {
    pub type_0: DeviceType,
    pub id: uint32,
}

#[no_mangle]
pub unsafe extern "C" fn Device_Equal(mut a: *mut Device, mut b: *mut Device) -> bool {
    return (*a).type_0 == (*b).type_0 && (*a).id == (*b).id;
}
#[no_mangle]
pub unsafe extern "C" fn Device_ToString(mut self_0: *mut Device) -> cstr {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    snprintf(
        buffer.as_mut_ptr(),
        (::core::mem::size_of::<[libc::c_char; 512]>())
            .wrapping_div(::core::mem::size_of::<libc::c_char>())
            as libc::c_int as libc::size_t,
        b"%s (%u)\0" as *const u8 as *const libc::c_char,
        DeviceType_ToString((*self_0).type_0),
        (*self_0).id,
    );
    return buffer.as_mut_ptr() as cstr;
}
