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
    // Slices.

    pub fn set_primitive_slice(&mut self, data: &[f32]) {
        self.val_array_primitive = data.to_vec();
    }

    pub fn get_primitive_slice(&self, out: &mut [f32]) {
        for i in 0..std::cmp::min(out.len(), self.val_array_primitive.len()) {
            out[i] = self.val_array_primitive[i];
        }
    }

    pub fn set_custom_slice(&mut self, data: &[Data]) {
        self.val_array_custom = data.to_vec();
    }

    pub fn get_custom_slice(&self, out: &mut [Data]) {
        for i in 0..std::cmp::min(out.len(), self.val_array_custom.len()) {
            out[i] = self.val_array_custom[i].clone();
        }
    }

    // Arrays.

    pub fn move_primitive_array(&mut self, data: [f32; 3]) {
        self.val_array_primitive = data.to_vec();
    }

    pub fn set_primitive_array(&mut self, data: &[f32; 3]) {
        self.val_array_primitive = data.to_vec();
    }

    pub fn get_primitive_array(&self, out: &mut [f32; 3]) {
        for i in 0..std::cmp::min(out.len(), self.val_array_primitive.len()) {
            out[i] = self.val_array_primitive[i];
        }
    }

    pub fn move_custom_array(&mut self, data: [Data; 3]) {
        self.val_array_custom = data.to_vec();
    }

    pub fn set_custom_array(&mut self, data: &[Data; 3]) {
        self.val_array_custom = data.to_vec();
    }

    pub fn get_custom_array(&self, out: &mut [Data; 3]) {
        for i in 0..std::cmp::min(out.len(), self.val_array_custom.len()) {
            out[i] = self.val_array_custom[i].clone();
        }
    }
}

#[test]
fn test_primitive_array() {
    let mut ts = TestStructArrays::default();

    let data = vec![1.0, 2.0, 3.0];
    let mut data_read = vec![0.0; 3];

    let data_array = [3.0, 4.0, 5.0];
    let data_array2 = [6.0, 7.0, 8.0];
    let mut data_array_read = [0.0; 3];

    unsafe {
        TestStructArrays_SetPrimitiveSlice(&mut ts, data.as_ptr(), data.len() as u32);
        assert_eq!(data, ts.val_array_primitive);
        TestStructArrays_GetPrimitiveSlice(&mut ts, data_read.as_mut_ptr(), data_read.len() as u32);
        assert_eq!(data, data_read);
        TestStructArrays_MovePrimitiveArray(&mut ts, data_array.as_ptr());
        assert_eq!(data_array.as_slice(), ts.val_array_primitive);
        TestStructArrays_SetPrimitiveArray(&mut ts, data_array2.as_ptr());
        assert_eq!(data_array2.as_slice(), ts.val_array_primitive);
        TestStructArrays_GetPrimitiveArray(&mut ts, data_array_read.as_mut_ptr());
        assert_eq!(data_array2, data_array_read);
    }
}

#[test]
fn test_custom_array() {
    let mut ts = TestStructArrays::default();

    let data = vec![Data::new(3), Data::new(4), Data::new(5)];
    let mut data_read = vec![Data::new(0); 3];

    let data_array = [Data::new(3), Data::new(4), Data::new(5)];
    let data_array2 = [Data::new(6), Data::new(7), Data::new(8)];
    let mut data_array_read = [Data::new(0), Data::new(0), Data::new(0)];

    unsafe {
        TestStructArrays_SetCustomSlice(&mut ts, data.as_ptr(), data.len() as u32);
        assert_eq!(data, ts.val_array_custom);
        TestStructArrays_GetCustomSlice(&mut ts, data_read.as_mut_ptr(), data_read.len() as u32);
        assert_eq!(data, data_read);
        TestStructArrays_MoveCustomArray(&mut ts, data_array.as_ptr());
        assert_eq!(data_array.as_slice(), ts.val_array_custom);
        TestStructArrays_SetCustomArray(&mut ts, data_array2.as_ptr());
        assert_eq!(data_array2.as_slice(), ts.val_array_custom);
        TestStructArrays_GetCustomArray(&mut ts, data_array_read.as_mut_ptr());
        assert_eq!(data_array2, data_array_read);
    }
}
