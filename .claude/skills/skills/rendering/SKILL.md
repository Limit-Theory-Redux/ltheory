---
name: rendering
description: Render architecture — the classic direct-GL path, the render module layout, and the in-progress multithreaded command-buffer render thread (Renderer/RenderCommand/RenderThread) and how to activate it. Use when touching engine/lib/phx/src/render or Lua draw code.
---

# Rendering

All rendering code lives in `engine/lib/phx/src/render/`. Two paths exist:

## 1. Classic direct-GL path (the one actually live)

Lua drives GL directly on the main thread through FFI-exposed types: `Draw`, `Shader`/`ShaderState`, `Tex1d/2d/3d/Cube`, `Mesh`, `RenderState`, `RenderTarget`, `Viewport`, `Font`, `Color`, etc. (one Rust file per type in `render/`). Shaders/assets are in `res/shader/`, meshes in `res/mesh/`. Lua-side pipeline code is in `script/Render/`.

## 2. Multithreaded render thread (branch `feat/multithreaded_rendering`, dormant by default)

A command-buffer renderer in `engine/lib/phx/src/render/thread/`. Status and completion plan: `ai/multithreaded_rendering.md` — read it before working here.

```
Main thread (game + Lua)                      Render thread (owns GL context)
Renderer.submit(RenderCommand) ─► bounded crossbeam channel ─► RenderThread.run() → gl::*
        fence/sync, shader-reload results, context handoff flow back over channels
```

Key pieces:

- `renderer.rs` — main-thread `Renderer` handle: spawns the `"RenderThread"` OS thread, moves the `WindowGlContext` into it; `submit`/`try_submit`, fence-based `sync`, triple-buffered frame pacing (`MAX_FRAMES_IN_FLIGHT = 3`), blocking `reload_shader`, stats via `Arc<SharedRenderStats>` atomics.
- `render_command.rs` — ~80 self-contained `Send` command variants: GL state, uniforms (by location and by name via per-shader cache), texture binds, FBO push/pop, mesh draws (plain/instanced/immediate), resource creation (`CreateShader/CreateTexture2D/CreateMesh` + `*ByResource` variants), UBOs, `Resize`, `SwapBuffers`, `Fence`, `Shutdown`.
- `render_thread.rs` — blocking `recv()` loop with one big match executor; owns `HashMap<ResourceId, GpuResource>`, uniform-location caches, redundant-bind skipping, FBO stack.
- `window/window_gl_context.rs` — `extract_gl_context()` / `restore_gl_context()` move the GL context (and surface — macOS requires main-thread surface creation) between threads; `winit_window.redraw()` skips `swap_buffers` while extracted.
- `renderer_queue.rs` — the Lua FFI surface (`#[luajit_ffi_gen::luajit_ffi] impl Renderer`), reachable as `Engine:renderer()`.
- `camera_render_data.rs`, `entity_render_data.rs`, `render_batch.rs` — batching layer (in progress on this branch).

Activation: `cargo run -- --render-thread ...` (default off) → `MainLoop::new_events` → `Engine::start_renderer()`. `Engine` holds `renderer: Option<Renderer>`.

## Guidelines

- Every `RenderCommand` must stay `Send` and self-contained (no GL calls on the main thread while the context is extracted).
- New GL-visible functionality must be added to both paths (or as a command + `*ByResource` variant) until the transition completes.
- Changes to `renderer_queue.rs` regenerate `Renderer.lua` in `ffi_gen`/`meta` — commit those (see `luajit-ffi-bindings` skill).
- Test with `cargo run -- RenderingTest` (and with/without `--render-thread`).
