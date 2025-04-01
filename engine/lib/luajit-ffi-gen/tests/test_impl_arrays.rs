#![allow(unsafe_code)] // TODO: remove

use luajit_ffi_gen::luajit_ffi;

#[allow(dead_code)]
mod helpers;
use helpers::*;

#[derive(Default)]
pub struct ArraysTest {
    val_array_primitive: Vec<u32>,
    val_array_managed: Vec<ManagedData>,
    val_array_copyable: Vec<CopyableData>,
    val_array_string: Vec<String>,
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

    pub fn set_copyable_slice(&mut self, data: &[CopyableData]) {
        self.val_array_copyable = data.to_vec();
    }

    pub fn get_copyable_slice(&self, out: &mut [CopyableData]) {
        let len = std::cmp::min(out.len(), self.val_array_copyable.len());
        out[..len].clone_from_slice(&self.val_array_copyable[..len]);
    }

    // String slices.

    pub fn set_str_slice(&mut self, data: &[&str]) {
        self.val_array_string = data.iter().map(|s| s.to_string()).collect();
    }

    pub fn set_string_slice(&mut self, data: &[String]) {
        self.val_array_string = data.to_vec();
    }

    // &mut [String] is not supported.

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

    pub fn move_copyable_array(&mut self, data: [CopyableData; 3]) {
        self.val_array_copyable = data.to_vec();
    }

    pub fn set_copyable_array(&mut self, data: &[CopyableData; 3]) {
        self.val_array_copyable = data.to_vec();
    }

    pub fn get_copyable_array(&self, out: &mut [CopyableData; 3]) {
        let len = std::cmp::min(out.len(), self.val_array_copyable.len());
        out[..len].clone_from_slice(&self.val_array_copyable[..len]);
    }

    // String arrays.

    pub fn move_str_array(&mut self, data: [&str; 3]) {
        self.val_array_string = data.iter().map(|s| s.to_string()).collect();
    }

    pub fn set_str_array(&mut self, data: &[&str; 3]) {
        self.val_array_string = data.iter().map(|s| s.to_string()).collect();
    }

    pub fn move_string_array(&mut self, data: [String; 3]) {
        self.val_array_string = data.to_vec();
    }

    pub fn set_string_array(&mut self, data: &[String; 3]) {
        self.val_array_string = data.to_vec();
    }

