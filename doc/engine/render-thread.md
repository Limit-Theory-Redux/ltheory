# Render Thread

All OpenGL work happens through `Renderer`/`RenderCommand`
(`render/thread/`): every GL-touching type (`Mesh`, `Shader`, `Tex2D`, `Draw`,
`RenderState`, ...) takes an explicit `&mut Renderer` parameter and encodes
its work as a `RenderCommand` rather than calling `gl::*` directly. `Engine`
owns a single `Renderer` (`engine.renderer: Renderer`) created
at startup and reachable from Lua via `Engine:renderer()`.

## Two Backends, One API

`render/thread/mod.rs` selects one of two implementations by `#[cfg]`; both
expose an identical public/FFI surface, so nothing above `Renderer` needs to
know which is active:

| Backend | File | Selected by | Behavior |
|---|---|---|---|
| Threaded (default) | `renderer_threaded.rs` | no feature flag | Spawns a dedicated `"RenderThread"` OS thread, moves the `WindowGlContext` into it, communicates over bounded `crossbeam` channels |
| Immediate | `renderer_immediate.rs` | `immediate` cargo feature | Executes every command inline on the calling thread, no channels, no second thread |

The immediate backend exists for debugging/comparison (e.g. ruling out
threading as the cause of a rendering bug); production builds use the
threaded backend.

Both backends drive the same `CommandExecutor` (`command_executor.rs`,
GL implementation in `command_executor_gl.rs`), which owns all GL state:
`resources: HashMap<ResourceId, GpuResource>`, per-program uniform-location
caches, cached texture bindings, the FBO stack, UBO handles. `RenderThread`
(`render_thread.rs`) is only the plumbing around it for the threaded
backend — it pulls `RenderCommand`s off a channel, hands them to the
executor, and forwards whatever the executor replies (`CommandReply`) back
over the matching channel.

```
Main thread (game + Lua)                      Render thread (owns GL context)
────────────────────────                      ───────────────────────────────
Renderer (renderer_threaded.rs)
  submit(RenderCommand) ──► bounded crossbeam channel ──► RenderThread.run()
  fence_rx ◄──────────────  Fence replies             ◄──  execute(cmd) → gl::*
  pacing_fence_rx ◄────────  PacingFence replies       ◄── (CommandExecutor)
  shader_result_rx ◄───────  ReloadShader results
  stats_rx ◄───────────────  per-frame RenderStats snapshot
  context_rx ◄─────────────  GL context returned on shutdown
```

## RenderCommand

`render_command.rs` defines the command enum sent across the channel (or
executed inline, in immediate mode): viewport/scissor/blend/cull/depth
state, uniform sets (by GL location or, for the batch path, by name/generic
name), texture binds and updates, framebuffer push/pop, mesh draws (plain,
instanced, `DrawInstancedWithData`, `DrawImmediate`), resource lifecycle
(`CreateShader`/`CreateTexture2D`/`CreateMesh`/`DestroyResources` and their
`*ByResource` bind/draw counterparts), UBO updates, `Resize`,
`SetPresentMode`, `SwapBuffers`, `Fence`, `PacingFence`, `Shutdown`.

`GpuHandle(u32)` identifies a raw GL object for the handful of call sites
that still bind by raw handle; `ResourceId(u64)` identifies a
render-thread-managed resource (see below) and is what current code (batch
rendering, `Mesh`/`Shader` lazily-created resources) uses.

## Resources: `ResourceId` / `ResourceHandle`

`Renderer::create_resource()` is the *only* way to mint a `ResourceId` — it
returns a `ResourceHandle` (`resource_handle.rs`) pairing the id with a
sender into the renderer's destroy queue. `ResourceHandle` is deliberately
not `Clone`: exactly one handle exists per id, and its `Drop` impl enqueues
a destroy. Types that need a resource to outlive a single call (`Mesh`,
`Shader`, `Tex2D`, ...) hold the handle inside their shared `Rf<...Shared>`
cell so it drops exactly once, when the last clone does.

