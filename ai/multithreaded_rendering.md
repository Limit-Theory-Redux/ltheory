# Multithreaded Rendering — State Analysis & Completion Plan

Branch: `feat/multithreaded_rendering`
Scope: `engine/lib/phx/src/render/thread/`, `engine/lib/phx/src/engine/`, `engine/lib/phx/src/window/`, Lua bindings in `engine/lib/phx/script/`.
Upstream source: `ltheory-redux`, branch `feat/multithreaded_rendering` (referred to below as **the fork**).
Status: **compiles and is clippy-clean** as of `adcc19d1 "Make it compile"` (2026-07-15); the feature itself is still dormant at runtime (§1.2).

---

## 0. Provenance — this branch is a partial port of the fork

The multithreaded rendering work in this repo is a **port + refactor of a complete,
working implementation** in the `ltheory-redux` fork. The fork's feature branch has
four commits on top of the shared history (`0ea724c0`):

| Commit | Content |
|---|---|
| `b2ee4ea2` | Multithreaded rendering with command-buffer architecture (render thread, commands, dual-mode interception, worker pool, ResourceId bridge) |
| `92adf595` | Post-processing, deferred shading shaders, shader hot-reload, docs |
| `b05b4a24` | Extract `RenderContext`, fix critical bugs (RenderQueue handle storage, frame-ring spin timeout, draw-call double-counting, uniform-cache clear on reload) |
| `0b8740ba` | Remove unused `frame_ring.rs` |

The port in this repo was made *after* the fork's final commit (it includes the
fork's `is_draw_call`/`DrawInstancedWithData` fix and the uniform-cache-clear fix)
and deliberately restructured the design:

| Fork (`engine/lib/phx/src/render/`) | This repo (`…/render/thread/`) |
|---|---|
| `render_thread.rs` (2228 lines, incl. `RenderThreadHandle`) | split: `render_thread.rs` (executor) + `renderer.rs` (`Renderer` handle) |
| `render_command.rs` | `render_command.rs` (near-identical) |
| `render_queue.rs` (global FFI singleton `RenderQueue` + `InstanceBatch`) | `renderer_queue.rs` (FFI on the `Engine`-owned `Renderer`) |
| `render_context.rs` (`RenderContext`: thread handle + worker pool + start/stop) | **not ported** (logic re-inlined into `Renderer::start/stop` + `Engine`) |
| `render_mode.rs` (global `COMMAND_MODE`, `RENDER_HANDLE`, `is_command_mode()`, `submit_command()`) | **not ported** |
| `render_worker.rs` (`WorkerPoolHandle`, `PrepareResult`, `CameraRenderData`, `EntityRenderData`, worker pool) | **partially ported** as `camera_render_data.rs` + `entity_render_data.rs` (data types only; no pool) |
| `render_batch.rs` (`RenderBatch`, `RenderBatchApi`) | `render_batch.rs`, trimmed to the data types in `adcc19d1` (the fork-only worker/FFI code was deleted, not ported) |
| dual-mode interception in `draw.rs`, `mesh.rs`, `shader.rs`, `tex2d.rs`, `render_target.rs`, `clip_rect.rs`, `render_state.rs`, `primitive_builder.rs` (~160 sites) | **not ported** — these files are still direct-GL-only here |
| `ResourceId` fields + lazy `Create*` submission in `Tex2D`/`Shader`/`Mesh` | **not ported** |
| Lua runtime control: `Engine:startRenderThread()/stopRenderThread()/isRenderThreadActive()` | replaced by `ltr --render-thread` CLI flag at boot |
| Lua integration: `RenderCoreSystem` mode switch, `Cache.lua` hot-reload, `RenderOverlay.lua`, `ShaderErrorOverlay.lua` | **not ported** |
| Docs: `doc/engine/multithreaded-rendering.md`, `doc/engine/shader-system.md`, `doc/script/rendering.md` | **not ported** |

Two design changes were intentional in the port and are worth keeping in mind:

1. **Ownership**: fork uses global statics (`render_mode.rs`) so that `Tex2D`/`Mesh`/
   `Shader`/`Draw` — which have no reference to `Engine` — can route GL calls;
   this repo instead owns the handle as `Engine.renderer: Option<Renderer>`.
   Both are needed: member ownership for lifecycle, a global mirror for interception
   (see §5.1).
