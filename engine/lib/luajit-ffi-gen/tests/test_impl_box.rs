use luajit_ffi_gen::luajit_ffi;

#[allow(dead_code)]
mod helpers;
use helpers::*;

#[derive(Default)]
pub struct BoxTest {
    // val_primitive: u32,
    val_managed: ManagedData,
    // val_copyable: CopyableData,
}

#[allow(clippy::boxed_local)]
#[luajit_ffi(gen_dir = "./tests/out/ffi_gen", meta_dir = "./tests/out/ffi_meta")]
impl BoxTest {
    // Primitve types.

    // pub fn set_primitive(&mut self, val: Box<u32>) {
    //     self.val_primitive = *val;
    // }

    // pub fn get_primitive(&self) -> Box<u32> {
    //     Box::new(self.val_primitive)
    // }

    // Managed custom type.

    pub fn set_managed(&mut self, val: Box<ManagedData>) {
        self.val_managed = *val;
    }

    pub fn get_managed(&self) -> Box<ManagedData> {
        Box::new(self.val_managed.clone())
    }

    // Copyable custom type.

    // pub fn set_copyable(&mut self, val: Box<CopyableData>) {
    //     self.val_copyable = *val;
    // }

    // pub fn get_copyable(&self) -> Box<CopyableData> {
    //     Box::new(self.val_copyable.clone())
    // }

    // Strings are not supported.
}

#[test]
fn test_custom_managed() {
    let mut t = BoxTest::default();

    unsafe {
        BoxTest_SetManaged(&mut t, Box::new(ManagedData::new(6)));
        assert_eq!(BoxTest_GetManaged(&t).val, 6);
    }
}