A destructor can't reach `&mut Renderer` to submit `DestroyResources`
directly, so drops only enqueue; `Renderer::end_frame_triple_buffered()`
drains the queue once per frame and submits one `DestroyResources { ids }`
batch.

## Frame Pacing & Fences

Two independent fence channels prevent cross-talk between blocking sync and
per-frame pacing:

- `fence_rx` — used only by `sync_intern()` (`Renderer::sync()`): submits
  `RenderCommand::Fence { fence_id }`, blocks until that exact id comes back,
  discards any other id it happens to receive.
- `pacing_fence_rx` — used only by `end_frame_triple_buffered()`: submits
  `SwapBuffers` + `PacingFence`, tracks `frames_in_flight`, and blocks only
  when `frames_in_flight >= MAX_FRAMES_IN_FLIGHT` (currently `3`).

`Renderer::submit()` is non-blocking (`try_send`) on the fast path and only
falls back to a blocking `send` when the channel is full, timing the stall
into `send_blocked_us`; `channel_high_water` tracks peak channel occupancy
per frame. `try_submit()` is the non-blocking-only variant that drops the
command if the channel is full instead of stalling.

## Activation & Context Handoff

`Engine::new` extracts the GL context from the window
(`WinitWindow::extract_gl_context`) and calls `Renderer::start(context)`
once, unconditionally — `WinitWindow` no longer performs any GL operations
itself after this point (its own `swap_buffers` is a no-op); frame end is
driven by `Renderer::end_frame_triple_buffered()`, called from
`MainLoop::about_to_wait`.

`WindowGlContext`/`WindowActiveGlContext` (`window/window_gl_context.rs`)
carry the not-yet-current GL context and surface across the thread boundary;
`make_current`/`release_for_main_thread` mediate context ownership at
startup/shutdown (macOS cannot cleanly release the context back to the main
thread — see that file's `release_for_main_thread`).

Because the handoff is unconditional, anything that needs the context
current - not just draws, but calls like `set_swap_interval` - has to go
through a `RenderCommand` rather than a direct `WinitWindow` call. That's
why runtime vsync changes (`Window:setPresentMode`) are routed as
`RenderCommand::SetPresentMode` instead of being applied in
`Engine::changed_window` directly.

## Stats

`Renderer` publishes a `RenderStats` snapshot once per frame over `stats_rx`
(`renderer_stats.rs`); `Renderer::get_stats()`/related getters expose it to
Lua. With the `stats-server` cargo feature and `--stats-server <port>`, a
live HTTP dashboard is also available — see `doc/STAT_SERVER.md`.

## Quick Reference

| Concern | File |
|---|---|
| Main-thread handle (threaded backend): submit, fences, frame pacing | `render/thread/renderer_threaded.rs` |
| Main-thread handle (immediate backend): same API, inline execution | `render/thread/renderer_immediate.rs` |
| Lua-facing FFI (`Renderer` class) | `render/thread/renderer_ffi.rs` |
| Command enum | `render/thread/render_command.rs` |
| Threaded-mode plumbing (channel loop) | `render/thread/render_thread.rs` |
| Command execution + GPU resource table | `render/thread/command_executor.rs`, `command_executor_gl.rs` |
| Resource id/handle | `render/thread/resource_handle.rs` |
| Renderer configuration (channel sizes) | `render/thread/config.rs` |
| Per-frame stats | `render/thread/renderer_stats.rs` |
| Stats HTTP dashboard (`stats-server` feature) | `render/thread/stats_server.rs`, `stats_snapshot.rs` |
| GL context handoff | `window/window_gl_context.rs` |
| Batch rendering + GPU instancing (built on this layer) | `doc/engine/batch-rendering.md` |
| Shader compilation, hot-reload, UBOs | `doc/engine/shader-system.md` |
