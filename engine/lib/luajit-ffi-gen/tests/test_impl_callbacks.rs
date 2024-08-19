use luajit_ffi_gen::luajit_ffi;

#[allow(dead_code)]
mod helpers;
use helpers::*;

#[derive(Default)]
pub struct CallbackTest {
    val_primitives: Vec<f32>,
    val_managed: Vec<ManagedData>,
}

#[luajit_ffi(gen_dir = "./tests/out/ffi_gen", meta_dir = "./tests/out/ffi_meta")]
impl CallbackTest {
    pub fn nth_primitive<F: FnOnce(f32) -> ()>(&self, index: usize, callback: F) {
        callback(self.val_primitives[index]);
    }

    // Note: -> () is optional in the Fn type, so lets omit it here.
    pub fn nth_primitive_ref<F: FnOnce(&f32)>(&self, index: usize, callback: F) {
        callback(&self.val_primitives[index]);
    }

    pub fn nth_primitive_mut<F: FnOnce(&mut f32) -> ()>(&mut self, index: usize, callback: F) {
        callback(&mut self.val_primitives[index]);
    }

    pub fn nth_primitive_val_opt<F: FnOnce(Option<f32>) -> ()>(&self, index: usize, callback: F) {
        callback(self.val_primitives.get(index).cloned());
    }

    pub fn nth_primitive_ref_opt<F: FnOnce(Option<&f32>) -> ()>(&self, index: usize, callback: F) {
        callback(self.val_primitives.get(index));
    }

    pub fn nth_primitive_mut_opt<F: FnOnce(Option<&mut f32>) -> ()>(
        &mut self,
        index: usize,
        callback: F,
    ) {
        callback(self.val_primitives.get_mut(index));
    }

    pub fn insert_primitive<F: FnOnce() -> f32>(&mut self, callback: F) {
        self.val_primitives.push(callback());
    }

    pub fn nth_managed<F: FnOnce(ManagedData) -> ()>(&self, index: usize, callback: F) {
        callback(self.val_managed[index].clone());
    }

    pub fn nth_managed_ref<F: FnOnce(&ManagedData) -> ()>(&self, index: usize, callback: F) {
        callback(&self.val_managed[index]);
    }

    pub fn nth_managed_mut<F: FnOnce(&mut ManagedData) -> ()>(
        &mut self,
        index: usize,
        callback: F,
    ) {
        callback(&mut self.val_managed[index]);
    }

    pub fn nth_managed_val_opt<F: FnOnce(Option<ManagedData>) -> ()>(
        &self,
        index: usize,
        callback: F,
    ) {
        callback(self.val_managed.get(index).cloned());
    }

    pub fn nth_managed_ref_opt<F: FnOnce(Option<&ManagedData>) -> ()>(
        &self,
        index: usize,
        callback: F,
    ) {
        callback(self.val_managed.get(index));
    }

    pub fn nth_managed_mut_opt<F: FnOnce(Option<&mut ManagedData>) -> ()>(
        &mut self,
        index: usize,
        callback: F,
    ) {
        callback(self.val_managed.get_mut(index));
    }

    pub fn insert_managed<F: FnOnce() -> ManagedData>(&mut self, callback: F) {
        self.val_managed.push(callback());
    }

    // Arrays.

    pub fn read_primitive_array<F: FnOnce(&[f32]) -> ()>(&self, callback: F) {
        callback(self.val_primitives.as_slice());
    }

    pub fn lock_primitive_array<F: FnOnce(&mut [f32]) -> ()>(&mut self, callback: F) {
        callback(self.val_primitives.as_mut_slice());
    }

    pub fn read_managed_array<F: FnOnce(&[ManagedData]) -> ()>(&self, callback: F) {
        callback(self.val_managed.as_slice());
    }

    pub fn lock_managed_array<F: FnOnce(&mut [ManagedData]) -> ()>(&mut self, callback: F) {
        callback(self.val_managed.as_mut_slice());
    }

    // Strings.

    pub fn transform_string<F: Fn(String) -> String>(s: String, callback: F) -> String {
        callback(s)
    }

    pub fn transform_str<F: Fn(&str) -> String>(s: &str, callback: F) -> String {
        callback(s)
    }

    // Edge cases.

    pub fn get_multiple_and_replace(
        &mut self,
        index: usize,
        callback: impl FnOnce(f32, Option<&ManagedData>) -> f32,
    ) {
        let result = callback(self.val_primitives[index], self.val_managed.get(index));
        self.val_primitives[index] = result;
    }

    pub fn passthrough<F: Fn(u32) -> u32>(input: u32, callback: F) -> u32 {
        callback(input)
    }
}

