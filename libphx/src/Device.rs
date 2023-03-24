use crate::internal::Memory::*;
use crate::Common::*;
use crate::DeviceType::*;
use crate::Math::Vec3;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Device {
    pub type_0: DeviceType,
    pub id: u32,
}

#[no_mangle]
pub unsafe extern "C" fn Device_Equal(mut a: *mut Device, mut b: *mut Device) -> bool {
    (*a).type_0 == (*b).type_0 && (*a).id == (*b).id
}

#[no_mangle]
pub unsafe extern "C" fn Device_ToString(this: *mut Device) -> *const libc::c_char {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    libc::snprintf(
        buffer.as_mut_ptr(),
        buffer.len(),
        b"%s (%u)\0" as *const u8 as *const libc::c_char,
        DeviceType_ToString((*this).type_0),
        (*this).id,
    );
    buffer.as_mut_ptr() as *const libc::c_char
}
