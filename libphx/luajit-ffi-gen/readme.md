# Attribute macro for LuaJIT FFI binding generation

This crate provides an attribute macro for generation of C/Lua API bindings.

It should be applied to the `impl` block.

Example:
```rust
pub struct MyStruct {
    val_u32: u32,
}

#[luajit_ffi_gen::luajit_ffi(name = "My_Struct", meta = true, managed = true)]
impl MyStruct {
    pub fn set_u32(&mut self, val: u32) {
        self.val_u32 = val;
    }

    #[bind(name = "FUNC3")]
    pub fn get_u32(&self) -> u32 {
        self.val_u32
    }
}
```

This will generate following C API wrappers:
```rust
#[no_mangle]
pub extern "C" fn MyStruct_SetU32(this: &mut MyStruct, val: u32) {
    this.set_u32(val);
}
#[no_mangle]
pub extern "C" fn MyStruct_FUNC3(this: &MyStruct) -> u32 {
    this.get_u32()
}
```

and **My_Struct.lua**:
```lua
-- My_Struct -------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local My_Struct

do -- C Definitions
  ffi.cdef [[
    void   My_Struct_SetU32 (My_Struct*, uint32 val);
    uint32 My_Struct_FUNC3  (My_Struct*);
  ]]
end

do -- Global Symbol Table
  My_Struct = {
    SetU32 = libphx.My_Struct_SetU32,
    FUNC3  = libphx.My_Struct_FUNC3,
  }

  if onDef_My_Struct then onDef_My_Struct(My_Struct, mt) end
  My_Struct = setmetatable(My_Struct, mt)
end

return My_Struct
```

By default function names are converted to camel case but it is possible to set a user defined names. See **Attribute parameters** section below for details.

There are 2 lists in [src/method_info.rs] file that contain additional configuration:
- **RUST_TO_LUA_TYPE_MAP** - maps Rust types to Lua ones
- **COPY_TYPES** - types that are passed as is in and out of C function bindings

Extend these lists with necessary data.

In all other cases types are following these rules:
- **bool** and **number** types  are passed always by value unless it's **&mut**
- **&str**/**String** is converted to **\*const libc::c_char**
- all other types are accepted either as **&** or **&mut** into the C wrapper, and are boxed (**Box\<T\>**) as outer

By default all generated Lua code created in the **libphx/script/ffi** folder. User can manually set this folder via **LUAJIT_FFI_GEN_DIR** environment variable. Path should be either absolute or relative to the **libphx/luajit_ffi_gen** folder.

## Attribute parameters

### luajit_ffi

- **name** [string] - set user defined name of the module
- **meta** [bool] - generate Metatype section in the Lua FFI file
- **managed** [bool] - generate **Free** C API function and add **managed** and **free** metatype bindings
- **clone** [bool] - adds **__call** method to Global Symbol Table section and **clone** method to metatype section
- **no_lua_ffi** [bool] - this is used in tests only to disable Lua FFI file generation

### bind
- **name** [string] - set user defined name of the function
- **role** [enum: constructor, to_string] - set function role.
  - **constructor** - function won't appear in the metatype section
  - **to_string** - will generate a binding in the metatype section

## Macro expansion

To expand attribute macro and see i.e. window.rs expansion, run:
```bash
cargo expand -p phx window
```

for test:
```bash
cargo expand -p luajit_ffi_gen --test basic_test
```
