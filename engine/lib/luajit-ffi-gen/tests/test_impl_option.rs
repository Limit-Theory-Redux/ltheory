use luajit_ffi_gen::luajit_ffi;

#[allow(dead_code)]
mod helpers;
use helpers::*;

#[derive(Default)]
pub struct OptionTest {
    val_primitive: u32,
    val_managed: ManagedData,
    val_copyable: CopyableData,
    val_str: String,
}

#[luajit_ffi(gen_dir = "./tests/out/ffi_gen", meta_dir = "./tests/out/ffi_meta")]
impl OptionTest {
    // Primitve types.

    pub fn set_primitive(&mut self, val: Option<u32>) {
        if let Some(val) = val {
            self.val_primitive = val;
        }
    }

    pub fn set_primitive_ref(&mut self, val: Option<&u32>) {
        if let Some(val) = val {
            self.val_primitive = *val;
        }
    }

    pub fn set_primitive_mut(&mut self, val: Option<&mut u32>) {
        if let Some(val) = val {
            self.val_primitive = *val;
            *val = 0;
        }
    }

    pub fn get_primitive(&self) -> Option<u32> {
        if self.val_primitive > 0 {
            Some(self.val_primitive)
        } else {
            None
        }
    }

    pub fn get_primitive_ref(&self) -> Option<&u32> {
        if self.val_primitive > 0 {
            Some(&self.val_primitive)
        } else {
            None
        }
    }

    pub fn get_primitive_mut(&mut self) -> Option<&mut u32> {
        if self.val_primitive > 0 {
            Some(&mut self.val_primitive)
        } else {
            None
        }
    }

    // Non-copyable custom type.

    pub fn set_managed(&mut self, val: Option<ManagedData>) {
        if let Some(val) = val {
            self.val_managed = val;
        }
    }

    pub fn set_managed_ref(&mut self, val: Option<&ManagedData>) {
        if let Some(val) = val {
            self.val_managed = val.clone();
        }
    }

    pub fn set_managed_mut(&mut self, val: Option<&mut ManagedData>) {
        if let Some(val) = val {
            self.val_managed = val.clone();
            *val = ManagedData::default();
        }
    }

    pub fn get_managed(&self) -> Option<ManagedData> {
        if self.val_managed.val > 0 {
            Some(self.val_managed.clone())
        } else {
            None
        }
    }

    pub fn get_managed_ref(&self) -> Option<&ManagedData> {
        if self.val_managed.val > 0 {
            Some(&self.val_managed)
        } else {
            None
        }
    }

    pub fn get_managed_mut(&mut self) -> Option<&mut ManagedData> {
        if self.val_managed.val > 0 {
            Some(&mut self.val_managed)
        } else {
            None
        }
    }

    // Copyable custom type.

    pub fn set_copyable(&mut self, val: Option<CopyableData>) {
        if let Some(val) = val {
            self.val_copyable = val;
        }
    }

    pub fn set_copyable_ref(&mut self, val: Option<&CopyableData>) {
        if let Some(val) = val {
            self.val_copyable = val.clone();
        }
    }

    pub fn set_copyable_mut(&mut self, val: Option<&mut CopyableData>) {
        if let Some(val) = val {
            self.val_copyable = val.clone();
            *val = CopyableData::default();
        }
    }

    pub fn get_copyable(&self) -> Option<CopyableData> {
        if self.val_copyable.val > 0 {
            Some(self.val_copyable.clone())
        } else {
            None
        }
    }

    pub fn get_copyable_ref(&self) -> Option<&CopyableData> {
        if self.val_copyable.val > 0 {
            Some(&self.val_copyable)
        } else {
            None
        }
    }

    pub fn get_copyable_mut(&mut self) -> Option<&mut CopyableData> {
        if self.val_copyable.val > 0 {
            Some(&mut self.val_copyable)
        } else {
            None
        }
    }

    // Strings

    pub fn set_str(&mut self, val: Option<&str>) {
        if let Some(val) = val {
            self.val_str = val.to_string();
        }
    }

