use crate::phx::internal::ffi;


use crate::phx::DeviceType::*;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct Device {
    pub ty: DeviceType,
    pub id: u32,
}

#[no_mangle]
pub extern "C" fn Device_Equal(a: &Device, b: &Device) -> bool {
    a.ty == b.ty && a.id == b.id
}

#[no_mangle]
pub extern "C" fn Device_ToString(this: &mut Device) -> *const libc::c_char {
    ffi::StaticString!(format!(
        "{} ({})",
        ffi::PtrAsSlice(DeviceType_ToString(this.ty)),
        this.id
    ))
}