2. **Activation**: fork starts/stops the render thread from Lua at runtime
   (toggleable with the `R` key in its test states); this repo moved it to a boot
   flag — and introduced a regression that makes the flag unusable (§2.2).

**Consequence for the roadmap:** most of §3 is not design work — it is porting the
fork's existing, tested code into this repo's structure.

---

## 1. Current state of this repo

### 1.1 Architecture as ported

The command-buffer render-thread core made it across intact:

```
Main thread (game + Lua)                      Render thread (owns GL context)
────────────────────────                      ───────────────────────────────
Renderer (renderer.rs)
  submit(RenderCommand) ──► bounded crossbeam channel ──► RenderThread.run()
  fence_rx ◄──────────────  fence echoes  ◄──────────────   execute(cmd) → gl::*
  shader_result_rx ◄──────  hot-reload results
  context_rx ◄────────────  GL context returned on shutdown
```

- **`Renderer`** (`render/thread/renderer.rs`) — main-thread handle (fork:
  `RenderThreadHandle`). Spawns the `"RenderThread"` OS thread, moves the
  `WindowGlContext` into it. Provides `submit`/`try_submit`, fence-based `sync`,
  triple-buffered frame pacing (`end_frame_triple_buffered`,
  `MAX_FRAMES_IN_FLIGHT = 3`), blocking `reload_shader`, and stat getters backed by
  `Arc<SharedRenderStats>` atomics.
- **`RenderCommand`** (`render_command.rs`, ~80 variants, near-identical to the
  fork) — state, uniforms **by location and by name** (`Arc<str>`, resolved via a
  per-shader uniform cache on the render thread), texture binds, FBO push/pop, mesh
  draws (plain, instanced, `DrawInstancedWithData`, `DrawImmediate`), **resource
  creation** (`CreateShader`/`CreateTexture2D`/`CreateMesh` + `*ByResource`
  variants), camera/material/light UBOs, `Resize`, `SwapBuffers`, `Flush`, `Fence`,
  `Shutdown`.
- **`RenderThread`** (`render_thread.rs`, ~1900 lines) — blocking `recv()` loop,
  one big `match` executor with `resources: HashMap<ResourceId, GpuResource>`,
  hot-reloaded shader map, uniform-location caches, cached texture bindings, FBO
  stack, UBO handles. Complete; no `todo!()`.
- **Context handoff** (`window/window_gl_context.rs`, `winit_window.rs`) —
  `extract_gl_context()` / `restore_gl_context()`; `winit_window.redraw()` silently
  skips `swap_buffers` while the context is extracted.
- **Activation** — `ltr --render-thread` CLI flag, default off
  (`engine/bin/ltr/src/main.rs:34`) → `MainLoop::new_events` (`main_loop.rs:29`) →
  `Engine::start_renderer()` (`engine.rs:131`).
- **Lua FFI** — `renderer_queue.rs` exposes the API as methods on the `Renderer`
  class (reachable via `Engine:renderer()`); regenerated bindings in
  `engine/lib/phx/script/ffi_gen|meta/*.lua` are committed.

### 1.2 What is actually live at runtime

**Nothing.** The system is dormant:

- The flag defaults to off, and (bug §2.2) turning it on exits the app.
- No runtime Lua script references the `Renderer` class; `RenderCoreSystem.lua`
  still renders through the legacy immediate path.
- The render thread's resource registry is never populated because the fork's
  interception + ResourceId layer was not ported (§2.6).

In the fork, by contrast, the feature is **runnable and toggleable at runtime**:
its `RenderingTest.lua:187-199`, `PlanetTest.lua:881-886`, and `DeferredTest.lua`
toggle the render thread with the `R` key, `RenderCoreSystem.lua:322` switches
between `renderBatched` and `renderDirect` based on `Engine:isRenderThreadActive()`,
and all resource creation transparently routes through commands.

### 1.3 The batch API (committed in `adcc19d1`)

The branch adds an entity-batch layer, originally copied from the fork and since
trimmed to fit the port's member-based design (`adcc19d1 "Make it compile"`):

