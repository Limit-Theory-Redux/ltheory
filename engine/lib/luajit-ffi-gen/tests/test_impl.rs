use luajit_ffi_gen::luajit_ffi;

mod helpers;
use helpers::*;

#[derive(Default)]
pub struct MyStruct {
    val_u32: u32,
    val_f32: f32,
    val_str: String,
    val_data: Data,
    val_copyable: CopyableData,
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

    #[allow(clippy::boxed_local)] // Box is needed for testing
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

    pub fn set_copyable(&mut self, c: CopyableData) {
        self.val_copyable = c;
    }

    pub fn set_copyable_by_ref(&mut self, c: &CopyableData) {
        self.val_copyable = *c;
    }

    pub fn set_copyable_by_mut_ref(&mut self, c: &mut CopyableData) {
        self.val_copyable = *c;
    }

    pub fn get_copyable(&self) -> CopyableData {
        self.val_copyable
    }

    #[bind(out_param = true)]
    pub fn get_copyable_via_out_param(&self) -> CopyableData {
        self.val_copyable
    }

    pub fn get_boxed_copyable(&self) -> Box<CopyableData> {
        Box::new(self.val_copyable)
    }

    pub fn get_opt_copyable(&self) -> Option<CopyableData> {
        Some(self.val_copyable)
    }

    pub fn set_opt_str(&mut self, val: Option<&str>) {
        if let Some(val) = val {
            self.val_str = val.to_string();
        }
    }

    pub fn set_opt_string(&mut self, val: Option<String>) {
        if let Some(val) = val {
            self.val_str = val;
        }
    }

    pub fn set_opt_string_ref(&mut self, val: Option<&String>) {
        if let Some(val) = val {
            self.val_str = val.clone();
        }
    }

    pub fn get_opt_str(&self) -> Option<&str> {
        if self.val_str.len() > 0 {
            Some(self.val_str.as_str())
        } else {
            None
        }
    }

    pub fn get_opt_string(&self) -> Option<String> {
        if self.val_str.len() > 0 {
            Some(self.val_str.clone())
        } else {
            None
        }
    }

    pub fn get_opt_string_ref(&self) -> Option<&String> {
        if self.val_str.len() > 0 {
            Some(&self.val_str)
        } else {
            None
        }
    }
}

#[test]
fn test_functions() {
    let ms = MyStruct::default();
    let mut ms2 = MyStruct::default();

    ms.func1();
    ms2.func2();
    MyStruct::func3();

    unsafe {
        My_Struct_Func1(&ms);
        My_Struct_Func2(&mut ms2);
        My_Struct_FUNC3();

        My_Struct_SetU32(&mut ms2, 33);
        assert_eq!(My_Struct_GetU32(&ms2), 33);

        My_Struct_SetF32(&mut ms2, 33.0);
        assert_eq!(My_Struct_GetF32(&ms2), 33.0);

        My_Struct_SetData(&mut ms2, &Data::new(2));
        assert_eq!(My_Struct_GetData(&ms2).val, 2);
        assert_eq!((My_Struct_GetOptData(&ms2)).unwrap().val, 2);

        My_Struct_TakeData(&mut ms2, Box::new(Data::new(4)));
        let mut returned_data = Data::new(0);
        My_Struct_GetDataViaOutParam(&ms2, &mut returned_data);
        assert_eq!(returned_data.val, 4);

        My_Struct_TakeBoxedData(&mut ms2, Box::new(Data::new(6)));
        assert_eq!(My_Struct_GetBoxedData(&ms2).val, 6);

        let val = My_Struct_RetResVal();
        assert_eq!(val, 42);
    }
}

#[test]
fn test_copyable_param() {
    let mut ms = MyStruct::default();

    unsafe {
        My_Struct_SetCopyable(&mut ms, CopyableData::new(5));
        assert_eq!(ms.val_copyable.val, 5);

        let copyable_data = CopyableData::new(7);
        My_Struct_SetCopyableByRef(&mut ms, &copyable_data);
        assert_eq!(My_Struct_GetCopyable(&ms).val, 7);

        let mut copyable_data2 = CopyableData::new(9);
        My_Struct_SetCopyableByMutRef(&mut ms, &mut copyable_data2);
        assert_eq!(My_Struct_GetBoxedCopyable(&ms).val, 9);

        My_Struct_SetCopyable(&mut ms, CopyableData::new(11));
        let mut copyable_result = CopyableData::default();
        My_Struct_GetCopyableViaOutParam(&ms, &mut copyable_result);
        assert_eq!(copyable_result.val, 11);
    }
}

#[test]
fn test_optional_strings() {
    let mut td = MyStruct::default();

    let str_data1 = std::ffi::CString::new("hello").unwrap();
    let str_data2 = std::ffi::CString::new("world").unwrap();
    let str_data3 = std::ffi::CString::new("test").unwrap();

    use internal::ConvertIntoString;

    unsafe {
        let data = My_Struct_GetOptStr(&td);
        assert_eq!(data, std::ptr::null());

        My_Struct_SetOptStr(&mut td, str_data1.as_ptr());
        assert_eq!(td.val_str, str_data1.to_str().unwrap());

        let data = My_Struct_GetOptStr(&mut td);
        assert_eq!(td.val_str, data.as_str());

        My_Struct_SetOptString(&mut td, str_data2.as_ptr());
        assert_eq!(td.val_str, str_data2.to_str().unwrap());

        let data = My_Struct_GetOptString(&mut td);
        assert_eq!(td.val_str, data.as_str());

        My_Struct_SetOptStringRef(&mut td, str_data3.as_ptr());
        assert_eq!(td.val_str, str_data3.to_str().unwrap());

        let data = My_Struct_GetOptString(&mut td);
        assert_eq!(td.val_str, data.as_str());
    }
}

#[test]
#[should_panic]
fn test_impl_return_error() {
    unsafe {
        My_Struct_RetResErr();
    }
}
