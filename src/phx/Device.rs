use crate::phx::internal::ffi;
use crate::phx::internal::Memory::*;
use crate::phx::Common::*;
use crate::phx::DeviceType::*;
use crate::phx::Math::Vec3;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Device {
    pub ty: DeviceType,
    pub id: u32,
}

#[no_mangle]
pub unsafe extern "C" fn Device_Equal(a: *mut Device, b: *mut Device) -> bool {
    (*a).ty == (*b).ty && (*a).id == (*b).id
}

#[no_mangle]
pub unsafe extern "C" fn Device_ToString(this: &mut Device) -> *const libc::c_char {
    ffi::StaticString!(format!(
        "{} ({})",
        ffi::PtrAsSlice(DeviceType_ToString(this.ty)),
        this.id
    ))
}
