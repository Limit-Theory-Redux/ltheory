mod utils;

use luajit_ffi_gen::luajit_ffi;

#[luajit_ffi(name = "My_Enum1", start_index = 3, lua_ffi = false)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyEnum1 {
    Var1,
    Var2,
}

#[luajit_ffi(repr = "u32", lua_ffi = false)]
#[derive(Debug)]
pub enum MyEnum2 {
    Var1 = 1,
    Var2 = 3,
}

pub struct MyStruct1 {
    my_enum: MyEnum1,
}

#[luajit_ffi(name = "My_Struct1", lua_ffi = false)]
impl MyStruct1 {
    pub fn new(my_enum: &MyEnum1) -> Self {
        Self { my_enum: *my_enum }
    }

    pub fn get(&self) -> MyEnum1 {
        self.my_enum
    }
}

#[test]
fn test_enum_to_string() {
    assert_eq!(MyEnum2::Var1.to_string(), "Var1");
}

#[test]
fn test_enum_value() {
    assert_eq!(MyEnum1::Var2.value(), 4);
}

#[test]
fn test_enum_in_struct() {
    let s = MyStruct1::new(&MyEnum1::Var1);

    assert_eq!(s.get(), MyEnum1::Var1);
}