    // &mut [String; N] is not supported.
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
fn test_copyable_array() {
    let mut t = ArraysTest::default();

    let data = vec![
        CopyableData::new(3),
        CopyableData::new(4),
        CopyableData::new(5),
    ];
    let mut data_read = vec![CopyableData::new(0); 3];

    let data_array = [
        CopyableData::new(3),
        CopyableData::new(4),
        CopyableData::new(5),
    ];
    let data_array2 = [
        CopyableData::new(6),
        CopyableData::new(7),
        CopyableData::new(8),
    ];
    let mut data_array_read = [
        CopyableData::new(0),
        CopyableData::new(0),
        CopyableData::new(0),
    ];

    unsafe {
        ArraysTest_SetCopyableSlice(&mut t, data.as_ptr(), data.len());
        assert_eq!(data, t.val_array_copyable);
        ArraysTest_GetCopyableSlice(&t, data_read.as_mut_ptr(), data_read.len());
        assert_eq!(data, data_read);
        ArraysTest_MoveCopyableArray(&mut t, data_array.as_ptr(), 3);
        assert_eq!(data_array.as_slice(), t.val_array_copyable);
        ArraysTest_SetCopyableArray(&mut t, data_array2.as_ptr(), 3);
        assert_eq!(data_array2.as_slice(), t.val_array_copyable);
        ArraysTest_GetCopyableArray(&t, data_array_read.as_mut_ptr(), 3);
        assert_eq!(data_array2, data_array_read);
    }
}

#[test]
fn test_string_array() {
    use std::ffi::CString;

    let mut t = ArraysTest::default();

    let data1_str = ["hello", "world", "foo"];
    let data1_cstr = [
        CString::new("hello").unwrap(),
        CString::new("world").unwrap(),
        CString::new("foo").unwrap(),
    ];
    let data1_ptrs: Vec<_> = data1_cstr.iter().map(|cstr| cstr.as_ptr()).collect();
    let data2_str = ["test", "test2", "test3"];
    let data2_cstr = [
        CString::new("test").unwrap(),
        CString::new("test2").unwrap(),
        CString::new("test3").unwrap(),
    ];
    let data2_ptrs: Vec<_> = data2_cstr.iter().map(|cstr| cstr.as_ptr()).collect();

    unsafe {
        ArraysTest_SetStrSlice(&mut t, data1_ptrs.as_ptr(), data1_ptrs.len());
        assert_eq!(data1_str.as_slice(), t.val_array_string.as_slice());
        ArraysTest_SetStringSlice(&mut t, data2_ptrs.as_ptr(), data2_ptrs.len());
        assert_eq!(data2_str.as_slice(), t.val_array_string.as_slice());
        t.val_array_string.clear();

        ArraysTest_MoveStrArray(&mut t, data1_ptrs.as_ptr(), data1_ptrs.len());
        assert_eq!(data1_str.as_slice(), t.val_array_string.as_slice());
        ArraysTest_MoveStringArray(&mut t, data2_ptrs.as_ptr(), data2_ptrs.len());
        assert_eq!(data2_str.as_slice(), t.val_array_string.as_slice());
        t.val_array_string.clear();

        ArraysTest_SetStrArray(&mut t, data1_ptrs.as_ptr(), data1_ptrs.len());
        assert_eq!(data1_str.as_slice(), t.val_array_string.as_slice());
        ArraysTest_SetStringArray(&mut t, data2_ptrs.as_ptr(), data2_ptrs.len());
        assert_eq!(data2_str.as_slice(), t.val_array_string.as_slice());
        t.val_array_string.clear();
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
fn test_move_primitive_array_wrong_size_should_panic() {
    let mut t = ArraysTest::default();
    let data = [0; 3];
    unsafe {
        ArraysTest_MovePrimitiveArray(&mut t, data.as_ptr(), 2);
    }
}

#[test]
#[should_panic]
fn test_ref_primitive_array_wrong_size_should_panic() {
    let mut t = ArraysTest::default();
    let data = [0; 3];
    unsafe {
        ArraysTest_SetPrimitiveArray(&mut t, data.as_ptr(), 4);
    }
}

#[test]
#[should_panic]
fn test_mut_ref_primitive_array_wrong_size_should_panic() {
    let t = ArraysTest::default();
    let mut data = [0; 3];
    unsafe {
        ArraysTest_GetPrimitiveArray(&t, data.as_mut_ptr(), 5);
    }
}

#[test]
#[should_panic]
fn test_move_managed_array_wrong_size_should_panic() {
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
fn test_ref_managed_array_wrong_size_should_panic() {
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
fn test_mut_ref_managed_array_wrong_size_should_panic() {
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

#[test]
#[should_panic]
fn test_move_copyable_array_wrong_size_should_panic() {
    let mut t = ArraysTest::default();
    let data = [
        CopyableData::new(0),
        CopyableData::new(0),
        CopyableData::new(0),
    ];
    unsafe {
        ArraysTest_MoveCopyableArray(&mut t, data.as_ptr(), 2);
    }
}

#[test]
#[should_panic]
fn test_ref_copyable_array_wrong_size_should_panic() {
    let mut t = ArraysTest::default();
    let data = [
        CopyableData::new(0),
        CopyableData::new(0),
        CopyableData::new(0),
    ];
    unsafe {
        ArraysTest_SetCopyableArray(&mut t, data.as_ptr(), 4);
    }
}

#[test]
#[should_panic]
fn test_mut_ref_copyable_array_wrong_size_should_panic() {
    let t = ArraysTest::default();
    let mut data = [
        CopyableData::new(0),
        CopyableData::new(0),
        CopyableData::new(0),
    ];
    unsafe {
        ArraysTest_GetCopyableArray(&t, data.as_mut_ptr(), 5);
    }
}

#[test]
#[should_panic]
fn test_move_str_array_wrong_size_should_panic() {
    let mut t = ArraysTest::default();
    let data = [std::ptr::null(), std::ptr::null(), std::ptr::null()];
    unsafe {
        ArraysTest_MoveStrArray(&mut t, data.as_ptr(), 2);
    }
}

#[test]
#[should_panic]
fn test_move_string_array_wrong_size_should_panic() {
    let mut t = ArraysTest::default();
    let data = [std::ptr::null(), std::ptr::null(), std::ptr::null()];
    unsafe {
        ArraysTest_MoveStringArray(&mut t, data.as_ptr(), 4);
    }
}

#[test]
#[should_panic]
fn test_set_str_array_wrong_size_should_panic() {
    let mut t = ArraysTest::default();
    let data = [std::ptr::null(), std::ptr::null(), std::ptr::null()];
    unsafe {
        ArraysTest_SetStrArray(&mut t, data.as_ptr(), 2);
    }
}

#[test]
#[should_panic]
fn test_set_string_array_wrong_size_should_panic() {
    let mut t = ArraysTest::default();
    let data = [std::ptr::null(), std::ptr::null(), std::ptr::null()];
    unsafe {
        ArraysTest_SetStringArray(&mut t, data.as_ptr(), 4);
    }
}
