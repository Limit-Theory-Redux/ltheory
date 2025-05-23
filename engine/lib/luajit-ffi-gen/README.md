# Attribute macro for LuaJIT FFI binding generation

This crate provides an attribute macro for generation of C/Lua API bindings.

## Usage with the impl blocks

[Attributes](#luajit_ffi-attributes-for-impl-block).

Example:
```rust
pub struct MyStruct {
    val_u32: u32,
}

#[luajit_ffi_gen::luajit_ffi(name = "My_Struct", meta = true)]
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
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct My_Struct {} My_Struct;
    ]]

    return 1, 'My_Struct'
end

function Loader.defineType()
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
end

return Loader
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

By default all generated Lua code created in the **phx/script/ffi_gen** folder. User can manually set this folder via **LUAJIT_FFI_GEN_DIR** environment variable. Path should be either absolute or relative to the **luajit_ffi_gen** folder.

## Usage with the enums

Macro can be applied to the enum types (see **./tests/test_enum.rs** for examples):

[Attributes](#luajit_ffi-attributes-for-enum-block).

```rust
#[luajit_ffi_gen::luajit_ffi(name = "My_Enum1", start_index = 3, lua_ffi = false)]
#[derive(Debug)]
pub enum MyEnum1 {
    Var1,
    Var2,
}

#[luajit_ffi_gen::luajit_ffi(lua_ffi = false)]
#[derive(Debug)]
pub enum MyEnum2 {
    Var1 = 1,
    Var2 = 3,
}

const VALUE1: u16 = 101;
const VALUE2: u16 = 42;

#[luajit_ffi_gen::luajit_ffi(repr = "u16", lua_ffi = false)]
#[derive(Debug)]
pub enum MyEnum3 {
    Var1 = VALUE1,
    Var2 = VALUE2,
}

#[luajit_ffi_gen::luajit_ffi(repr = "u16", lua_ffi = false)]
#[derive(Debug)]
pub enum MyEnum4 {
    Var1 = VALUE1,
    Var2 = VALUE2,
    Var3 = 11,
}
```

This will generate following C API wrappers:
```rust

#[no_mangle]
pub extern "C" fn MyEnum1_ToString(this: MyEnum1) -> *const libc::c_char {
    // ...
}

#[no_mangle]
pub extern "C" fn MyEnum2_ToString(this: MyEnum2) -> *const libc::c_char {
    // ...
}

#[no_mangle]
pub const My_Enum3_Var1: u16 = MyEnum1::Var1.value();

#[no_mangle]
pub const My_Enum3_Var2: u16 = MyEnum1::Var2.value();

#[no_mangle]
pub extern "C" fn MyEnum3_ToString(this: MyEnum3) -> *const libc::c_char {
    // ...
}

#[no_mangle]
pub const MyEnum4_Var1: u16 = MyEnum4::Var1.value();

#[no_mangle]
pub const MyEnum4_Var2: u16 = MyEnum4::Var2.value();

#[no_mangle]
pub const MyEnum4_Var3: u16 = MyEnum4::Var2.value();

#[no_mangle]
pub extern "C" fn MyEnum4_ToString(this: MyEnum4) -> *const libc::c_char {
    // ...
}
```

and **My_Enum3.lua**:
```lua
-- My_Enum1 --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 My_Enum3;
    ]]

    return 2, 'My_Enum3'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('ffi.libphx').lib
    local My_Enum1

    do -- C Definitions
        ffi.cdef [[
            My_Enum3 My_Enum3_Var1;
            My_Enum3 My_Enum3_Var2;

            cstr     My_Enum3_ToString(My_Enum3);
        ]]
    end

    do -- Global Symbol Table
        My_Enum3 = {
            Var1     = libphx.My_Enum3_Var1,
            Var2     = libphx.My_Enum3_Var2,

            ToString = libphx.My_Enum3_ToString,
        }

        if onDef_My_Enum3 then onDef_My_Enum3(My_Enum3, mt) end
        My_Enum3 = setmetatable(My_Enum3, mt)
    end

    return My_Enum3
end

return Loader
```

Under the hood `ToString` trait is implemented for the enum so it should derive `Debug` to support that.

Variant values can be of 3 types:

1. No values. In this case all variants should not have values. Values vill be genarated automatically starting with `start_index` attribute or 0 if it's not set.
2. Numeric values, i.e. `Undefined = 0`. In this case value will be used as is.
3. Expression, i.e. `MyVariant = MyConst`. In this case `repr` attribute is mandatory.

Value and expression variants can be combined (see `MyEnum4` example above).

If enum contains at least 1 expression variant then C constants will be generated for them. Otherwise explicit values will be used in the generated Rust and Lua code.

For the variants without values starting index can be set, otherwise it starts from 0. See attribute parameters description below.

If `repr` parameter is set then `#[repr(...)]` attribute will be added with the specified type, otherwise type will be deducted from the variants values: `u8`, `u16`, etc.

### Joining enum and impl blocks

