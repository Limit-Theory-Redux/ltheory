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

If same structure needs other methods that should not be exposed as C API, just put them in another **impl** block.

There are 2 lists in [src/method_info.rs] file that contain additional configuration:
- **RUST_TO_LUA_TYPE_MAP** - maps Rust types to Lua ones
- **COPY_TYPES** - types that are passed as is in and out of C function bindings

Extend these lists with necessary data.

In all other cases types are following these rules:
- **bool** and **number** types  are passed always by value unless it's **&mut**
- **&str**/**String** is converted to **\*const libc::c_char**
- all other types are accepted either as **&** or **&mut** into the C wrapper, and are boxed (**Box\<T\>**) as outer

By default all generated Lua code created in the **phx/script/ffi** folder. User can manually set this folder via **LUAJIT_FFI_GEN_DIR** environment variable. Path should be either absolute or relative to the **luajit_ffi_gen** folder.

## Attribute parameters

### luajit_ffi

- **name** [string, default = None] - set user defined name of the module
- **managed** [bool, default = false] - generate **Free** C API function and add **managed** and **free** metatype bindings
- **clone** [bool, default = false] - adds **__call** method to Global Symbol Table section and **clone** method to metatype section
- **lua_ffi** [bool, default = true] - specify if Lua FFI file should be generated or only C API

### bind
- **name** [string] - set user defined name of the function
- **role** [enum: constructor, to_string] - set function role.
  - **constructor** - function won't appear in the metatype section
  - **to_string** - will generate a binding in the metatype section

## Macro expansion

Install **cargo-expand**:
```bash
cargo install cargo-expand
```

To expand attribute macro and see i.e. window.rs expansion, run:
```bash
cargo expand -p phx system::window
```

for test:
```bash
cargo expand -p luajit-ffi-gen --test basic_test
```

## Supported types

**luajit_ffi_gen** attribute supports different types as input parameters and return type in the impl methods.

Following table shows representation of Rust types in the generated code.

| Rust type                            | extern "C" interface         | C type |
| ------------------------------------ | ------------------- | ---------- |
| Immutable reference (&T)             | &T                  | T const*   |
| Mutable reference (&mut T)           | &mut T              | T *        |
| Self (in return position)            | Box\<T>             | T *        |
| Basic and copy types (T)             | T                   | T          |
| String, str                          | *const libc::c_char | cstr       |
| Option\<T>                           | *mut T              | T *        |
| Result\<T, E> (only return position) | T, panic on error   | T          |

### (Mutable) references

In the generated C API references are used as is: **&** or **&mut**. Exceptions are only strings and options.

### Self

Self as input parameter is expected as reference (**&self**) or mutable reference (**&mut self**).

In the return position **Self** is boxed: Box\<T>.

### Basic and copy types

Basic (bool, i32, u64, f32, etc.) and copy types (defined in **COPY_TYPES** map) are sent as is via the C API.

### Strings

To avoid additional copies, C strings in the method parameter position is converted unsafely to a Rust string.

When returning, Rust strings are converted to CStr and copied to a static scoped buffer. A pointer to this buffer is returned via the C API.

### User defined types (non copyable)

User types in method parameters position are sent by reference or mutable reference. When returning, the type is boxed to transfer ownership to the caller (Lua).

**TODO**: should we accept user types in parameters position boxed as well?

### Option

Returned as a **\*mut T**, and **None** is nterpreted as **NULL** pointer.

### Result

Accepted only in the return position. Panics on error.

---
## Tip

If there are any compilation errors in the generated C API bindings, developer can copy **#[no_mangle]** code generated by **cargo expand** into the *.rs file and disable **#[luajit_ffi_gen]** attribute.

This way it will be much easier to spot the place of the problem.

## Optimization ideas

If after converting engine to use **luajit_ffi_gen** attribute everywhere compilation time significantly increases, code generation can be optimized in 2 stages.

### Stage 1. Regenerate Lua FFI only if Rust code changes

To avoid unnecessary Lua FFI files regeneration we can calculate hash of the [ImplInfo] structure (actually only name and methods fields) and store it in a file. So before generating Lua FFI file we can check if it's hash changed and do regeneration only if it does.

File with a hash can be stored either in **target** folder subfolder or in a git. For the former improvement will be visible only for the incremental build. The latter will help CI as well.

### Stage 2. Regenerate C API bindings only if Rust code changes

If optimization from the stage 1 is not enough similar approach can be applied to the generated C API code.

In this case it should not be put in the same file where **luajit_ffi_gen** was added but in a separate one.

Here we also do regeneration of the C API file only if hash of the [ImplInfo] structure was changed.
