---
name: lua-scripting
description: Conventions of the Lua game layer in script/ — boot sequence (Main.lua/Init.lua), directory roles, app states, globals, and class utilities. Use when writing or modifying game-side Lua code.
---

# Lua Game Layer

All game logic lives in `script/` and runs on LuaJIT inside the engine process. The engine calls `script/Main.lua` (default entry point), which requires `script/Init.lua` and then boots an app state.

## Boot sequence

1. `Main.lua` extends `package.path` (note: `?/__init__.lua`, `?.ext.lua`, `?.ffi.lua` variants) and requires `Init`.
2. `Init.lua` loads `ffi`/`jit`/`lfs`, disables JIT on macOS ARM64, dumps `math` into globals, requires `Globals`, Lua extensions (`Core.LuaExtensions.*`: ToString, IOEx, StringEx, TableEx, TypeEx), and namespaces `Core.Struct`, `Core.Util`, `Core.Events`.
3. `SetEngine()` (called from Rust) sets globals: `Engine`, `EventBus`, `TaskQueue`, `Input`, `Window`, `Gui`.
4. `InitSystem()` enables `GlobalRestrict` (accidental global writes error out), checks engine/script version match, loads `script/Config/`, `script/Enums/`, `script/Types/`, and the HmGui namespaces.
5. The app state named by `__app__` (CLI positional arg, default `LTheoryRedux`) is required from `States.App.<name>` or `States.App.Tests.<name>` and its lifecycle callbacks run.

## Directory roles

```
script/Config/      Configuration files, loaded at boot (App.lua first).
script/Core/        Framework: ECS core, class system, structures, util, Lua extensions.
script/Enums/       Enum tables, loaded globally at boot.
script/Types/       Type definitions, loaded at boot.
script/Globals.lua  Global variables.
script/Modules/     ECS modules (Entities/Components/Systems) — see `ecs` skill.
script/States/      Application.lua base + App/ states (the runnable "apps").
script/States/App/Tests/  Runnable test/demo apps (RenderingTest, PhysicsTest, ...).
script/UI/          HmGui-based UI (see `hmgui` skill).
script/Shared/      Registries (e.g. Items), helpers, tools shared across modules.
script/Render/      Lua-side rendering pipeline code.
script/Legacy/      Old code being phased out (e.g. event systems → Rust event bus).
```

## App states

An app is a table derived from `script/States/Application.lua` implementing lifecycle callbacks (`onInit`, `onInput`, `onUpdate`, `onDraw`, ...). Copy an existing test app in `script/States/App/Tests/` as a template; the filename (without `.lua`) is the name passed on the command line (`cargo run -- MyTest`).

## Conventions

- Classes use the global `Class(name, ctor)` / `Subclass(name, parent, ctor)` helpers; annotate with Lua LS `---@class`/`---@overload` comments (see any component in `script/Modules/`).
- Engine events: subscribe via `EventBus:subscribe(Event.PreRender, self, self.onPreRender)`.
- Logging: `Log.Debug/Info/Warn/Error` with printf-style formatting.
- Engine FFI types have Lua LS annotations in `engine/lib/phx/script/meta/` (wired via `.luarc.json`) — respect the annotated signatures.
- `GlobalRestrict` is on: declare locals; intentional globals go in `Globals.lua`.