If it's required to expose both `enum` and its `impl` block then `with_impl` attribute should be used:

```rust
#[luajit_ffi_gen::luajit_ffi(with_impl = true)]
#[derive(Debug)]
pub enum MyEnum1 {
    Var1,
    Var2,
}

#[luajit_ffi_gen::luajit_ffi]
impl MyEnum1 {
    pub fn is_var1(&self) -> bool {
        *self == Self::Var1
    }
}
```

This will generate following C API wrappers:
```rust
#[no_mangle]
pub extern "C" fn MyEnum1_ToString(this: MyEnum1) -> *const libc::c_char {
    // ...
}

#[no_mangle]
pub extern "C" fn MyEnum1_IsVar1(this: &MyEnum1) -> bool {
    this.is_var1()
}
```

and **MyEnum1.lua**:
```lua
-- My_Enum1 --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 MyEnum1;
    ]]

    return 2, 'MyEnum1'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('ffi.libphx').lib
    local MyEnum1

    do -- C Definitions
        ffi.cdef [[
            cstr     MyEnum1_ToString(MyEnum1);

            bool MyEnum1_IsVar1(MyEnum1);
        ]]
    end

    do -- Global Symbol Table
        MyEnum1 = {
            Var1     = 0,
            Var2     = 1,

            ToString = libphx.MyEnum1_ToString,

            IsVar1 = libphx.MyEnum1_IsVar1,
        }

        if onDef_MyEnum1 then onDef_MyEnum1(MyEnum1, mt) end
        MyEnum1 = setmetatable(MyEnum1, mt)
    end

    return MyEnum1
end

return Loader
```

Take in account that `enum` block should be defined before `impl` otherwise `enum` data will be lost. Also `opaque` parameter doesn't have any effect in this case.

Under the hood proc macro on the `enum` block saves all necessary information in JSON file in **target/ffi** folder instead of generating ***.lua** script. This information is merged later by proc macro on `impl` block.

## Attribute parameters

### luajit_ffi attributes for `impl` block

- **name** \[string, default = None]: set user defined name of the module
- **forward_decl** \[string, default = ""]: set user defined list of comma separated type names that should be pre-declared before current type. Used only when **typedef** argument is set. See `Collision` type for example.
- **typedef** \[string, default = ""]: set user defined structure fields. Use '\n' to separate multiple fields. Example:
    ```
    float x;
    float y;
    float z;
    float w;
    ```
    Do not forget to add `#[repr(C)]` declaration to the structure to specify proper memory layout.
- **opaque** \[bool, default = true]: generate **typedef** C API module structure definition.
- **clone** \[bool, default = false]: adds **__call** method to Global Symbol Table section and **clone** method to metatype section.
- **lua_ffi** \[bool, default = true]: specify if Lua FFI file should be generated or only C API.
- **gen_dir** \[string]: folder where generated lua file should be put. Default: ../phx/script/ffi_gen.
- **meta_dir** \[string]: folder where generated lua meta file should be put. Default: ../phx/script/meta.

### luajit_ffi attributes for `enum` block

- **name** \[string, default = None]: optional object name. If not specified then name is taken from the `impl` definition.
- **repr** \[string, default = None]: specify what type will be used in `#[repr(...)]` attribute that will be added to the enum definition. If not set then type will be deducted from the maximal discriminant: u8, u16, u32 or u64.
- **start_index** \[int, default = None]: set starting index for discriminant values. Ignored if enum already has discriminants. Default: 0.
- **lua_ffi** \[bool, default = true]: specify if Lua FFI file should be generated or only C API.
- **with_impl** \[bool, default = false]: specify if enum has connected implementation block.
- **gen_dir** \[string]: folder where generated lua file should be put. Default: ../phx/script/ffi_gen.
- **meta_dir** \[string]: folder where generated lua meta file should be put. Default: ../phx/script/meta.

