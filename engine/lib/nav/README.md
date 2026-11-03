# nav

Pure navigation algorithms in Rust: no FFI, no engine types, no game vocabulary. The crate depends on external crates only, so its tests and benchmarks need no window. The engine (`phx`) wraps it for Lua as `NavWorld`; the game decides what ships do and Lua calls in for the maths.

`World` holds agents and obstacles and, per step, gives each agent normalised control axes: Arrive, sphere avoidance and a damping controller. How they work, and their limits: [`doc/engine/nav.md`](../../../doc/engine/nav.md). `tests/flight.rs` flies agents through a simple ship model, and `cargo bench -p nav --bench step` measures a step.

## Testing Lua kernels

Performance-sensitive Lua is written as pure kernels (plain numbers and `double[?]` buffers, no entities, no globals) in files of their own under `script/Shared/Helpers/`. Tests in `tests/` load a kernel through `mlua` into a LuaJIT state and run it, and compare it with its Rust version once one exists.

`tests/common/mod.rs` is the harness:
- `KernelVm::new(extra_dir)` creates a state whose `require` finds modules under `script/` (and under `extra_dir`, for test fixtures).
- `_G` is locked before the kernel loads: reading an undefined global or writing a new one raises an error. A kernel that needs an engine global fails its test.
- `buffer(&[f64])` makes a `double[?]` and `read_buffer(&buffer, n)` reads one back.

To test a kernel, `mod common;` in a new file in `tests/`, `require` the kernel by module name, and call its functions with buffers. `tests/kernel_harness.rs` shows the pattern with the fixture kernels in `tests/fixtures/`.
