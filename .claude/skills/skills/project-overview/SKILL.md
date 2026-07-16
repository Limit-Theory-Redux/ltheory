---
name: project-overview
description: Codebase map for Limit Theory Redux — workspace layout, how the Rust engine and Lua game layer interact, and where to find docs. Read this first when orienting in the repository.
---

# Project Overview

Limit Theory Redux (LTR) is a fork of the cancelled open-world space simulation game Limit Theory. The original C/C++ engine (LibPHX) has been ported to Rust; game logic lives in Lua and runs on LuaJIT. The engine is a *library*, not a framework: control flow lives in Lua scripts, which make zero-overhead FFI calls into the Rust engine for heavy computation.

## Workspace layout

```
engine/bin/ltr/            Executable. Parses CLI args, dlopens libphx, calls Engine_Entry.
engine/lib/phx/            The engine ("PHX") — a cdylib exposing a C API to LuaJIT.
  src/audio|engine|input|math|physics|render|system|ui|window/
  script/ffi_gen/          GENERATED Lua FFI loaders (one per exposed Rust type).
  script/ffi_ext/          Hand-written Lua extensions to generated types.
  script/meta/             GENERATED Lua LS definition files (annotations only).
engine/lib/luajit-ffi-gen/ Proc-macro crate that generates the C API + Lua files above.
engine/lib/internal/       Shared internal helpers (memory etc.).
script/                    The game, in Lua. Entry point: script/Main.lua.
res/                       Assets: shaders, meshes, fonts, textures, sounds.
doc/                       Documentation index: doc/README.md.
ai/                        Notes and skills for AI assistants (this folder).
```

## How the layers connect

1. `ltr` starts the engine (`Engine_Entry` in `engine/lib/phx/src/engine/`), which creates a LuaJIT state and runs `script/Main.lua` (configurable via `--entry-point`).
2. Rust types are exposed with the `#[luajit_ffi_gen::luajit_ffi]` attribute macro, which generates `extern "C"` wrappers plus the Lua loader files in `engine/lib/phx/script/ffi_gen/` and annotations in `script/meta/`. See the `luajit-ffi-bindings` skill.
3. Lua receives the `Engine` pointer via `SetEngine()` in `script/Main.lua` and pulls globals from it: `EventBus`, `TaskQueue`, `Input`, `Window`, `Gui` (HmGui).
4. The game selects an "app state" by name (positional CLI arg, default `LTheoryRedux`) from `script/States/App/` or `script/States/App/Tests/`.

## Key subsystems and their skills

- Building, running, formatting → `build-and-run`
- Rust↔Lua bindings → `luajit-ffi-bindings`
- Lua game-layer conventions → `lua-scripting`
- Entity Component System (Lua-side) → `ecs`
- Rendering & the render thread → `rendering`
- Lua worker threads → `lua-workers`
- HmGui UI framework → `hmgui`

## Documentation entry points

- `doc/README.md` — index of all docs.
- `engine/lib/phx/README.md` — LibPHX philosophy.
- `CONTRIBUTING.md` — licensing (Unlicense for original content, MIT/Apache-2.0 dual for new) and workflow; discussion happens on Discord.
