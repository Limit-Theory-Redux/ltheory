# Multithreaded Rendering — State Analysis & Completion Plan

Branch: `feat/multithreaded_rendering`
Scope: `engine/lib/phx/src/render/thread/`, `engine/lib/phx/src/engine/`, `engine/lib/phx/src/window/`, Lua bindings in `engine/lib/phx/script/`.

---

## 1. Current state

### 1.1 Architecture as built

The feature implements a classic command-buffer render thread:

```
Main thread (game + Lua)                      Render thread (owns GL context)
────────────────────────                      ───────────────────────────────
Renderer (renderer.rs)
  submit(RenderCommand) ──► bounded crossbeam channel ──► RenderThread.run()
  fence_rx ◄──────────────  fence echoes  ◄──────────────   execute(cmd) → gl::*
  shader_result_rx ◄──────  hot-reload results
  context_rx ◄────────────  GL context returned on shutdown
```

- **`Renderer`** (`render/thread/renderer.rs`) — main-thread handle. Spawns the
  `"RenderThread"` OS thread, moves the `WindowGlContext` into it, and makes it
  current there. Provides `submit`/`try_submit`, fence-based `sync`, triple-buffered
  frame pacing (`end_frame_triple_buffered`, `MAX_FRAMES_IN_FLIGHT = 3`), blocking
  `reload_shader`, and stat getters backed by `Arc<SharedRenderStats>` atomics.
- **`RenderCommand`** (`render_command.rs`, ~80 variants) — self-contained, `Send`
  commands: viewport/scissor/blend/cull/depth state, uniforms **by location and by
  name** (`Arc<str>`, resolved via a per-shader uniform cache on the render thread),
  texture binds, FBO push/pop, mesh bind/draw (plain, instanced,
  `DrawInstancedWithData`, immediate mode), **resource creation**
  (`CreateShader`/`CreateTexture2D`/`CreateMesh` + `*ByResource` bind/draw variants),
  camera/material/light UBOs, `Resize`, `SwapBuffers`, `Flush`, `Fence`, `Shutdown`.
- **`RenderThread`** (`render_thread.rs`, ~1900 lines) — blocking `recv()` loop,
  one big `match` executor. Maintains `resources: HashMap<ResourceId, GpuResource>`,
  hot-reloaded shader map, per-shader uniform-location caches, cached texture
  bindings (skip-redundant-bind), an FBO stack with depth guard, and the UBO
  handles. No `todo!()`/`unimplemented!()` — the executor itself is complete.
- **Context handoff** (`window/window_gl_context.rs`, `winit_window.rs`) —
  `extract_gl_context()` makes the context not-current on the main thread and moves
  it (context + surface, because macOS requires main-thread surface creation) into
  the render thread; on shutdown the context is sent back over a channel and
  `restore_gl_context()` re-establishes direct GL mode. `winit_window.redraw()`
  silently skips `swap_buffers` while the context is extracted.
- **Activation** — `ltr --render-thread` CLI flag, **default off**
  (`engine/bin/ltr/src/main.rs:34`) → `MainLoop::new_events` (`main_loop.rs:29`) →
  `Engine::start_renderer()` (`engine.rs:131`). Engine holds
  `renderer: Option<Renderer>` with a `// TODO: remove Option after transition period`.
- **Lua FFI** — `renderer_queue.rs` exposes the whole API
  (`#[luajit_ffi_gen::luajit_ffi] impl Renderer`): frame management, batch API,
  state, uniforms, textures, framebuffers, draws, UBOs. Reachable from Lua via
  `Engine:renderer()`. Bindings are regenerated in the uncommitted
  `engine/lib/phx/script/ffi_gen|meta/*.lua` changes.

### 1.2 What is actually live at runtime

**Nothing.** The system is dormant:

- The flag defaults to off, and (bug #2 below) turning it on exits the app.
- No runtime Lua script references the `Renderer` class.
  `script/Modules/Rendering/Systems/RenderCoreSystem.lua` and everything else still
  render through the legacy immediate path (`Window:beginDraw/endDraw`,
  `RenderState`, `Draw`, `Tex2D`, `Mesh`, `Shader`) — direct `gl::*` calls on the
  main thread.
- The render thread's resource registry is never populated because no code submits
  `Create*` commands (see bug #6).

### 1.3 Uncommitted work in progress (the batch API)

The working tree adds an entity-batch layer intended to move culling + command
generation off the Lua side:

- `render_batch.rs` (new) — `RenderBatch` (entity accumulator + `CameraRenderData`),
  `BatchStats`, plus a **dead second design**: `flush(worker_pool)`,
  `apply_result(PrepareResult)`, `process_serial()`, and a commented-out FFI wrapper
  `RenderBatchApi` with a 35-scalar-argument `set_camera`. These reference symbols
  that do not exist anywhere in the crate (`WorkerPoolHandle`, `PrepareResult`,
  `is_command_mode`, `submit_command`, `RENDER_BATCH`, `UNIFORM_MVP/MODEL`).
- `camera_render_data.rs` (new) — view/proj/view-proj + frustum-plane extraction and
  `sphere_in_frustum` test.
- `entity_render_data.rs` (new) — per-entity payload (transform, bounding sphere,
  VAO, index count, shader handle, uniform locations, sort key).
- `renderer.rs` — adds `active_batch`, `CullStats`, and `process_batch()` (sort by
  key → frustum-cull → dedupe shader binds → emit `SetUniformMat4` + `DrawMesh`
  into `command_buffer`).
- `renderer_queue.rs` — FFI `begin_batch` / `add_entity` / `flush_batch`.

This is the *new* member-based design; the thread-local + worker-pool code in
`render_batch.rs` is a leftover from an earlier iteration and is what breaks the
build.

---

## 2. Blocking defects (verified)

### 2.1 The crate does not compile (26 errors)

`cargo check -p phx` fails; all errors are in the uncommitted files.

- `render_batch.rs:87,111,117,119,148,151,159–177` — `WorkerPoolHandle`,
  `PrepareResult`, `is_command_mode()`, `submit_command()`, `UNIFORM_MVP`,
  `UNIFORM_MODEL` do not exist; `RenderCommand` is not imported.
- `render_batch.rs:195` — `impl Default` calls `Self::new()` with 0 of 5 args.
- `render_batch.rs:210,259,301,318,325,330,334,338,342` — `RENDER_BATCH`
  thread-local does not exist (whole `RenderBatchApi` is dead).
- `renderer.rs:422` — `self.render_stats.batches_processed`: no such field on
  `RenderStats` (it lives on `BatchStats`).

**Fix:** delete the dead half of `render_batch.rs` (keep `RenderBatch`,
`BatchStats`, `new`, `add_entity`; drop `flush`/`apply_result`/`process_serial`/
`RenderBatchApi`), and track batch stats in a proper field (e.g. give `Renderer`
a `batch_stats: BatchStats` or add the counter to `RenderStats`).

### 2.2 `--render-thread` exits the app on successful startup

`Engine::start_renderer` (`engine.rs:131-153`) returns **`false` on the success
path** and `true` when the renderer was already started (also: typo "olready").
`main_loop.rs:29`:

```rust
if self.render_thread && !engine.start_renderer() {
    event_loop.exit();
}
```

So enabling the feature immediately exits the event loop. Invert the returns
(`true` on success / already-started, `false` on failure).

### 2.3 MVP uniform is never actually set in the batch path

`RenderBatch::add_entity` hardcodes `mvp_location = -1` and `model_location = -1`
("Will use name-based uniforms"), but `Renderer::process_batch`
(`renderer.rs:401`) unconditionally emits location-based
`SetUniformMat4 { location: -1, .. }`. GL silently ignores location -1 → geometry
draws with a stale/identity MVP, i.e. nothing visible.

**Fix (pick one):**
- Use the existing name-based path: emit `SetUniformMat4ByName` with interned
  `Arc<str>` constants (e.g. `static UNIFORM_MVP: LazyLock<Arc<str>>`), which the
  render thread already resolves through its per-shader uniform cache; or
- Extend the FFI `add_entity` to accept real uniform locations from Lua (the shader
  is introspectable on the script side today).

The name-based route is more robust because the render thread's program may be a
hot-reloaded replacement with different locations — that is exactly why the
`*ByName` variants exist (`render_command.rs:180-183`).

### 2.4 Frustum-plane normalization is mathematically wrong

`camera_render_data.rs:39-46` normalizes plane vectors with `Vec4::normalize()`,
which divides by the **4D** norm (including `w`). Plane normalization for a
sphere-distance test must divide by `length(plane.xyz)` only; otherwise
`distance < -radius` compares a wrongly scaled distance against the radius, causing
both false culls and false accepts (worse for large near/far offsets where `w`
dominates).

```rust
fn normalize_plane(p: Vec4) -> Vec4 {
    p / p.truncate().length()
}
```

### 2.5 Fence channel cross-talk between `sync` and frame pacing

Two consumers drain the single `fence_rx`:

- `sync_intern` (`renderer.rs:195-211`) waits for *its* fence ID and **discards**
  any other IDs (`Ok(_) => continue`) — those are frame fences, so
  `frames_in_flight` is never decremented for them and frame pacing drifts toward
  permanent max-in-flight blocking.
- `end_frame_triple_buffered` (`renderer.rs:231-243`) drains/blocks on the same
  channel and can consume a sync fence, after which a concurrent/later `sync()`
  blocks forever.

**Fix:** either use two channels (frame fences vs. sync fences — the fence command
could carry a `FenceKind`), or keep one channel but route every received ID through
shared bookkeeping (e.g. a small `HashSet` of signaled IDs + in-flight counter
updated for *frame* IDs regardless of which call site received them).

### 2.6 GPU resource creation is unbridged (the core missing piece)

The render thread fully implements `CreateShader` / `CreateTexture2D` /
`CreateMesh` / `DestroyResource` and the `*ByResource` bind/draw variants
(`render_thread.rs:1439,1527,1564`), keeping a `HashMap<ResourceId, GpuResource>`.
**Nothing in the codebase ever submits those commands.** Meanwhile `tex2d.rs`
(~65 direct GL calls), `mesh.rs` (~29), and `shader.rs` still create GL objects
directly on the main thread — which has **no current GL context** once
`extract_gl_context()` runs. In render-thread mode, every legacy `Tex2D.Create`,
mesh upload, or shader compile is undefined behavior / GL errors.

This is the real remaining work of the feature; see the roadmap (Phase 2) and the
open design decision in §5.

### 2.7 Cull stats are computed and dropped

`process_batch` fills a local `CullStats` (`renderer.rs:365-386`) but never stores
it into `self.cull_stats`, so the new stats getters/HUD will always read zeros.
Assign it at the end of the function (and expose it via FFI if the Lua HUD should
show culling numbers, as `RenderCoreSystem` does for other stats).

---

## 3. Roadmap to finish the feature

### Phase 0 — Restore compilation (small)
1. Gut `render_batch.rs` to: `BatchStats`, `RenderBatch { entities, camera, stats }`,
   `new()`, `add_entity()`. Delete `flush`, `apply_result`, `process_serial`,
   `RenderBatchApi`, the `Default` impl, and the `next_entity_id` atomic
   (`entity_id` is never consumed; if an ID is ever needed, a plain `u64` counter
   behind `&mut self` suffices).
2. Fix the `batches_processed` stat (`renderer.rs:422`) — move it to `BatchStats`
   held by `Renderer`, or add the field to `RenderStats`.
3. `cargo check -p phx` and `cargo clippy -p phx` clean.

### Phase 1 — Correctness fixes (small, high value)
1. Invert `Engine::start_renderer` returns (bug 2.2) + fix the "olready" typo.
2. Fix frustum-plane normalization (bug 2.4). Add a unit test: unit sphere at
   origin with a known view-proj → inside; sphere far outside a side plane → culled;
   sphere straddling a plane by less than its radius → kept.
3. Switch `process_batch` MVP/model uniforms to `SetUniformMat4ByName` with interned
   names (bug 2.3); drop `mvp_location`/`model_location` from `EntityRenderData`.
4. Store `CullStats` (bug 2.7).
5. Fix fence cross-talk (bug 2.5).

### Phase 2 — Resource-creation bridge (the core work)
Goal: GPU objects can be created while the render thread owns the context.

1. **Shaders first** (smallest surface, plumbing already half-exists via
   `ReloadShader`/`hot_reloaded_shaders`): when `Engine.renderer` is active,
   `Shader` creation submits `CreateShader { id, vertex_src, fragment_src }` and
   stores a `ResourceId`; binding uses `BindShaderByResource`. Uniform setting goes
   through the `*ByName` commands (locations are meaningless across threads).
2. **Textures**: `Tex2D::create/setData/setMagFilter/...` submit
   `CreateTexture2D`/`UpdateTexture2DDataByResource`/`SetTexture2D*` when the
   renderer is active. Note the readback problem: `Tex2D_GetData`-style APIs need a
   blocking round-trip command (add `ReadTexture2DData { id, reply_tx }`) or must be
   documented as unsupported in render-thread mode.
3. **Meshes**: `Mesh` upload submits `CreateMesh { id, vertices, indices, format }`;
   draws use `DrawMeshByResource`/`DrawMeshInstancedByResource`.
4. Introduce a single handle abstraction inside the existing types, e.g.
   `enum GpuObject { Direct(u32), Deferred(ResourceId) }`, so `Tex2D`/`Mesh`/`Shader`
   keep their public API and pick the path at creation time based on
   `Engine.renderer`. Free the deferred variant by submitting `DestroyResource` in
   `Drop`.
5. Resource IDs: `Renderer::next_resource_id` already exists — make it the single
   source (it currently starts at 1 per-`Renderer`, which is fine while there is
   exactly one).

### Phase 3 — Frame-path integration
1. Decide the integration strategy (see §5.1). Recommended: **Rust-side
   interception** — keep Lua scripts on `Draw`/`RenderState`/`ClipRect` and make
   those Rust entry points submit commands when `Engine.renderer.is_some()`; the
   command set already covers state, clears, immediate-mode geometry
   (`DrawImmediate`), and FBO push/pop (`PushFramebuffer`/`PopFramebuffer` exist
   precisely for `RenderTarget::push/pop`).
2. Wire the frame boundary: in render-thread mode `MainLoop::about_to_wait` should
   call `renderer.end_frame_triple_buffered()` instead of relying on
   `winit_window.redraw()` (which currently no-ops the swap because the context is
   extracted). `Window:endDraw()` on the Lua side must not double-swap.
3. Route `WindowEvent::Resized` to `renderer.resize(w, h)` (the `Resize` command and
   surface-resize handling already exist on the render thread; `try_submit` was
   built for exactly this).
4. End-to-end smoke test: `ltr --render-thread` running
   `script/States/App/Tests/RenderingTest.lua` (or `CameraTest.lua`) renders
   identically to the direct mode.

### Phase 4 — Performance
1. **Batch the channel** (existing TODO at `renderer.rs:184`): change the channel
   payload to `Vec<RenderCommand>` (or a `CommandBuffer` struct), have
   `flush_intern` send one message per frame/segment, and recycle buffers through a
   return channel to avoid per-frame allocation. Per-command `send()` on a bounded
   channel is the dominant overhead of the current design.
2. Only then consider parallel batch preparation (cull + sort + command generation
   per camera/pass) using the **existing** `task_queue` worker subsystem
   (`engine/lib/phx/src/engine/task_queue/`) — do not resurrect the abandoned
   `WorkerPoolHandle` design.
3. Extend render-thread state caching (texture binds and uniform locations are
   already cached) to shader-program binds and blend/depth state.
4. Revisit `MAX_FRAMES_IN_FLIGHT = 3`: with a vsync'd `SwapBuffers` this adds up to
   two frames of latency; 2 is usually the better default. Make it part of
   `RenderThreadConfig`.

### Phase 5 — Cleanup & landing
1. Rename for clarity: `renderer.rs` (thread management) vs `renderer_queue.rs`
   (FFI API) are inverted relative to their names — e.g. `renderer.rs` →
   `render_handle.rs` / keep FFI in `renderer_api.rs`.
2. Fix `RenederThreadError` → `RenderThreadError`.
3. Resolve the commented-out `unsafe impl Send for WindowGlContext`
   (`window_gl_context.rs:21`) deliberately: document why the glutin types are safe
   to move (they are moved once into the thread, never shared), or restructure so
   no manual `unsafe impl` is needed.
4. Remove `Option<Renderer>` once render-thread mode is the default (existing TODO,
   `engine.rs:30`).
5. Commit the regenerated Lua bindings (`ffi_gen`/`meta` for `Renderer`, `CubeFace`,
   `DataFormat`, `Metric`, `PixelFormat`, `TexFormat`) together with the Rust API
   changes so generated code never drifts from the macro output.
6. Add a short `ai/` or module-level doc describing the command-flow architecture
   (this file can seed it) and a CI job that builds with the feature exercised.

---

## 4. Improvement suggestions (beyond bug fixes)

- **Unify `CullStats` and `BatchStats`** — same numbers, two structs
  (`renderer.rs:30` vs `render_batch.rs:9`). Keep one, expose it through
  `SharedRenderStats` so the Lua HUD can display culling like it displays draw
  calls.
- **`process_batch` shader dedupe is per-batch only**; track the last-bound shader
  across the whole frame in `Renderer` (reset in `begin_frame_intern`) to skip
  redundant `BindShader` between consecutive batches.
- **`submit()` blocking semantics**: `submit` silently blocks when the bounded
  channel is full. That is fine for backpressure but should be measured — the
  `main_thread_wait_us` stat only covers `end_frame_triple_buffered`. Consider
  accumulating wait time in `submit` too, so stalls are visible.
- **Error surfacing**: the render thread logs GL errors but the main thread never
  learns about them. A lightweight error counter in `SharedRenderStats` (or a
  drained error channel) would make failures observable from Lua.
- **`RenderBatch::entities` capacity**: `Vec::with_capacity(1024)` is re-allocated
  every `begin_batch`. Keep the `RenderBatch` inside `Renderer` and `clear()` it
  per frame to reuse the allocation (the `active_batch: Option<RenderBatch>`
  take/replace pattern already fights the borrow checker; a persistent field with a
  `bool`/state enum is simpler and allocation-free).
- **`add_entity` FFI granularity**: one FFI call per entity per frame is expensive
  from LuaJIT at scale. Once the Rust-side ECS render systems exist (the branch's
  direction), batch construction should happen in Rust from ECS storage; treat the
  Lua `addEntity` path as a bring-up/debug tool.
- **Sort key**: currently a bare `u32` supplied by the caller. Define its layout
  (e.g. pass ≪ shader ≪ material ≪ depth) in one place so batching quality is
  predictable, and compute depth-based ordering for transparent objects
  (back-to-front) as a second key.
- **`begin_frame` vs `flush` contract** is implicit (`begin_frame_intern` clears
  whatever wasn't flushed). Either flush automatically in `begin_frame` or log when
  non-empty, so silently dropped commands can't hide bugs.

---

## 5. Open design decisions

### 5.1 How does the game reach the render thread?
- **Option A — port Lua to the `Renderer` API**: `RenderCoreSystem.lua` calls
  `beginBatch`/`addEntity`/`flushBatch`/UBO methods directly. Pros: explicit,
  no dual-path Rust code. Cons: enormous Lua surface to port (`Draw`, `ClipRect`,
  post-fx, HmGui all bypass it), and per-entity FFI overhead.
- **Option B — Rust-side interception (recommended)**: keep the Lua API unchanged;
  `Draw`/`RenderState`/`Tex2D`/`Mesh`/`Shader`/`RenderTarget` internally submit
  commands when `Engine.renderer` is active. Pros: every existing script and test
  works in both modes, migration is incremental per module, and the command enum was
  clearly designed for this (`DrawImmediate`, `PushFramebuffer`, `*ByName`
  uniforms). Cons: a mode check inside each render-facing type until direct mode is
  retired.

### 5.2 Resource story: command bridge vs. shared contexts
The `ResourceId` command bridge (Phase 2) matches the code already written. The
alternative — a second GL context shared with the render thread's, kept current on
the main thread for resource creation only — would let `tex2d.rs`/`mesh.rs` stay
untouched, but shared-context object visibility across threads is a notorious
driver-bug minefield and still requires sync for completeness guarantees.
Recommendation: stay with the command bridge; the `*ByResource` machinery is
already implemented and tested code paths exist on the render thread.

### 5.3 Frames in flight
3 maximizes throughput but costs latency; 2 is the common sweet spot for a vsync'd
GL swap. Make it configurable in `RenderThreadConfig` and default to 2 once
end-to-end rendering works and can be measured.

---

## 6. Quick reference — where things are

| Concern | Location |
|---|---|
| Main-thread handle, fences, frame pacing | `engine/lib/phx/src/render/thread/renderer.rs` |
| Lua-facing FFI (`#[luajit_ffi]`) | `engine/lib/phx/src/render/thread/renderer_queue.rs` |
| Command enum | `engine/lib/phx/src/render/thread/render_command.rs` |
| GL executor thread | `engine/lib/phx/src/render/thread/render_thread.rs` |
| Batch/cull WIP (broken, uncommitted) | `render_batch.rs`, `camera_render_data.rs`, `entity_render_data.rs` |
| Start/stop + inverted-return bug | `engine/lib/phx/src/engine/engine.rs:131-174` |
| CLI flag (default off) | `engine/bin/ltr/src/main.rs:34` |
| Flag → start call | `engine/lib/phx/src/engine/main_loop.rs:29` |
| GL context handoff/restore | `engine/lib/phx/src/window/window_gl_context.rs`, `winit_window.rs:297-360` |
| Legacy direct-GL resource paths | `engine/lib/phx/src/render/{tex2d,mesh,shader}.rs` |
| Existing worker pool to reuse | `engine/lib/phx/src/engine/task_queue/` |