    pub fn set_string(&mut self, val: Option<String>) {
        if let Some(val) = val {
            self.val_str = val;
        }
    }

    pub fn set_string_ref(&mut self, val: Option<&String>) {
        if let Some(val) = val {
            self.val_str = val.clone();
        }
    }

    pub fn get_str(&self) -> Option<&str> {
        if self.val_str.len() > 0 {
            Some(self.val_str.as_str())
        } else {
            None
        }
    }

    pub fn get_string(&self) -> Option<String> {
        if self.val_str.len() > 0 {
            Some(self.val_str.clone())
        } else {
            None
        }
    }

    pub fn get_string_ref(&self) -> Option<&String> {
        if self.val_str.len() > 0 {
            Some(&self.val_str)
        } else {
            None
        }
    }
}

#[test]
fn test_primitives() {
    let mut t = OptionTest::default();

    unsafe {
        OptionTest_SetPrimitive(&mut t, None);
        assert_eq!(OptionTest_GetPrimitive(&t), None);

        let tmp = Some(33);
        OptionTest_SetPrimitive(&mut t, tmp.as_ref());
        assert_eq!(OptionTest_GetPrimitive(&t).cloned(), Some(33));
        assert_eq!(t.val_primitive, 33);

        OptionTest_SetPrimitiveRef(&mut t, None);
        assert_eq!(OptionTest_GetPrimitiveRef(&t).cloned(), Some(33));

        let tmp = Some(44);
        OptionTest_SetPrimitiveRef(&mut t, tmp.as_ref());
        let inner_ref = OptionTest_GetPrimitiveRef(&t);
        assert_eq!(inner_ref.cloned(), Some(44));
        assert_eq!(t.val_primitive, 44);

        OptionTest_SetPrimitiveMut(&mut t, None);
        assert_eq!(OptionTest_GetPrimitiveMut(&mut t).cloned(), Some(44));

        let mut tmp = Some(55);
        OptionTest_SetPrimitiveMut(&mut t, tmp.as_mut());
        assert_eq!(tmp, Some(0));
        assert_eq!(t.val_primitive, 55);
        assert_eq!(OptionTest_GetPrimitiveMut(&mut t).cloned(), Some(55));

        // Mutate value through the mutable ref
        {
            if let Some(val) = OptionTest_GetPrimitiveMut(&mut t) {
                *val = 56
            };
        }
        assert_eq!(t.val_primitive, 56);
    }
}

#[test]
fn test_custom_managed() {
    let mut t = OptionTest::default();

    unsafe {
        OptionTest_SetManaged(&mut t, None);
        assert_eq!(OptionTest_GetManaged(&t), None);

        let tmp = Some(Box::new(ManagedData::new(33)));
        OptionTest_SetManaged(&mut t, tmp);
        assert_eq!(
            OptionTest_GetManaged(&t),
            Some(Box::new(ManagedData::new(33)))
        );
        assert_eq!(t.val_managed.val, 33);

        OptionTest_SetManagedRef(&mut t, None);
        assert_eq!(
            OptionTest_GetManagedRef(&t).cloned(),
            Some(ManagedData::new(33))
        );

        let tmp = Some(ManagedData::new(44));
        OptionTest_SetManagedRef(&mut t, tmp.as_ref());
        let inner_ref = OptionTest_GetManagedRef(&t);
        assert_eq!(inner_ref.cloned(), Some(ManagedData::new(44)));
        assert_eq!(t.val_managed.val, 44);

        OptionTest_SetManagedMut(&mut t, None);
        assert_eq!(
            OptionTest_GetManagedMut(&mut t).cloned(),
            Some(ManagedData::new(44))
        );

        let mut tmp = Some(ManagedData::new(55));
        OptionTest_SetManagedMut(&mut t, tmp.as_mut());
        assert_eq!(tmp, Some(ManagedData::default()));
        assert_eq!(t.val_managed.val, 55);
        assert_eq!(
            OptionTest_GetManagedMut(&mut t).cloned(),
            Some(ManagedData::new(55))
        );

        // Mutate value through the mutable ref
        {
            if let Some(val) = OptionTest_GetManagedMut(&mut t) {
                val.val = 56
            };
        }
        assert_eq!(t.val_managed.val, 56);
    }
}

