mod utils;

use luajit_ffi_gen::luajit_ffi;

use crate::utils::*;

#[derive(Default, Clone)]
pub struct Data {
    pub val: u32,
}

impl Data {
    fn new(val: u32) -> Data {
        Data { val }
    }
}

// This is a well known copyable type defined in type_info.rs
#[derive(Default, Clone, Copy)]
pub struct WindowPos {
    pub val: u32,
}

impl WindowPos {
    fn new(val: u32) -> WindowPos {
        WindowPos { val }
    }
}

#[derive(Default)]
pub struct MyStruct {
    val_u32: u32,
    val_f32: f32,
    val_str: String,
    val_data: Data,
    val_copyable: WindowPos,
}

// NOTE: remove 'lua_ffi' parameter to see generated Lua file. Do not commit it!!!
#[luajit_ffi(name = "My_Struct", lua_ffi = false)]
impl MyStruct {
    pub fn func1(&self) {}
    pub fn func2(&mut self) {}

    #[bind(name = "FUNC3")]
    pub fn func3() {}

    pub fn set_u32(&mut self, val: u32) {
        self.val_u32 = val;
    }

    pub fn get_u32(&self) -> u32 {
        self.val_u32
    }

    pub fn set_f32(&mut self, val: &f32) {
        self.val_f32 = *val;
    }

    pub fn get_f32(&self) -> f32 {
        self.val_f32
    }

    pub fn set_str(&mut self, val: &str) {
        self.val_str = val.into();
    }

    pub fn get_str(&self) -> &str {
        &self.val_str
    }

    pub fn set_data(&mut self, val: &Data) {
        self.val_data = val.clone();
    }

    pub fn take_data(&mut self, val: Data) {
        self.val_data = val;
    }

    pub fn take_boxed_data(&mut self, val: Box<Data>) {
        self.val_data = *val;
    }

    pub fn get_data(&self) -> Data {
        self.val_data.clone()
    }

    #[bind(out_param = true)]
    pub fn get_data_via_out_param(&self) -> Data {
        self.val_data.clone()
    }

    pub fn get_data_ref(&self) -> &Data {
        &self.val_data
    }

    pub fn get_boxed_data(&self) -> Box<Data> {
        Box::new(self.val_data.clone())
    }

    pub fn get_data_mut(&mut self) -> &mut Data {
        &mut self.val_data
    }

    pub fn set_opt(&mut self, val: Option<u32>) {
        if let Some(val) = val {
            self.val_u32 = val;
        }
    }

    pub fn get_opt_u32(&self) -> Option<u32> {
        if self.val_u32 > 0 {
            Some(self.val_u32)
        } else {
            None
        }
    }

    pub fn get_opt_data(&self) -> Option<Data> {
        Some(self.val_data.clone())
    }

    pub fn set_opt_ref(&mut self, val: Option<&u32>) {
        if let Some(val) = val {
            self.val_u32 = *val;
        }
    }

    pub fn set_opt_mut(&mut self, val: Option<&mut u32>) {
        if let Some(val) = val {
            self.val_u32 = *val;
            *val = 0;
        }
    }

    pub fn ret_res_val() -> Result<u8, u8> {
        Ok(42)
    }

    pub fn ret_res_err() -> Result<u8, u8> {
        Err(13)
    }

    pub fn ret_res_opt_val() -> Result<Option<u8>, u8> {
        Ok(Some(42))
    }

    pub fn set_copyable(&mut self, c: WindowPos) {
        self.val_copyable = c;
    }

    pub fn set_copyable_by_ref(&mut self, c: &WindowPos) {
        self.val_copyable = *c;
    }

    pub fn set_copyable_by_mut_ref(&mut self, c: &mut WindowPos) {
        self.val_copyable = *c;
    }

    pub fn get_copyable(&self) -> WindowPos {
        self.val_copyable
    }

    #[bind(out_param = true)]
    pub fn get_copyable_via_out_param(&self) -> WindowPos {
        self.val_copyable
    }

    pub fn get_boxed_copyable(&self) -> Box<WindowPos> {
        Box::new(self.val_copyable)
    }
}

#[test]
fn test_functions() {
    let ms = MyStruct::default();
    let mut ms2 = MyStruct::default();

    ms.func1();
    ms2.func2();
    MyStruct::func3();

    MyStruct_Func1(&ms);
    MyStruct_Func2(&mut ms2);
    MyStruct_FUNC3();

    MyStruct_SetU32(&mut ms2, 33);
    assert_eq!(MyStruct_GetU32(&ms2), 33);

    MyStruct_SetF32(&mut ms2, 33.0);
    assert_eq!(MyStruct_GetF32(&ms2), 33.0);

    MyStruct_SetData(&mut ms2, &Data::new(2));
    assert_eq!(MyStruct_GetData(&ms2).val, 2);
    assert_eq!(unsafe { (*MyStruct_GetOptData(&ms2)).val }, 2);

    MyStruct_TakeData(&mut ms2, Box::new(Data::new(4)));
    let mut returned_data = Data::new(0);
    MyStruct_GetDataViaOutParam(&ms2, &mut returned_data);
    assert_eq!(returned_data.val, 4);

    MyStruct_TakeBoxedData(&mut ms2, Box::new(Data::new(6)));
    assert_eq!(MyStruct_GetBoxedData(&ms2).val, 6);

    let val = MyStruct_RetResVal();
    assert_eq!(val, 42);
}

#[test]
fn test_copyable_param() {
    let mut ms = MyStruct::default();

    MyStruct_SetCopyable(&mut ms, WindowPos::new(5));
    assert_eq!(ms.val_copyable.val, 5);

    let copyable_data = WindowPos::new(7);
    MyStruct_SetCopyableByRef(&mut ms, &copyable_data);
    assert_eq!(MyStruct_GetCopyable(&ms).val, 7);

    let mut copyable_data2 = WindowPos::new(9);
    MyStruct_SetCopyableByMutRef(&mut ms, &mut copyable_data2);
    assert_eq!(MyStruct_GetBoxedCopyable(&ms).val, 9);

    MyStruct_SetCopyable(&mut ms, WindowPos::new(11));
    let mut copyable_result = WindowPos::default();
    MyStruct_GetCopyableViaOutParam(&ms, &mut copyable_result);
    assert_eq!(copyable_result.val, 11);
}

#[test]
#[should_panic]
fn test_impl_return_error() {
    MyStruct_RetResErr();
}
