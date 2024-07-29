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
pub struct TestStructCallbacks {
    val_primitives: Vec<f32>,
    val_customs: Vec<Data>,
}

// NOTE: remove 'lua_ffi' parameter to see generated Lua file. Do not commit it!!!
#[luajit_ffi(lua_ffi = false)]
impl TestStructCallbacks {
    pub fn get_nth_primitive<F: FnOnce(f32) -> ()>(&mut self, index: usize, callback: F) {
        callback(self.val_primitives[index]);
    }
}

#[test]
fn test_callbacks() {
    let mut ts = TestStructCallbacks::default();
}