### bind
- **name** [string] - set user defined name of the function.
- **role** [enum: constructor, to_string] - set function role.
- **constructor** - function won't appear in the metatype section.
- **to_string** - will generate a binding in the metatype section.
- **out_param** - return value via an out parameter at the end of the function signature.

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
cargo expand -p luajit-ffi-gen --test test_impl
```

## Supported types

**luajit_ffi_gen** attribute supports different types as input parameters and return type in the impl methods.

Following table shows representation of Rust types in the generated code.

Glossary:
1. **CT** - copyable type (including primitives).
2. **MT** - managed type.
3. **T** - any type.

### Input position

List of allowed types in the input parameter position.

| Rust type                      | extern "C" interface       | C type            |
| ------------------------------ | -------------------------- | ----------------- |
| By value (MT) (not working!)   | Box\<MT>                   | MT*               |
| By value (CT)                  | CT                         | CT                |
| Immutable reference (&T)       | &T                         | T const*          |
| Mutable reference (&mut T)     | &mut T                     | T*                |
| String, &str                   | \*const/mut libc::c_char   | cstr              |
| Option\<MT>                    | Option\<Box\<MT>>          | MT*               |
| Option\<CT>                    | Option\<&CT>               | CT const*         |
| Option\<String>, Option\<&str> | \*const libc::c_char       | cstr              |
| &[T]                           | \*const T, usize           | T const *, uint64 |
| &mut [T]                       | \*mut T, usize             | T*, uint64        |
| &[T; N]                        | \*const T, usize           | T const *, uint64 |
| [T; N], &mut [T; N]            | \*mut T, usize             | T*, uint64        |
| trait FnOnce/Fn/FnMut          | extern fn(T1, T2, ..) -> R | R (*)(T1, T2, ..) |

### Return position

List of allowed types in the return parameter position.

| Rust type                     | extern "C" interface | C type    | Buffered |
| ----------------------------- | -------------------- | --------- | -------- |
| By value (MT)                 | Box\<MT>             | MT*       |          |
| By value (CT)                 | CT                   | CT        |          |
| &T                            | \*const T            | T const*  |          |
| &mut T                        | \*T                  | T*        |          |
| String, &str                  | \*const libc::c_char | cstr      | yes      |
| Option\<MT>                   | Option\<Box\<MT>>    | MT*       |          |
| Option\<CT>                   | \*const CT           | CT const* | yes      |
| Option\<String>, Option<&str> | \*const libc::c_char | cstr      | yes      |
| Result\<..., E>               | [1], panic on error  | [1]       | [1]      |
| Result\<Option\<...>, E>      | [2], panic on error  | [2]       | [2]      |

[1] - same as for standalone by value variants.
[2] - same as for standalone Option variants.

For the buffered types static buffer is created in the `extern "C"` wrapper function.

### (Mutable) references

In the generated C API references are used as is: **&** or **&mut**. Exceptions are only strings and options.

### Self

Self as input parameter is expected as reference (**&self**) or mutable reference (**&mut self**).

In the return position **Self** is boxed: Box\<T>.

### Basic and copy types

Basic (bool, i32, u64, f32, etc.) and copy types (defined in **COPY_TYPES** map) are sent as-is via the C API.

### Strings

To avoid additional copies, C strings in the method parameter position is converted unsafely to a Rust string.

When returning, Rust strings are converted to CStr and copied to a static scoped buffer. A pointer to this buffer is returned via the C API.

### User defined types (non copyable)

User types in method parameters position are sent by reference or mutable reference. When returning, the type is boxed to transfer ownership to the caller (Lua).

**TODO**: should we accept user types in parameters position boxed as well?

### Option

Returned as a **\*mut T**, and **None** is interpreted as **NULL** pointer. For copyable types, we pin the result in a static and return a reference to it.

### Result

Accepted only in the return position. Panics on error.

### Function objects

When the Rust code expects a FnOnce/Fn/FnMut trait object, the generated C API expects a function pointer. We then wrap that function pointer in a Rust closure which converts each argument from the Rust type to the C type, then invokes the C function pointer. The return type from the C function pointer is then converted back to the Rust equivalent type, if required.

## Lua definition files

Procedural macro also generates [Lua definition files](https://luals.github.io/wiki/definition-files/) to support Language Server.

By default all definition files go the **engine/lib/phx/script/meta** folder. This can be changed by setting **LUAJIT_FFI_GEN_DIR** environment variable during compilation.

Path to the definition files is configured in **.luarc.json** file.

All the comments to the methods in the Rust `impl` blocks with `#[luajit_ffi_gen::luajit_ffi]` attribute are copied to the definition files together with method parameters and return value description.

See files in the **engine/lib/phx/script/meta** folder for examples of the generated definition files.

---
## Tips

If there are any compilation errors in the generated C API bindings, developer can copy **#[no_mangle]** code generated by **cargo expand** into the *.rs file and disable **#[luajit_ffi_gen]** attribute.

This way it will be much easier to spot the place of the problem.

## Optimization ideas

Compilation time can significantly increase after utilizing the **luajit_ffi_gen** attribute.

One optimization is to disable Lua definition files generation in CI. This can be done by introducing cargo feature flag, i.e. **no-lua-ls-defs**.

Code generation can be optimized in 2 stages:

### Stage 1. Regenerate Lua FFI only if Rust code changes

To avoid unnecessary Lua FFI files regeneration we can calculate the hash of the [ImplInfo] and [EnumInfo] structures and store it in a file. So before generating the Lua FFI code, we can check to see if it's hash changed first before regenerating.

The hash file can be stored either in a subfolder of the **target** directory, or in git. In the former case, improvement will be visible only during an incremental build, but the latter will help during CI as well.

### Stage 2. Regenerate C API bindings only if Rust code changes

If the optimization steps from the stage 1 is not enough, a similar approach can be applied to the generated C API code.

In this case it should be placed in a different location to the hash of the Lua FFI code.

Here we also do regeneration of the C API file only if the hash of the [ImplInfo] and [EnumInfo] structures were changed.