#[test]
fn test_primitives() {
    let mut t = CallbackTest::default();

    static mut VALUE: f32 = 0.0;

    extern "C" fn add_1() -> f32 {
        1.0
    }

    extern "C" fn add_2() -> f32 {
        2.0
    }

    extern "C" fn get_value(v: f32) {
        unsafe {
            VALUE = v;
        }
    }

    extern "C" fn get_value_ref(v: &f32) {
        unsafe {
            VALUE = *v;
        }
    }

    extern "C" fn get_value_opt(v: Option<&f32>) {
        unsafe {
            if let Some(v) = v {
                VALUE = *v;
            } else {
                VALUE = -1.0;
            }
        }
    }

    extern "C" fn mutate_value(v: &mut f32) {
        *v = 4.0;
    }

    extern "C" fn mutate_value_if_present(v: Option<&mut f32>) {
        if let Some(v) = v {
            *v = 4.0;
        }
    }

    unsafe {
        CallbackTest_InsertPrimitive(&mut t, add_1);
        CallbackTest_InsertPrimitive(&mut t, add_2);

        CallbackTest_NthPrimitive(&t, 0, get_value);
        assert_eq!(VALUE, 1.0);
        CallbackTest_NthPrimitive(&t, 1, get_value);
        assert_eq!(VALUE, 2.0);

        CallbackTest_NthPrimitiveRef(&mut t, 0, get_value_ref);
        assert_eq!(VALUE, 1.0);

        CallbackTest_NthPrimitiveMut(&mut t, 0, mutate_value);
        CallbackTest_NthPrimitive(&t, 0, get_value);
        assert_eq!(VALUE, 4.0);

        CallbackTest_NthPrimitiveValOpt(&t, 1, get_value_opt);
        assert_eq!(VALUE, 2.0);
        CallbackTest_NthPrimitiveValOpt(&t, 5, get_value_opt);
        assert_eq!(VALUE, -1.0);

        CallbackTest_NthPrimitiveMutOpt(&mut t, 1, mutate_value_if_present);
        CallbackTest_NthPrimitiveRefOpt(&mut t, 1, get_value_opt);
        assert_eq!(VALUE, 4.0);
        CallbackTest_NthPrimitiveRefOpt(&mut t, 5, get_value_opt);
        assert_eq!(VALUE, -1.0);

        // We're just expecting this to not crash.
        CallbackTest_NthPrimitiveMutOpt(&mut t, 5, mutate_value_if_present);
    }
}

#[test]
fn test_managed() {
    let mut t = CallbackTest::default();

    static mut VALUE: ManagedData = ManagedData::new(0);

    extern "C" fn add_1() -> Box<ManagedData> {
        Box::new(ManagedData::new(1))
    }

    extern "C" fn add_2() -> Box<ManagedData> {
        Box::new(ManagedData::new(2))
    }

    extern "C" fn get_value(v: Box<ManagedData>) {
        unsafe {
            VALUE = *v;
        }
    }

    extern "C" fn get_value_opt(v: Option<&ManagedData>) {
        unsafe {
            if let Some(v) = v {
                VALUE = v.clone();
            } else {
                VALUE = ManagedData::new(1024);
            }
        }
    }

    extern "C" fn get_value_ref(v: &ManagedData) {
        unsafe {
            VALUE = v.clone();
        }
    }

    extern "C" fn mutate_value(v: &mut ManagedData) {
        *v = ManagedData::new(4);
    }

    extern "C" fn mutate_value_if_present(v: Option<&mut ManagedData>) {
        if let Some(v) = v {
            *v = ManagedData::new(4);
        }
    }

    unsafe {
        CallbackTest_InsertManaged(&mut t, add_1);
        CallbackTest_InsertManaged(&mut t, add_2);

        CallbackTest_NthManaged(&t, 0, get_value);
        assert_eq!(VALUE, ManagedData::new(1));
        CallbackTest_NthManaged(&t, 1, get_value);
        assert_eq!(VALUE, ManagedData::new(2));

        CallbackTest_NthManagedRef(&mut t, 0, get_value_ref);
        assert_eq!(VALUE, ManagedData::new(1));

        CallbackTest_NthManagedMut(&mut t, 0, mutate_value);
        CallbackTest_NthManaged(&t, 0, get_value);
        assert_eq!(VALUE, ManagedData::new(4));

        CallbackTest_NthManagedValOpt(&t, 1, get_value_opt);
        assert_eq!(VALUE, ManagedData::new(2));
        CallbackTest_NthManagedValOpt(&t, 5, get_value_opt);
        assert_eq!(VALUE, ManagedData::new(1024));

        CallbackTest_NthManagedMutOpt(&mut t, 1, mutate_value_if_present);
        CallbackTest_NthManagedRefOpt(&mut t, 1, get_value_opt);
        assert_eq!(VALUE, ManagedData::new(4));
        CallbackTest_NthManagedRefOpt(&mut t, 5, get_value_opt);
        assert_eq!(VALUE, ManagedData::new(1024));

        // We're just expecting this to not crash.
        CallbackTest_NthManagedMutOpt(&mut t, 5, mutate_value_if_present);
    }
}

