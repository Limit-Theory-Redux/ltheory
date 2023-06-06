# Attribute macro for LuaJIT FFI binding generation

To expand attribute macro and see i.e. window.rs expansion, run:
```bash
cargo expand -p phx window
```

for test:
```bash
cargo expand -p luajit_ffi_gen --test basic_test
```

## Type mapping

| Rust                | Wrapper in               | Wrapper out              | Lua ffi.cdef              |
| ------------------- | ------------------------ | ------------------------ | ------------------------- |
| bool                | bool                     | bool                     | bool                      |
| i8, u8              | i8, u8                   | i8, u8                   | int8, uint8               |
| i16, u16            | i16, u16                 | i16, u16                 | int16, uint16             |
| i32, u32            | i32, u32                 | i32, u32                 | int, uint32               |
| i64, u64            | i64, u64                 | i64, u64                 | int64, uint64             |
| &str, String        | *const libc::c_char      | *const libc::c_char      | cstr                      |
| [registered type]   | [registered type]        | [registered type]        | [registered type map]     |
| [unregistered type] | *[unregistered type map] | Box<[unregistered type]> | [unregistered type map]\* |
