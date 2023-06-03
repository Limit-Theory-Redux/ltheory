use luajit_ffi_gen::luajit_ffi;

#[derive(Default, Clone)]
pub struct Data {
    pub val: bool,
}

#[derive(Default)]
pub struct MyStruct {
    val_u32: u32,
    val_f32: f32,
    // val_str: String,
    val_data: Data,
}

#[luajit_ffi(name = "My_Struct")]
impl MyStruct {
    pub fn func1(&self) {}
    pub fn func2(&mut self) {}

    #[bind = "Func3"]
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

    // pub fn set_str(&mut self, val: &str) {
    //     self.val_str = val.into();
    // }

    // pub fn get_str(&self) -> &str {
    //     &self.val_str
    // }

    pub fn set_data(&mut self, val: &Data) {
        self.val_data = val.clone();
        self.val_data.val = true;
    }

    pub fn get_data(&self) -> &Data {
        &self.val_data
    }
}

#[test]
fn basic_test() {
    let ms = MyStruct::default();
    let mut ms2 = MyStruct::default();

    ms.func1();
    ms2.func2();
    MyStruct::func3();

    MyStruct_func1(&ms);
    MyStruct_func2(&mut ms2);
    MyStruct_Func3();

    MyStruct_set_u32(&mut ms2, 33);
    assert_eq!(MyStruct_get_u32(&mut ms2), 33);

    MyStruct_set_f32(&mut ms2, 33.0);
    assert_eq!(MyStruct_get_f32(&mut ms2), 33.0);

    MyStruct_set_data(&mut ms2, &Data::default());
    assert!(MyStruct_get_data(&mut ms2).val);
}
