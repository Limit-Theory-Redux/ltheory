use luajit_ffi_gen::luajit_ffi;

#[allow(dead_code)]
mod helpers;
use helpers::*;

#[derive(Default)]
pub struct ArraysTest {
    val_array_primitive: Vec<u32>,
    val_array_managed: Vec<ManagedData>,
}

#[luajit_ffi(gen_dir = "./tests/out/ffi_gen", meta_dir = "./tests/out/ffi_meta")]
impl ArraysTest {
    // Primitive slices.

    pub fn set_primitive_slice(&mut self, data: &[u32]) {
        self.val_array_primitive = data.to_vec();
    }

    pub fn get_primitive_slice(&self, out: &mut [u32]) {
        let len = std::cmp::min(out.len(), self.val_array_primitive.len());
        out[..len].copy_from_slice(&self.val_array_primitive[..len]);
    }

    // Managed custom slices.

    pub fn set_managed_slice(&mut self, data: &[ManagedData]) {
        self.val_array_managed = data.to_vec();
    }

    pub fn get_managed_slice(&self, out: &mut [ManagedData]) {
        let len = std::cmp::min(out.len(), self.val_array_managed.len());
        out[..len].clone_from_slice(&self.val_array_managed[..len]);
    }

    // Copyable custom slices.

    // String slices.

    // Primitive arrays.

    pub fn move_primitive_array(&mut self, data: [u32; 3]) {
        self.val_array_primitive = data.to_vec();
    }

    pub fn set_primitive_array(&mut self, data: &[u32; 3]) {
        self.val_array_primitive = data.to_vec();
    }

    pub fn get_primitive_array(&self, out: &mut [u32; 3]) {
        let len = std::cmp::min(out.len(), self.val_array_primitive.len());
        out[..len].copy_from_slice(&self.val_array_primitive[..len]);
    }

    // Managed custom arrays.

    pub fn move_managed_array(&mut self, data: [ManagedData; 3]) {
        self.val_array_managed = data.to_vec();
    }

    pub fn set_managed_array(&mut self, data: &[ManagedData; 3]) {
        self.val_array_managed = data.to_vec();
    }

    pub fn get_managed_array(&self, out: &mut [ManagedData; 3]) {
        let len = std::cmp::min(out.len(), self.val_array_managed.len());
        out[..len].clone_from_slice(&self.val_array_managed[..len]);
    }

    // Copyable custom arrays.

    // String arrays.
}

/// # Safety
/// this is a test
#[no_mangle]
pub unsafe extern "C" fn ArraysTest_GetPrimitiveArrays2(
    this: &ArraysTest,
    out: *mut u32,
    out_size: usize,
) {
    this.get_primitive_array({
        assert!(out.is_null());
        assert_eq!(out_size, 3, "incorrect number of elements for array");
        std::slice::from_raw_parts_mut(out, 3usize)
            .try_into()
            .unwrap()
    });
}

#[test]
fn test_primitive_array() {
    let mut t = ArraysTest::default();

    let data = vec![1, 2, 3];
    let mut data_read = vec![0; 3];

    let data_array = [3, 4, 5];
    let data_array2 = [6, 7, 8];
    let mut data_array_read = [0; 3];

    unsafe {
        ArraysTest_SetPrimitiveSlice(&mut t, data.as_ptr(), data.len());
        assert_eq!(data, t.val_array_primitive);
        ArraysTest_GetPrimitiveSlice(&t, data_read.as_mut_ptr(), data_read.len());
        assert_eq!(data, data_read);
        ArraysTest_MovePrimitiveArray(&mut t, data_array.as_ptr(), 3);
        assert_eq!(data_array.as_slice(), t.val_array_primitive);
        ArraysTest_SetPrimitiveArray(&mut t, data_array2.as_ptr(), 3);
        assert_eq!(data_array2.as_slice(), t.val_array_primitive);
        ArraysTest_GetPrimitiveArray(&t, data_array_read.as_mut_ptr(), 3);
        assert_eq!(data_array2, data_array_read);
    }
}