- `render_batch.rs` — `RenderBatch` (entity accumulator + camera) and `BatchStats`.
  The fork-design code that depended on unported modules (`flush(worker_pool)`,
  `apply_result`, `process_serial`, the `RenderBatchApi` FFI wrapper) was deleted
  rather than ported; it lives on in the fork's `render_batch.rs` if the worker
  path is ever revived.
- `camera_render_data.rs` / `entity_render_data.rs` — extracted from the fork's
  `render_worker.rs` (data types only).
- `renderer.rs` holds `active_batch: Option<RenderBatch>` and a **port-only**
  `process_batch()` (sort → frustum-cull → dedupe shader binds → emit commands)
  that operates on the active batch in place and writes culling counters into
  `batch.stats`; `renderer_queue.rs` exposes FFI
  `begin_batch`/`add_entity`/`flush_batch`. Still unfinished: MVP uniforms don't
  reach the shader (§2.3) and flush semantics need defining (§2.7 caveats).

Note: even in the fork the batch/worker path is **dormant at runtime** — nothing in
its `script/` feeds `RenderBatch` or the worker pool; `renderBatched` groups
materials in Lua and calls `mesh:draw()` per entity. The pool also has no
intra-batch parallelism (each flush sends the whole entity list to one worker,
`render_worker.rs:177,262-329`). Treat this layer as an optimization to finish
*after* parity, not a prerequisite.

---

## 2. Defects (verified; classified against the fork)

Legend: **[regression]** introduced by the port, fork is correct · **[upstream]**
present in the fork too · **[gap]** works in the fork, missing here ·
**[port-only]** in new code that has no fork counterpart.

### 2.1 The crate does not compile (26 errors) — ✅ FIXED in `adcc19d1`

Was: `render_batch.rs` (copied from the fork) referenced symbols whose home
modules (`render_mode.rs`, `render_worker.rs`) were never ported, plus a stats
field on the wrong struct in `renderer.rs`.

Fixed exactly along recommendation (a): `render_batch.rs` was stripped to
`RenderBatch` + `BatchStats` + `new` + `add_entity` (−258 lines of dead
fork-design code), `CullStats` was deleted, and `process_batch` now records stats
into `batch.stats`. `cargo check -p phx` and `cargo clippy -p phx` are clean.

Leftovers to pick up later (folded into Phase 1/5):

- `render_batch.rs:33` — `next_entity_id: AtomicU64` survives; it's pointless
  behind `&mut self` and `entity_id` is never consumed. Remove both.
- `renderer.rs` — `render_stats: RenderStats` is kept but `#[allow(dead_code)]`;
  either wire it up or remove it.
- `render_thread.rs` — `GpuResource` is now `#[allow(dead_code)]`, which is the
  compiler literally flagging §2.6: nothing populates the resource registry.
  The annotation should come off when the resource bridge is ported.

### 2.2 `--render-thread` exits the app on successful startup — [regression]

`Engine::start_renderer` (`engine.rs:131-153`) returns **`false` on the success
path** and `true` when the renderer was already started (also: typo "olready").
`main_loop.rs:29`:

```rust
if self.render_thread && !engine.start_renderer() {
    event_loop.exit();
}
```

The fork gets this right: `RenderContext::start` (`render_context.rs:66-94`)
returns `true` on success / `false` if already running, and its Lua callers rely on
that (`if Engine:startRenderThread() then …`). Restore the fork's semantics.

### 2.3 MVP uniform is never actually set in the batch path — [port-only]

