use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::{InputDeviceId, InputDeviceType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputDevice {
    pub ty: InputDeviceType,
    pub id: InputDeviceId,
}

#[luajit_ffi_gen::luajit_ffi(clone = true)]
impl InputDevice {
    pub fn equal(&self, other: &InputDevice) -> bool {
        self.ty == other.ty && self.id == other.id
    }

    #[bind(role = "to_string")]
    pub fn get_string(&self) -> String {
        let mut hasher = DefaultHasher::new();

        self.id.hash(&mut hasher);

        format!("{} ({:?})", self.ty, hasher.finish())
    }
}