#[test]
fn test_managed_array() {
    let mut t = ArraysTest::default();

    let data = vec![
        ManagedData::new(3),
        ManagedData::new(4),
        ManagedData::new(5),
    ];
    let mut data_read = vec![ManagedData::new(0); 3];

    let data_array = [
        ManagedData::new(3),
        ManagedData::new(4),
        ManagedData::new(5),
    ];
    let data_array2 = [
        ManagedData::new(6),
        ManagedData::new(7),
        ManagedData::new(8),
    ];
    let mut data_array_read = [
        ManagedData::new(0),
        ManagedData::new(0),
        ManagedData::new(0),
    ];

    unsafe {
        ArraysTest_SetManagedSlice(&mut t, data.as_ptr(), data.len());
        assert_eq!(data, t.val_array_managed);
        ArraysTest_GetManagedSlice(&t, data_read.as_mut_ptr(), data_read.len());
        assert_eq!(data, data_read);
        ArraysTest_MoveManagedArray(&mut t, data_array.as_ptr(), 3);
        assert_eq!(data_array.as_slice(), t.val_array_managed);
        ArraysTest_SetManagedArray(&mut t, data_array2.as_ptr(), 3);
        assert_eq!(data_array2.as_slice(), t.val_array_managed);
        ArraysTest_GetManagedArray(&t, data_array_read.as_mut_ptr(), 3);
        assert_eq!(data_array2, data_array_read);
    }
}

#[test]
#[should_panic]
fn test_null_slice_should_panic() {
    let mut t = ArraysTest::default();
    unsafe {
        ArraysTest_SetPrimitiveSlice(&mut t, std::ptr::null(), 3);
    }
}

#[test]
#[should_panic]
fn test_zero_length_slice_should_panic() {
    let mut t = ArraysTest::default();
    let data = [0; 3];
    unsafe {
        ArraysTest_SetPrimitiveSlice(&mut t, data.as_ptr(), 0);
    }
}

#[test]
#[should_panic]
fn test_null_array_should_panic() {
    let mut t = ArraysTest::default();
    unsafe {
        ArraysTest_MovePrimitiveArray(&mut t, std::ptr::null(), 3);
    }
}

#[test]
#[should_panic]
fn test_move_primitive_array_should_panic() {
    let mut t = ArraysTest::default();
    let data = [0; 3];
    unsafe {
        ArraysTest_MovePrimitiveArray(&mut t, data.as_ptr(), 2);
    }
}

#[test]
#[should_panic]
fn test_ref_primitive_array_should_panic() {
    let mut t = ArraysTest::default();
    let data = [0; 3];
    unsafe {
        ArraysTest_SetPrimitiveArray(&mut t, data.as_ptr(), 4);
    }
}

#[test]
#[should_panic]
fn test_mut_ref_primitive_array_should_panic() {
    let t = ArraysTest::default();
    let mut data = [0; 3];
    unsafe {
        ArraysTest_GetPrimitiveArray(&t, data.as_mut_ptr(), 5);
    }
}

#[test]
#[should_panic]
fn test_move_managed_array_should_panic() {
    let mut t = ArraysTest::default();
    let data = [
        ManagedData::new(0),
        ManagedData::new(0),
        ManagedData::new(0),
    ];
    unsafe {
        ArraysTest_MoveManagedArray(&mut t, data.as_ptr(), 2);
    }
}

#[test]
#[should_panic]
fn test_ref_managed_array_should_panic() {
    let mut t = ArraysTest::default();
    let data = [
        ManagedData::new(0),
        ManagedData::new(0),
        ManagedData::new(0),
    ];
    unsafe {
        ArraysTest_SetManagedArray(&mut t, data.as_ptr(), 4);
    }
}

#[test]
#[should_panic]
fn test_mut_ref_managed_array_should_panic() {
    let t = ArraysTest::default();
    let mut data = [
        ManagedData::new(0),
        ManagedData::new(0),
        ManagedData::new(0),
    ];
    unsafe {
        ArraysTest_GetManagedArray(&t, data.as_mut_ptr(), 5);
    }
}