#[test]
fn test_primitive_arrays() {
    let mut t = CallbackTest::default();

    static mut OUT: Vec<f32> = vec![];

    extern "C" fn get_values(data: *const f32, len: usize) {
        unsafe {
            OUT = std::slice::from_raw_parts(data, len).to_vec();
        }
    }

    extern "C" fn double_values(data: *mut f32, len: usize) {
        unsafe {
            for v in std::slice::from_raw_parts_mut(data, len) {
                *v *= 2.0;
            }
        }
    }

    unsafe {
        t.val_primitives = vec![1.0, 2.0, 3.0];

        CallbackTest_ReadPrimitiveArray(&t, get_values);
        assert_eq!(t.val_primitives, OUT);

        CallbackTest_LockPrimitiveArray(&mut t, double_values);
        assert_eq!(t.val_primitives, vec![2.0, 4.0, 6.0]);
    }
}

#[test]
fn test_managed_arrays() {
    let mut t = CallbackTest::default();

    static mut OUT: Vec<ManagedData> = vec![];

    extern "C" fn get_values(data: *const ManagedData, len: usize) {
        unsafe {
            OUT = std::slice::from_raw_parts(data, len).to_vec();
        }
    }

    extern "C" fn double_values(data: *mut ManagedData, len: usize) {
        unsafe {
            for v in std::slice::from_raw_parts_mut(data, len) {
                v.val *= 2;
            }
        }
    }

    unsafe {
        t.val_managed = vec![
            ManagedData::new(1),
            ManagedData::new(2),
            ManagedData::new(3),
        ];

        CallbackTest_ReadManagedArray(&t, get_values);
        assert_eq!(t.val_managed, OUT);

        CallbackTest_LockManagedArray(&mut t, double_values);
        assert_eq!(
            t.val_managed,
            vec![
                ManagedData::new(2),
                ManagedData::new(4),
                ManagedData::new(6)
            ]
        );
    }
}

#[test]
fn test_strings() {
    use std::ffi::{CStr, CString};

    let test_str = CString::new("hello").unwrap();
    let expected_str = CString::new("HELLO").unwrap();
    let test_str2 = CString::new("world").unwrap();
    let expected_str2 = CString::new("WORLD").unwrap();

    static mut STORAGE: Option<CString> = None;

    extern "C" fn str_to_uppercase(ptr: *const i8) -> *const i8 {
        unsafe {
            let s = CStr::from_ptr(ptr).to_str().unwrap().to_ascii_uppercase();
            STORAGE = Some(CString::new(s).unwrap());
            STORAGE.as_ref().unwrap().as_ptr()
        }
    }

    unsafe {
        let result = CallbackTest_TransformString(test_str.as_ptr(), str_to_uppercase);
        assert_eq!(CStr::from_ptr(result), expected_str.as_c_str());

        let result = CallbackTest_TransformStr(test_str2.as_ptr(), str_to_uppercase);
        assert_eq!(CStr::from_ptr(result), expected_str2.as_c_str());
    }
}

#[test]
fn test_edge_cases() {
    let mut t = CallbackTest::default();

    extern "C" fn combine_and_replace(in_prim: f32, in_managed: Option<&ManagedData>) -> f32 {
        in_prim + in_managed.cloned().unwrap_or(ManagedData::new(10)).val as f32
    }

    extern "C" fn square_value(input: u32) -> u32 {
        input * input
    }

    unsafe {
        t.val_primitives = vec![0.0, 2.0];
        t.val_managed = vec![ManagedData::new(5)];

        CallbackTest_GetMultipleAndReplace(&mut t, 0, combine_and_replace);
        assert_eq!(t.val_primitives[0], 5.0);

        CallbackTest_GetMultipleAndReplace(&mut t, 1, combine_and_replace);
        assert_eq!(t.val_primitives[1], 12.0);

        let result = CallbackTest_Passthrough(4, square_value);
        assert_eq!(4 * 4, result);
    }
}
