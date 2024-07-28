use luajit_ffi_gen::luajit_ffi;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Data {
    pub val: u32,
}

impl Data {
    fn new(val: u32) -> Data {
        Data { val }
    }
}

#[derive(Default)]
pub struct TestStructArrays {
    val_array_primitive: Vec<f32>,
    val_array_custom: Vec<Data>,
}

// NOTE: remove 'lua_ffi' parameter to see generated Lua file. Do not commit it!!!
#[luajit_ffi(lua_ffi = false)]
impl TestStructArrays {
    pub fn set_primitive_array(&mut self, data: &[f32]) {
        self.val_array_primitive = data.to_vec();
    }

    pub fn get_primitive_array(&mut self, out: &mut [f32]) {
        for i in 0..std::cmp::min(out.len(), self.val_array_primitive.len()) {
            out[i] = self.val_array_primitive[i];
        }
    }

    pub fn set_custom_array(&mut self, data: &[Data]) {
        self.val_array_custom = data.to_vec();
    }

    pub fn get_custom_array(&mut self, out: &mut [Data]) {
        for i in 0..std::cmp::min(out.len(), self.val_array_custom.len()) {
            out[i] = self.val_array_custom[i].clone();
        }
    }
}

#[test]
fn test_primitive_aray() {
    let mut ts = TestStructArrays::default();

    let data = vec![1.0, 2.0, 3.0];
    let mut new_data = vec![4.0, 5.0, 6.0];

    unsafe {
        TestStructArrays_SetPrimitiveArray(&mut ts, data.as_ptr(), data.len() as u32);
        assert_eq!(data, ts.val_array_primitive);
        TestStructArrays_GetPrimitiveArray(&mut ts, new_data.as_mut_ptr(), new_data.len() as u32);
        assert_eq!(data, new_data);
    }
}

#[test]
fn test_custom_array() {
    let mut ts = TestStructArrays::default();

    let data = vec![Data::new(3), Data::new(4), Data::new(5)];
    let mut new_data = vec![Data::new(0); 3];

    unsafe {
        TestStructArrays_SetCustomArray(&mut ts, data.as_ptr(), data.len() as u32);
        assert_eq!(data, ts.val_array_custom);
        TestStructArrays_GetCustomArray(&mut ts, new_data.as_mut_ptr(), new_data.len() as u32);
        assert_eq!(data, new_data);
    }
}
