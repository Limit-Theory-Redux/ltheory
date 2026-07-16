---
name: luajit-ffi-bindings
description: How to expose Rust engine APIs to Lua with the #[luajit_ffi_gen::luajit_ffi] macro — attribute parameters, supported types, generated files, and the regeneration workflow. Use when adding or changing any Rust function/enum that Lua must call.
---

# LuaJIT FFI Bindings (luajit-ffi-gen)

The `engine/lib/luajit-ffi-gen` proc-macro crate turns annotated Rust `impl` blocks and enums into:

1. `extern "C"` wrapper functions (PascalCase, e.g. `fn set_u32` → `MyStruct_SetU32`),
2. a Lua loader file in `engine/lib/phx/script/ffi_gen/<Name>.lua` (ffi.cdef + symbol table),
3. a Lua Language Server definition file in `engine/lib/phx/script/meta/<Name>.lua` (doc comments on the Rust methods are copied here).

Full reference: `engine/lib/luajit-ffi-gen/README.md`. Working examples: `engine/lib/luajit-ffi-gen/tests/`.

## Basic usage

```rust
#[luajit_ffi_gen::luajit_ffi(name = "My_Struct", meta = true)]
impl MyStruct {
    pub fn set_u32(&mut self, val: u32) { ... }

    #[bind(name = "GetValue")]        // override generated name
    pub fn get_u32(&self) -> u32 { ... }
}
```

Methods that should NOT be exposed go in a separate plain `impl` block. Enums are also supported (`#[luajit_ffi_gen::luajit_ffi]` on the enum; use `with_impl = true` to combine an enum with an exposed impl block — the enum must be defined before the impl).

## Key attribute parameters

`impl` block: `name` (Lua module name), `opaque` (default true — emit opaque typedef), `typedef` (explicit C struct fields; requires `#[repr(C)]`), `forward_decl`, `clone` (adds `__call`/`clone`), `lua_ffi` (default true; false = C API only), `gen_dir`/`meta_dir` (output overrides).

`enum` block: `name`, `repr` (sets `#[repr(...)]`; mandatory when variants use const expressions), `start_index`, `with_impl`, `lua_ffi`.

`#[bind(...)]` on methods: `name`, `role = constructor` (kept out of metatype section), `role = to_string`, `out_param` (return via out-parameter).

## Type rules (summary)

- Primitives and `COPY_TYPES` pass by value; other types pass as `&`/`&mut` and are returned boxed (`Box<T>`, ownership goes to Lua).
- `String`/`&str` ⇄ `*const libc::c_char`; returned strings are copied into a static buffer.
- `Option<T>`: `None` ⇄ NULL pointer. `Result<T, E>`: return position only, panics on error.
- Slices/arrays become pointer + length; `Fn`/`FnMut`/`FnOnce` parameters become C function pointers.
- To support a new type, extend `RUST_TO_LUA_TYPE_MAP` / `COPY_TYPES` in `engine/lib/luajit-ffi-gen/src/method_info.rs`.

## Workflow when changing an exposed API

1. Edit the Rust code inside the `#[luajit_ffi_gen::luajit_ffi]` impl/enum.
2. `cargo build -p phx` — this regenerates `script/ffi_gen/*.lua` and `script/meta/*.lua` (output dir overridable with the `LUAJIT_FFI_GEN_DIR` env var).
3. Commit the regenerated Lua files together with the Rust change. Never hand-edit files in `ffi_gen/` or `meta/`; hand-written additions belong in `engine/lib/phx/script/ffi_ext/`.
4. Update Lua call sites (methods are camelCase on instances, e.g. `Engine:taskQueue()`).

## Debugging macro issues

Install `cargo-expand`, then e.g. `cargo expand -p phx system::window` or `cargo expand -p luajit-ffi-gen --test test_impl` to inspect generated code. If generated C API code fails to compile, paste the expanded `#[no_mangle]` code into the source temporarily to locate the error.
