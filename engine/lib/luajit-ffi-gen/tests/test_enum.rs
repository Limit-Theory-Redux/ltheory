mod utils;

use luajit_ffi_gen::luajit_ffi;

#[luajit_ffi(name = "My_Enum1", start_index = 3, lua_ffi = false)]
#[derive(Debug)]
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

#[test]
fn test_enum_to_string() {
    assert_eq!(MyEnum2::Var1.to_string(), "Var1");
}

#[test]
fn test_enum_value() {
    assert_eq!(MyEnum1::Var2.value(), 4);
}
