use super::*;
use crate::internal::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Device {
    pub ty: DeviceType,
    pub id: u32,
}

impl Device {
    pub fn to_string(&self) -> String {
        format!("{} ({})", device_type_to_string(self.ty), self.id)
    }
}

#[no_mangle]
pub extern "C" fn Device_Equal(a: &Device, b: &Device) -> bool {
    a.ty == b.ty && a.id == b.id
}

#[no_mangle]
pub extern "C" fn Device_ToString(this: &Device) -> *const libc::c_char {
    static_string!(this.to_string())
}
