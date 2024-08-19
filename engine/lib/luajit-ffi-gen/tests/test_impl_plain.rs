use luajit_ffi_gen::luajit_ffi;

#[allow(dead_code)]
mod helpers;
use helpers::*;

pub struct SomeStruct;

#[luajit_ffi(
    name = "Renamed_Struct",
    gen_dir = "./tests/out/ffi_gen",
    meta_dir = "./tests/out/ffi_meta"
)]
impl SomeStruct {}

#[derive(Default)]
pub struct PlainTest {
    val_u32: u32,
    val_f32: f32,
    val_str: String,
    val_managed: ManagedData,
    val_copyable: CopyableData,
}

#[luajit_ffi(gen_dir = "./tests/out/ffi_gen", meta_dir = "./tests/out/ffi_meta")]
impl PlainTest {
    pub fn func1(&self) {}
    pub fn func2(&mut self) {}

    // Some functions we want to be private to Rust, but exposed to Lua.
    fn private_func1(&self) {}

    #[bind(name = "FUNC3")]
    pub fn func3() {}

    // Primitives.

    pub fn set_u32(&mut self, val: u32) {
        self.val_u32 = val;
    }

    pub fn get_u32(&self) -> u32 {
        self.val_u32
    }

    pub fn set_f32_ref(&mut self, val: &f32) {
        self.val_f32 = *val;
    }

    pub fn get_f32(&self) -> f32 {
        self.val_f32
    }

    // Managed custom type.

    pub fn set_managed(&mut self, val: ManagedData) {
        self.val_managed = val;
    }

    pub fn set_managed_ref(&mut self, val: &ManagedData) {
        self.val_managed = val.clone();
    }

    pub fn set_managed_mut(&mut self, val: &mut ManagedData) {
        self.val_managed = val.clone();
        *val = ManagedData::default();
    }

    pub fn get_managed(&self) -> ManagedData {
        self.val_managed.clone()
    }

    #[bind(out_param = true)]
    pub fn get_managed_via_out_param(&self) -> ManagedData {
        self.val_managed.clone()
    }

    pub fn get_managed_ref(&self) -> &ManagedData {
        &self.val_managed
    }

    pub fn get_managed_mut(&mut self) -> &mut ManagedData {
        &mut self.val_managed
    }

    // Copyable custom type.

    pub fn set_copyable(&mut self, val: CopyableData) {
        self.val_copyable = val;
    }

    pub fn set_copyable_ref(&mut self, val: &CopyableData) {
        self.val_copyable = *val;
    }

    pub fn set_copyable_mut(&mut self, val: &mut CopyableData) {
        self.val_copyable = *val;
        *val = CopyableData::default();
    }

    pub fn get_copyable(&self) -> CopyableData {
        self.val_copyable
    }

    #[bind(out_param = true)]
    pub fn get_copyable_via_out_param(&self) -> CopyableData {
        self.val_copyable
    }

    pub fn get_copyable_ref(&self) -> &CopyableData {
        &self.val_copyable
    }

    pub fn get_copyable_mut(&mut self) -> &mut CopyableData {
        &mut self.val_copyable
    }

    // String.

    pub fn set_str(&mut self, val: &str) {
        self.val_str = val.into();
    }

    pub fn set_string(&mut self, val: String) {
        self.val_str = val.into();
    }

    pub fn set_string_ref(&mut self, val: &String) {
        self.val_str = val.into();
    }

    pub fn get_str(&self) -> &str {
        &self.val_str
    }

    pub fn get_string(&self) -> String {
        self.val_str.clone()
    }

    pub fn get_string_ref(&self) -> &String {
        &self.val_str
    }
}

#[test]
fn test_functions() {
    let t = PlainTest::default();
    let mut t2 = PlainTest::default();

    t.func1();
    t2.func2();
    PlainTest::func3();

    unsafe {
        PlainTest_Func1(&t);
        PlainTest_Func2(&mut t2);
        PlainTest_PrivateFunc1(&t);
        PlainTest_FUNC3();
    }
}

#[test]
fn test_primitives() {
    let mut t = PlainTest::default();

    unsafe {
        PlainTest_SetU32(&mut t, 33);
        assert_eq!(PlainTest_GetU32(&t), 33);

        PlainTest_SetF32Ref(&mut t, 33.0);
        assert_eq!(PlainTest_GetF32(&t), 33.0);
    }
}

#[test]
fn test_managed() {
    let mut t = PlainTest::default();

    unsafe {
        PlainTest_SetManaged(&mut t, Box::new(ManagedData::new(5)));
        assert_eq!(t.val_managed.val, 5);

        let mut data = ManagedData::new(9);
        PlainTest_SetManagedMut(&mut t, &mut data);
        assert_eq!(PlainTest_GetManaged(&t).val, 9);

        let data = ManagedData::new(7);
        PlainTest_SetManagedRef(&mut t, &data);
        assert_eq!(PlainTest_GetManagedRef(&t).val, 7);

        PlainTest_SetManaged(&mut t, Box::new(ManagedData::new(11)));
        let mut result = ManagedData::default();
        PlainTest_GetManagedViaOutParam(&t, &mut result);
        assert_eq!(result.val, 11);

        let inner_ref = PlainTest_GetManagedMut(&mut t);
        inner_ref.val = 13;
        assert_eq!(t.val_managed.val, 13);
    }
}

#[test]
fn test_copyable() {
    let mut t = PlainTest::default();

    unsafe {
        PlainTest_SetCopyable(&mut t, CopyableData::new(5));
        assert_eq!(t.val_copyable.val, 5);

        let mut data = CopyableData::new(9);
        PlainTest_SetCopyableMut(&mut t, &mut data);
        assert_eq!(PlainTest_GetCopyable(&t).val, 9);

        let data = CopyableData::new(7);
        PlainTest_SetCopyableRef(&mut t, &data);
        assert_eq!(PlainTest_GetCopyableRef(&t).val, 7);

        PlainTest_SetCopyable(&mut t, CopyableData::new(11));
        let mut result = CopyableData::default();
        PlainTest_GetCopyableViaOutParam(&t, &mut result);
        assert_eq!(result.val, 11);

        // TODO: &mut Copyable should return a ref so we can mutate the internals.
        // let inner_ref = PlainTest_GetCopyableMut(&mut t);
        // inner_ref.val = 13;
        // assert_eq!(t.val_copyable.val, 13);
    }
}

#[test]
fn test_strings() {
    use std::ffi::CString;

    use internal::ConvertIntoString;

    let mut t = PlainTest::default();

    let str_data1 = CString::new("hello").unwrap();
    let str_data2 = CString::new("world").unwrap();
    let str_data3 = CString::new("test").unwrap();

    unsafe {
        let data = PlainTest_GetStr(&t);
        assert_eq!(t.val_str, data.as_str());
        assert_eq!("", data.as_str());

        PlainTest_SetStr(&mut t, str_data1.as_ptr());
        assert_eq!(t.val_str, str_data1.to_str().unwrap());

        let data = PlainTest_GetStr(&mut t);
        assert_eq!(t.val_str, data.as_str());

        PlainTest_SetString(&mut t, str_data2.as_ptr());
        assert_eq!(t.val_str, str_data2.to_str().unwrap());

        let data = PlainTest_GetString(&mut t);
        assert_eq!(t.val_str, data.as_str());

        PlainTest_SetStringRef(&mut t, str_data3.as_ptr());
        assert_eq!(t.val_str, str_data3.to_str().unwrap());

        let data = PlainTest_GetStringRef(&mut t);
        assert_eq!(t.val_str, data.as_str());
    }
}
