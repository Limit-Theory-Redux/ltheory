---
name: build-and-run
description: How to build, run, test, and format Limit Theory Redux — cargo commands, build.sh, ltr CLI flags, app selection, and lint rules. Use when compiling the project, launching the game or a test app, or verifying changes.
---

# Build, Run, Test, Format

## Build

Standard cargo workspace (edition 2024, toolchain pinned in `rust-toolchain.toml`):

```bash
cargo build              # debug; external deps still built with opt-level 3
cargo build --release
./build.sh               # release build + packaging (flags: --run-tests, --debug, --bundle)
```

On Windows (msys) `LIBCLANG_PATH` must be set for bindgen. `--bundle` only affects macOS.

## Run

The executable is `ltr` (`engine/bin/ltr`). It loads `libphx` as a dynamic library, so run from the repo root so relative paths (`./script/Main.lua`, `res/`) resolve:

```bash
cargo run                          # runs default app: LTheoryRedux
cargo run -- <AppName>             # positional arg selects app state
cargo run -- RenderingTest         # e.g. a test app
```

App names resolve to `script/States/App/<AppName>.lua`, falling back to `script/States/App/Tests/<AppName>.lua`. Subdirectories use `/` (e.g. `ECS/UniverseCreationTest`).

Useful flags (see `engine/bin/ltr/src/main.rs`):

- `-e, --entry-point <path>` — starting Lua script (default `./script/Main.lua`)
- `-c, --console-log <bool>` — console logging (default true)
- `-l, --log-dir <dir>` — also write log to file
- `-r, --render-thread` — enable the multithreaded renderer (default off; see `rendering` skill)
- `-n, --no-color` — disable colored output

## Test

```bash
cargo test                                  # Rust tests
cargo test -p luajit-ffi-gen                # FFI generator tests (tests/ has good examples)
```

Caveat: `build.sh --run-tests` disables tests on Linux ("tests are currently not working correctly on Linux").

## Format & lints

- `./format.sh` runs `cargo +nightly fmt` (nightly rustfmt required; config in `rustfmt.toml`).
- Workspace denies `unsafe_code` (`Cargo.toml`); crates/files that need it use explicit `#[allow(unsafe_code)]`. Don't introduce unsafe casually.
- Run `cargo clippy` before committing Rust changes.

## After changing FFI-exposed Rust code

A build regenerates Lua files in `engine/lib/phx/script/ffi_gen/` and `script/meta/`. These are committed to git — include the regenerated files in the same commit as the Rust change (see `luajit-ffi-bindings` skill).
