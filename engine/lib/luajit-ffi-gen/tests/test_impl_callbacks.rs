use luajit_ffi_gen::luajit_ffi;

mod helpers;
use helpers::*;

#[derive(Default)]
pub struct CallbackTest {
    val_primitives: Vec<f32>,
    val_noncopyable: Vec<Data>,
}

// NOTE: remove 'lua_ffi' parameter to see generated Lua file. Do not commit it!!!
#[luajit_ffi(lua_ffi = false)]
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

    pub fn nth_noncopyable<F: FnOnce(Data) -> ()>(&self, index: usize, callback: F) {
        callback(self.val_noncopyable[index].clone());
    }

    pub fn nth_noncopyable_ref<F: FnOnce(&Data) -> ()>(&self, index: usize, callback: F) {
        callback(&self.val_noncopyable[index]);
    }

    pub fn nth_noncopyable_mut<F: FnOnce(&mut Data) -> ()>(&mut self, index: usize, callback: F) {
        callback(&mut self.val_noncopyable[index]);
    }

    pub fn nth_noncopyable_val_opt<F: FnOnce(Option<Data>) -> ()>(
        &self,
        index: usize,
        callback: F,
    ) {
        callback(self.val_noncopyable.get(index).cloned());
    }

    pub fn nth_noncopyable_ref_opt<F: FnOnce(Option<&Data>) -> ()>(
        &self,
        index: usize,
        callback: F,
    ) {
        callback(self.val_noncopyable.get(index));
    }

    pub fn nth_noncopyable_mut_opt<F: FnOnce(Option<&mut Data>) -> ()>(
        &mut self,
        index: usize,
        callback: F,
    ) {
        callback(self.val_noncopyable.get_mut(index));
    }

    pub fn insert_noncopyable<F: FnOnce() -> Data>(&mut self, callback: F) {
        self.val_noncopyable.push(callback());
    }

    // Arrays.

    pub fn read_primitive_array<F: FnOnce(&[f32]) -> ()>(&self, callback: F) {
        callback(self.val_primitives.as_slice());
    }

    pub fn lock_primitive_array<F: FnOnce(&mut [f32]) -> ()>(&mut self, callback: F) {
        callback(self.val_primitives.as_mut_slice());
    }

    pub fn read_noncopyable_array<F: FnOnce(&[Data]) -> ()>(&self, callback: F) {
        callback(self.val_noncopyable.as_slice());
    }

    pub fn lock_noncopyable_array<F: FnOnce(&mut [Data]) -> ()>(&mut self, callback: F) {
        callback(self.val_noncopyable.as_mut_slice());
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
        callback: impl FnOnce(f32, Option<&Data>) -> f32,
    ) {
        let result = callback(self.val_primitives[index], self.val_noncopyable.get(index));
        self.val_primitives[index] = result;
    }

    pub fn passthrough<F: Fn(u32) -> u32>(input: u32, callback: F) -> u32 {
        callback(input)
    }
}

#[test]
fn test_primitives() {
    let mut ts = CallbackTest::default();

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
        CallbackTest_InsertPrimitive(&mut ts, add_1);
        CallbackTest_InsertPrimitive(&mut ts, add_2);

        CallbackTest_NthPrimitive(&ts, 0, get_value);
        assert_eq!(VALUE, 1.0);
        CallbackTest_NthPrimitive(&ts, 1, get_value);
        assert_eq!(VALUE, 2.0);

        CallbackTest_NthPrimitiveRef(&mut ts, 0, get_value);
        assert_eq!(VALUE, 1.0);

        CallbackTest_NthPrimitiveMut(&mut ts, 0, mutate_value);
        CallbackTest_NthPrimitive(&ts, 0, get_value);
        assert_eq!(VALUE, 4.0);

        CallbackTest_NthPrimitiveValOpt(&ts, 1, get_value_opt);
        assert_eq!(VALUE, 2.0);
        CallbackTest_NthPrimitiveValOpt(&ts, 5, get_value_opt);
        assert_eq!(VALUE, -1.0);

        CallbackTest_NthPrimitiveMutOpt(&mut ts, 1, mutate_value_if_present);
        CallbackTest_NthPrimitiveRefOpt(&mut ts, 1, get_value_opt);
        assert_eq!(VALUE, 4.0);
        CallbackTest_NthPrimitiveRefOpt(&mut ts, 5, get_value_opt);
        assert_eq!(VALUE, -1.0);

        // We're just expecting this to not crash.
        CallbackTest_NthPrimitiveMutOpt(&mut ts, 5, mutate_value_if_present);
    }
}