`RenderBatch::add_entity` hardcodes `mvp_location = -1` ("Will use name-based
uniforms"), but the port-only `Renderer::process_batch` (`renderer.rs:401`) emits
location-based `SetUniformMat4 { location: -1, .. }`. GL ignores location -1 →
nothing visible.

The fork's equivalent serial path (`render_batch.rs::process_serial`) does it
correctly: `SetUniformMat4ByName` with interned thread-local `Arc<str>` names
(`UNIFORM_MVP`, `UNIFORM_MODEL`), resolved through the render thread's per-shader
uniform cache — which also survives shader hot-reload, where locations don't.
Port that approach into `process_batch` and drop
`mvp_location`/`model_location` from `EntityRenderData`.

### 2.4 Frustum-plane normalization is mathematically wrong — [upstream]

`camera_render_data.rs:39-46` normalizes plane vectors with `Vec4::normalize()`
(4D norm including `w`). Plane normalization for a sphere-distance test must divide
by `length(plane.xyz)` only; otherwise `distance < -radius` compares a wrongly
scaled distance against the radius.

```rust
fn normalize_plane(p: Vec4) -> Vec4 {
    p / p.truncate().length()
}
```

This was copied verbatim from the fork (`render_worker.rs:79-84`), where it is
equally wrong but currently unreachable at runtime (the worker path is dormant).
Its unit tests pass only because they use spheres far from the planes. **Fix here
and upstream the fix to the fork.**

### 2.5 Fence channel cross-talk between `sync` and frame pacing — [upstream]

Two consumers drain the single `fence_rx`:

- `sync_intern` (`renderer.rs:195-211`) waits for *its* fence ID and **discards**
  any other IDs (`Ok(_) => continue`) — those are frame fences, so
  `frames_in_flight` is never decremented for them and frame pacing drifts toward
  permanent max-in-flight blocking.
- `end_frame_triple_buffered` (`renderer.rs:231-243`) drains/blocks on the same
  channel and can consume a sync fence, after which a concurrent/later `sync()`
  blocks forever.

Identical bug in the fork (`render_thread.rs:261-315`), and reachable there because
`RenderQueue:Sync` is exposed to Lua. (The shader-reload result path uses its own
channel in both repos and is clean.)

**Fix:** either two channels (tag the fence command with a kind), or shared
bookkeeping so every received ID updates the in-flight count for frame fences no
matter which call site received it. **Fix here and upstream.**

### 2.6 GPU resource creation is unbridged — [gap], solved in the fork

The render thread fully implements `CreateShader`/`CreateTexture2D`/`CreateMesh`/
`DestroyResource` and the `*ByResource` variants, but in this repo **nothing ever
submits them** — `tex2d.rs`, `mesh.rs`, `shader.rs` still call `gl::*` directly on
the main thread, which has no current GL context once the render thread starts.

The fork solved this with a **ResourceId bridge + dual-mode interception**, and its
code is the template to port:

- `Tex2D` has `resource_id: Option<ResourceId>` (fork `tex2d.rs:22`); creation in
  command mode submits `CreateTexture2D { id, … }` and stores the id
  (`tex2d.rs:143-151,177-191,255-281`); updates go via
  `UpdateTexture2DDataByResource`; `ensure_resource_id()` (`tex2d.rs:527`) lazily
  creates for pre-existing textures.
- `Shader` has `resource_id` (fork `shader.rs:39`); lazily submits `CreateShader`
  on first bind (`shader.rs:865-876`), binds via
  `BindShaderByResource { id, shader_key }` (`shader.rs:892`) — `shader_key` lets
  the render thread substitute a hot-reloaded program. Texture binds inside shaders
  use `BindTexture2DByResource` (`shader.rs:396-531`).
- `Mesh` has `resource_id` (fork `mesh.rs:28`); submits `CreateMesh` lazily and
  re-creates when its version counter changes (`mesh.rs:391-430`); draws via
  `DrawMeshByResource` (`mesh.rs:542-557`).
- Immediate mode: `PrimitiveBuilder` emits `DrawImmediate { vertices }`
  (fork `primitive_builder.rs:241-273`); `Draw`, `RenderState`, `ClipRect`,
  `RenderTarget`, `Viewport` all branch on `is_command_mode()` → `submit_command()`
  (~160 sites total).

Known limitations in the fork to carry over/document: texture read-back,
`screen_capture`, and `deep_clone` are unsupported in command mode
(fork `tex2d.rs:82,335,441`), and a texture bound without a `resource_id` or cached
CPU data logs a warning and renders wrong (fork `shader.rs:405,468,533`).

### 2.7 Cull stats are computed and dropped — ✅ FIXED in `adcc19d1`, with caveats

`process_batch` now writes `total_entities`/`entities_visible`/`entities_culled`
directly into `batch.stats` instead of a dropped local. Two caveats remain:

1. **Flush no longer consumes the batch.** `flush_batch` used to `take()`
   `active_batch`; now `process_batch` operates in place and never clears
   `batch.entities`. Calling `flush_batch` twice re-submits every entity, and
   `add_entity` after a flush accumulates on top — correctness silently relies on
   `begin_batch` being called every frame. Define the semantics: clear `entities`
   at the end of `process_batch` (keeping the allocation, see §4), or make a
   second flush a no-op/error.
2. **Stats are not reachable from Lua.** `RenderBatch::get_stats()` exists but has
   no FFI exposure; the fork shows the equivalent numbers in its
   `RenderOverlay.lua`. Expose them (e.g. via `SharedRenderStats` or getters on
   `Renderer`) when the overlay is ported.

---

## 3. Roadmap to finish — port from the fork, don't re-design

Ordering principle: restore compilation, fix the shared bugs once (here + upstream),
then port the fork's layers in the order that gets the feature end-to-end runnable.

### Phase 0 — Restore compilation — ✅ DONE (`adcc19d1`, 2026-07-15)
`render_batch.rs` stripped to the data types, stats moved into `batch.stats`,
check + clippy clean. Small leftovers moved into Phase 1 (items 4-6).

### Phase 1 — Correctness fixes (small, high value; upstream the shared ones)
1. Fix `Engine::start_renderer` return semantics to match the fork's
   `RenderContext::start` (§2.2) + the "olready" typo.
2. Fix frustum-plane normalization (§2.4). Add the missing unit test: sphere
   straddling a plane by less than its radius must be kept. **Also send to fork.**
3. Port the fork's name-based-uniform approach into `process_batch` (§2.3).
4. Define flush semantics: clear `batch.entities` at the end of `process_batch`
   so a double `flush_batch` can't re-submit everything (§2.7 caveat 1).
5. Remove `next_entity_id: AtomicU64` + the unused `entity_id`; wire up or drop
   the dead `render_stats` field (§2.1 leftovers).
6. Expose `BatchStats` to Lua for the perf overlay (§2.7 caveat 2).
7. Fix fence cross-talk (§2.5). **Also send to fork.**

### Phase 2 — Port the dual-mode interception + ResourceId layer (the core work)
This was §"design a resource bridge" before the fork was known; it is now a port:

1. Port `render_mode.rs` (global `COMMAND_MODE` + `RENDER_HANDLE` mirror +
   `is_command_mode()`/`submit_command()`/`try_submit_command()`/
   `submit_commands()`/`next_resource_id()`). Keep `Engine.renderer` as the owner;
   have `Renderer::start/stop` set/clear the global mirror exactly as the fork's
   `RenderContext::start/stop` does (`render_context.rs:77,113-116`). The mirror is
   required because `Tex2D`/`Mesh`/`Shader`/`Draw` have no `Engine` reference.
2. Port the `ResourceId` fields + lazy `Create*` submission + `*ByResource`
   bind/draw into `tex2d.rs`, `shader.rs`, `mesh.rs`, using the fork's diffs as the
   template (§2.6 has the exact locations). Adapt module paths
   (`render::thread::…`) and this repo's ECS-era changes to these files —
   expect real merge work in `shader.rs`/`tex2d.rs`, which diverged on this branch
   (shader hot-reload landed here separately).
3. Port the interception sites in `draw.rs`, `render_state.rs`, `clip_rect.rs`,
   `render_target.rs`, `viewport.rs`, `primitive_builder.rs`.
4. Carry over the fork's command-mode limitation warnings (read-back etc.) and its
   `next_resource_id` global counter (replaces the per-`Renderer` counter, which
   would reset across runtime restarts of the thread and collide with resources
   that survived in `Tex2D` objects).

### Phase 3 — Runtime control + Lua integration (port from fork)
1. Port `render_context.rs` (or fold its logic cleanly into `Renderer`): worker
   pool optional, `Arc<Renderer>` handle, ordered stop (disable command mode →
   clear global → shutdown → restore context).
2. Restore the fork's runtime FFI: `Engine:startRenderThread()/stopRenderThread()/
   isRenderThreadActive()` + stats getters, so the thread can be toggled at runtime
   (see §5.3 — recommended over the CLI-flag-only approach; keep the flag as "start
   enabled at boot").
3. Wire frame pacing: call `renderer.end_frame_triple_buffered()` from
   `MainLoop::about_to_wait` when active, as the fork does (`main_loop.rs:331`
   there), and route `WindowEvent::Resized` to `try_submit(Resize)`.
4. Port the Lua side: `RenderCoreSystem.lua` active-mode switch
   (fork line 322: `renderBatched` vs `renderDirect`), `Cache.lua` hot-reload path
   (`Engine:reloadShaderOnRenderThread` + resource-id invalidation),
   `RenderOverlay.lua` (perf overlay, Shift+O) and `ShaderErrorOverlay.lua`, and
   the R-key toggle in the test states. Adapt to this repo's newer
   `RenderCoreSystem` (it diverged substantially on this branch).
5. End-to-end check: toggle the render thread at runtime in `RenderingTest` /
   `PlanetTest` with identical output in both modes.

### Phase 4 — Performance (after parity)
1. **Batch the channel**: change the payload to `Vec<RenderCommand>` /
   `CommandBuffer` with buffer recycling (existing TODO at `renderer.rs:184`;
   the fork has `submit_commands` but still per-command sends — improvement applies
   to both).
2. Revive the batch/worker path only if profiling justifies it — note it is dormant
   even in the fork, and the fork's pool has **no intra-batch parallelism** (whole
   batch → one worker, `render_worker.rs:177`). If revived, shard entities across
   workers and prefer this repo's existing `task_queue` subsystem
   (`engine/lib/phx/src/engine/task_queue/`) over maintaining a second pool.
3. Extend render-thread state caching (texture binds + uniform locations exist) to
   shader-program binds; keep per-frame shader dedupe across batches.
4. Revisit `MAX_FRAMES_IN_FLIGHT = 3` (§5.2).

### Phase 5 — Cleanup & landing
1. Rename for clarity: `renderer.rs` (thread management) vs `renderer_queue.rs`
   (FFI API) are inverted relative to their names.
2. Fix `RenederThreadError` → `RenderThreadError`; resolve the commented-out
   `unsafe impl Send for WindowGlContext` (`window_gl_context.rs:21`) deliberately.
3. Remove `Option<Renderer>`/transition TODO once render-thread mode is default.
4. Keep regenerated Lua bindings committed in the same change as the Rust API
   that produces them (current bindings are committed; maintain this going
   forward as Phases 2-3 grow the FFI surface).
5. Port the fork's docs (`doc/engine/multithreaded-rendering.md`,
   `doc/engine/shader-system.md`, `doc/script/rendering.md`) and update them: the
   fork's multithreading doc still references the deleted `frame_ring.rs` and
   documents the dormant worker pool as if active — fix while porting.
6. Add a CI-runnable smoke test that exercises command mode.

---

## 4. Improvement suggestions (beyond the port)

- **Consolidate the two global handles** (fork carry-over): the fork keeps a handle
  in `render_mode::RENDER_HANDLE` *and* another inside the `RenderQueue` singleton
  (`render_queue.rs:31`), plus a thread-local `COMMAND_BUFFER` fallback. When
  porting Phase 2/3, keep exactly one submission path (`render_mode`'s) and make
  the Lua-facing queue forward to it.
- **`submit()` blocking visibility**: `submit` silently blocks when the bounded
  channel is full; only `end_frame_triple_buffered` records wait time. Accumulate
  wait time in `submit` too.
- **Error surfacing**: render-thread GL errors are only logged. A error counter in
  `SharedRenderStats` (or drained error channel) would make failures observable
  from Lua — the fork's `ShaderError` queue + overlay already does this for shader
  compiles; generalize it.
- **`RenderBatch` allocation reuse**: `begin_batch` still allocates a fresh
  `RenderBatch` (1024-entity `Vec`) every frame. Since `adcc19d1` the batch already
  persists in `Renderer` between `begin` and `flush`; go the rest of the way — keep
  one `RenderBatch` for the `Renderer`'s lifetime, have `begin_batch` reset camera +
  `clear()` entities/stats. This also resolves the flush-clearing question (§2.7).
- **`add_entity` FFI granularity**: one FFI call per entity per frame is expensive
  from LuaJIT at scale (likely why the fork's `renderBatched` stayed Lua-side).
  Long-term, batch construction belongs in Rust, fed from ECS storage.
- **Sort key**: define the `u32` layout (pass ≪ shader ≪ material ≪ depth) in one
  place; add back-to-front depth ordering for transparents.
- **`begin_frame` vs `flush` contract**: `begin_frame_intern` silently clears
  unflushed commands; flush automatically or log when non-empty.

---

## 5. Design decisions

### 5.1 How does the game reach the render thread? — **decided by the fork: port it**
The fork implements Rust-side dual-mode interception (`is_command_mode()` →
`submit_command()` in every GL-touching type), keeping all Lua scripts unchanged
and both modes runtime-switchable. This was previously an open question here; the
answer now is to port that layer (Phase 2), keeping `Engine.renderer` ownership
plus the `render_mode` global mirror — the interception sites have no `Engine`
reference, which is exactly why the fork used globals.

### 5.2 Resource story — **decided by the fork: ResourceId bridge, port it**
Implemented and working upstream (`resource_id: Option<ResourceId>` in
`Tex2D`/`Shader`/`Mesh`, lazy `Create*`, `*ByResource` bind/draw). The
shared-context alternative is moot. Remaining sub-decision: how to support
command-mode texture read-back (`screen_capture`, `getData`) — the fork punts with
warnings; a blocking `ReadTexture2DData { id, reply_tx }` round-trip command is the
likely completion.

### 5.3 Runtime toggle (fork) vs boot flag (this repo) — open, recommend the fork's
The fork can start/stop the render thread mid-session from Lua (context extraction
and restore both work), which is invaluable for A/B testing, perf overlays, and
falling back on driver issues. The boot flag is a strict subset. Recommendation:
port the runtime FFI (Phase 3.2) and keep `--render-thread` only as "enable at
startup".

### 5.4 Frames in flight — open
3 maximizes throughput but costs latency; 2 is the common sweet spot for a vsync'd
GL swap. Make it configurable in `RenderThreadConfig`, default 2, measure once
end-to-end rendering works.

---

## 6. Quick reference — where things are

| Concern | This repo | Fork (`ltheory-redux`) |
|---|---|---|
| Main-thread handle, fences, frame pacing | `render/thread/renderer.rs` | `render/render_thread.rs` (`RenderThreadHandle`, L193-315) |
| Lua-facing FFI | `render/thread/renderer_queue.rs` (`Renderer` class) | `render/render_queue.rs` (`RenderQueue` global, `InstanceBatch`) |
| Command enum | `render/thread/render_command.rs` | `render/render_command.rs` |
| GL executor thread | `render/thread/render_thread.rs` | `render/render_thread.rs` |
| Dual-mode switch + global handle | — (not ported) | `render/render_mode.rs` |
| Lifecycle encapsulation | inlined in `Renderer`/`Engine` | `render/render_context.rs` |
| Worker pool / cull data | `camera_render_data.rs`, `entity_render_data.rs` (types only) | `render/render_worker.rs` |
| Batch collector | `render/thread/render_batch.rs` (trimmed to data types) | `render/render_batch.rs` |
| GL interception + ResourceId in resources | — (not ported) | `render/{tex2d,mesh,shader,draw,render_state,clip_rect,render_target,primitive_builder}.rs` |
| Start/stop + regression bug | `engine/engine.rs:131-174` | `engine/engine.rs:136-158` + `render_context.rs:66-154` |
| Activation | `engine/bin/ltr/src/main.rs:34` (CLI flag), `engine/main_loop.rs:29` | Lua `Engine:startRenderThread()`, R-key in tests |
| Frame-end wiring | — (never called) | fork `engine/main_loop.rs:331` |
| GL context handoff/restore | `window/window_gl_context.rs`, `winit_window.rs:297-360` | `window/winit_window.rs` (`RenderThreadGlData`) |
| Lua integration | — (not ported) | `script/Modules/Rendering/Systems/RenderCoreSystem.lua:322`, `script/Render/Cache.lua:86-110`, `script/Shared/Tools/RenderOverlay.lua`, `ShaderErrorOverlay.lua` |
| Docs | this file | `doc/engine/multithreaded-rendering.md`, `doc/engine/shader-system.md`, `doc/script/rendering.md` |
| Existing worker infra to reuse | `engine/lib/phx/src/engine/task_queue/` | — |