#[test]
fn test_custom_copyable() {
    let mut t = OptionTest::default();

    unsafe {
        OptionTest_SetCopyable(&mut t, None);
        assert_eq!(OptionTest_GetCopyable(&t), None);

        let tmp = Some(CopyableData::new(33));
        OptionTest_SetCopyable(&mut t, tmp.as_ref());
        assert_eq!(
            OptionTest_GetCopyable(&t).cloned(),
            Some(CopyableData::new(33))
        );
        assert_eq!(t.val_copyable.val, 33);

        OptionTest_SetCopyableRef(&mut t, None);
        assert_eq!(
            OptionTest_GetCopyableRef(&t).cloned(),
            Some(CopyableData::new(33))
        );

        let tmp = Some(CopyableData::new(44));
        OptionTest_SetCopyableRef(&mut t, tmp.as_ref());
        let inner_ref = OptionTest_GetCopyableRef(&t);
        assert_eq!(inner_ref.cloned(), Some(CopyableData::new(44)));
        assert_eq!(t.val_copyable.val, 44);

        OptionTest_SetCopyableMut(&mut t, None);
        assert_eq!(
            OptionTest_GetCopyableMut(&mut t).cloned(),
            Some(CopyableData::new(44))
        );

        let mut tmp = Some(CopyableData::new(55));
        OptionTest_SetCopyableMut(&mut t, tmp.as_mut());
        assert_eq!(tmp, Some(CopyableData::default()));
        assert_eq!(t.val_copyable.val, 55);
        assert_eq!(
            OptionTest_GetCopyableMut(&mut t).cloned(),
            Some(CopyableData::new(55))
        );

        // Mutate value through the mutable ref
        {
            if let Some(val) = OptionTest_GetCopyableMut(&mut t) {
                val.val = 56
            };
        }
        assert_eq!(t.val_copyable.val, 56);
    }
}

#[test]
fn test_strings() {
    use std::ffi::CString;

    use internal::ConvertIntoString;

    let mut t = OptionTest::default();

    let str_data1 = CString::new("hello").unwrap();
    let str_data2 = CString::new("world").unwrap();
    let str_data3 = CString::new("test").unwrap();

    unsafe {
        let data = OptionTest_GetStr(&t);
        assert_eq!(data, std::ptr::null());

        OptionTest_SetStr(&mut t, std::ptr::null());
        assert_eq!(t.val_str, "");
        assert_eq!(OptionTest_GetStr(&t), std::ptr::null());
        assert_eq!(OptionTest_GetString(&t), std::ptr::null());
        assert_eq!(OptionTest_GetStringRef(&t), std::ptr::null());

        OptionTest_SetStr(&mut t, str_data1.as_ptr());
        assert_eq!(t.val_str, str_data1.to_str().unwrap());

        let data = OptionTest_GetStr(&mut t);
        assert_eq!(t.val_str, data.as_str());

        OptionTest_SetString(&mut t, std::ptr::null());
        assert_eq!(t.val_str, data.as_str());

        OptionTest_SetString(&mut t, str_data2.as_ptr());
        assert_eq!(t.val_str, str_data2.to_str().unwrap());

        let data = OptionTest_GetString(&mut t);
        assert_eq!(t.val_str, data.as_str());

        OptionTest_SetStringRef(&mut t, std::ptr::null());
        assert_eq!(t.val_str, data.as_str());

        OptionTest_SetStringRef(&mut t, str_data3.as_ptr());
        assert_eq!(t.val_str, str_data3.to_str().unwrap());

        let data = OptionTest_GetStringRef(&mut t);
        assert_eq!(t.val_str, data.as_str());
    }
}