#[test]
fn test_noncopyable() {
    let mut ts = CallbackTest::default();

    static mut VALUE: Data = Data::new(0);

    extern "C" fn add_1() -> Box<Data> {
        Box::new(Data::new(1))
    }

    extern "C" fn add_2() -> Box<Data> {
        Box::new(Data::new(2))
    }

    extern "C" fn get_value(v: Box<Data>) {
        unsafe {
            VALUE = *v;
        }
    }

    extern "C" fn get_value_opt(v: Option<&Data>) {
        unsafe {
            if let Some(v) = v {
                VALUE = v.clone();
            } else {
                VALUE = Data::new(1024);
            }
        }
    }

    extern "C" fn get_value_ref(v: &Data) {
        unsafe {
            VALUE = v.clone();
        }
    }

    extern "C" fn mutate_value(v: &mut Data) {
        *v = Data::new(4);
    }

    extern "C" fn mutate_value_if_present(v: Option<&mut Data>) {
        if let Some(v) = v {
            *v = Data::new(4);
        }
    }

    unsafe {
        CallbackTest_InsertNoncopyable(&mut ts, add_1);
        CallbackTest_InsertNoncopyable(&mut ts, add_2);

        CallbackTest_NthNoncopyable(&ts, 0, get_value);
        assert_eq!(VALUE, Data::new(1));
        CallbackTest_NthNoncopyable(&ts, 1, get_value);
        assert_eq!(VALUE, Data::new(2));

        CallbackTest_NthNoncopyableRef(&mut ts, 0, get_value_ref);
        assert_eq!(VALUE, Data::new(1));

        CallbackTest_NthNoncopyableMut(&mut ts, 0, mutate_value);
        CallbackTest_NthNoncopyable(&ts, 0, get_value);
        assert_eq!(VALUE, Data::new(4));

        CallbackTest_NthNoncopyableValOpt(&ts, 1, get_value_opt);
        assert_eq!(VALUE, Data::new(2));
        CallbackTest_NthNoncopyableValOpt(&ts, 5, get_value_opt);
        assert_eq!(VALUE, Data::new(1024));

        CallbackTest_NthNoncopyableMutOpt(&mut ts, 1, mutate_value_if_present);
        CallbackTest_NthNoncopyableRefOpt(&mut ts, 1, get_value_opt);
        assert_eq!(VALUE, Data::new(4));
        CallbackTest_NthNoncopyableRefOpt(&mut ts, 5, get_value_opt);
        assert_eq!(VALUE, Data::new(1024));

        // We're just expecting this to not crash.
        CallbackTest_NthNoncopyableMutOpt(&mut ts, 5, mutate_value_if_present);
    }
}

#[test]
fn test_primitive_arrays() {
    let mut ts = CallbackTest::default();

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
        ts.val_primitives = vec![1.0, 2.0, 3.0];

        CallbackTest_ReadPrimitiveArray(&ts, get_values);
        assert_eq!(ts.val_primitives, OUT);

        CallbackTest_LockPrimitiveArray(&mut ts, double_values);
        assert_eq!(ts.val_primitives, vec![2.0, 4.0, 6.0]);
    }
}

#[test]
fn test_noncopyable_arrays() {
    let mut ts = CallbackTest::default();

    static mut OUT: Vec<Data> = vec![];

    extern "C" fn get_values(data: *const Data, len: usize) {
        unsafe {
            OUT = std::slice::from_raw_parts(data, len).to_vec();
        }
    }

    extern "C" fn double_values(data: *mut Data, len: usize) {
        unsafe {
            for v in std::slice::from_raw_parts_mut(data, len) {
                v.val *= 2;
            }
        }
    }

    unsafe {
        ts.val_noncopyable = vec![Data::new(1), Data::new(2), Data::new(3)];

        CallbackTest_ReadNoncopyableArray(&ts, get_values);
        assert_eq!(ts.val_noncopyable, OUT);

        CallbackTest_LockNoncopyableArray(&mut ts, double_values);
        assert_eq!(
            ts.val_noncopyable,
            vec![Data::new(2), Data::new(4), Data::new(6)]
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
    let mut ts = CallbackTest::default();

    extern "C" fn combine_and_replace(in_prim: f32, in_noncopyable: Option<&Data>) -> f32 {
        in_prim + in_noncopyable.cloned().unwrap_or(Data::new(10)).val as f32
    }

    extern "C" fn square_value(input: u32) -> u32 {
        input * input
    }

    unsafe {
        ts.val_primitives = vec![0.0, 2.0];
        ts.val_noncopyable = vec![Data::new(5)];

        CallbackTest_GetMultipleAndReplace(&mut ts, 0, combine_and_replace);
        assert_eq!(ts.val_primitives[0], 5.0);

        CallbackTest_GetMultipleAndReplace(&mut ts, 1, combine_and_replace);
        assert_eq!(ts.val_primitives[1], 12.0);

        let result = CallbackTest_Passthrough(4, square_value);
        assert_eq!(4 * 4, result);
    }
}
