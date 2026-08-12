# Batch rendering: finish the port

## Process

Implementation proceeds **phase by phase** (Phases 1-6 below), gated on review
at each end:

1. Before starting a phase: post a short summary of what that phase will do
   and wait for approval.
2. After implementing a phase: post a summary of what was actually changed
   and wait for approval before moving to the next phase.
3. After each approved phase, generate a one-line git commit message for it
   (commit is only created if/when the user separately asks — see standing
   "no autocommit" preference).

## Context

`feat/multithreaded_rendering` has already landed the render-thread core, shader
hot-reload, and camera/point-light UBOs (`91401ab0`, `618bdd4d`, `59e69d8a`). The
renderer architecture has moved well past the state described in
`ai/multithreaded_rendering.md`, which is now stale in an important way: there is
no more dual-mode GL interception (`render_mode.rs`/`is_command_mode()` from the
fork was never needed) — every GL-touching type (`Mesh`, `Shader`, `Draw`, ...)
now takes an explicit `&mut Renderer`, and `Renderer` itself has two compile-time
backends (`renderer_threaded.rs` default, `renderer_immediate.rs` under the
`immediate` cargo feature) sharing one public/FFI surface. GPU resources are
tracked by `ResourceId`/`ResourceHandle` (`render/thread/resource_handle.rs`),
not raw GL handles.

The one piece that was ported but never adapted to this newer design is **batch
rendering**: `render_batch.rs`, `entity_render_data.rs`, `camera_render_data.rs`,
`batch_stats.rs`, and `Renderer::process_batch`/`renderer_ffi.rs`'s
`begin_batch`/`add_entity`/`flush_batch`. This layer is a dead stub:

- `EntityRenderData::mesh_vao: u32` / `shader_handle: u32` are raw GL handles —
  but GL objects live only on the executor thread now, and Lua/game code can no
  longer obtain them (Mesh/Shader only expose `ResourceId`s via `ResourceHandle`).
  `begin_batch`/`add_entity` (`renderer_ffi.rs:42-80`) are literally uncallable
  with real data today.
- `process_batch_intern` (`renderer_shared.rs:40-105`) emits
  `RenderCommand::SetUniformMat4 { location: entity.mvp_location, .. }` with
  `mvp_location` hardcoded to `-1` in `RenderBatch::add_entity` — GL silently
  ignores location `-1`, so no uniform is ever actually set.
  It also computes an MVP matrix and looks for `model_location`, both dead
  ideas: this repo's shaders get `mView`/`mProj` from `CameraUBO`
  (`res/shader/include/camera_ubo.glsl`) and take only `mWorld`/`mWorldIT` as
  per-draw uniforms (see `res/shader/vertex/wvp.glsl`,
  `script/Shared/Definitions/ShaderVarFuncs.lua:5-15`). The fork predates the
  UBO migration, so its `UNIFORM_MVP`/`UNIFORM_MODEL` design must **not** be
  ported as-is.
- `RenderCommand::SetUniform*ByName`/`BindShaderByResource`/`DrawMeshByResource`
  and their executor implementations (`command_executor.rs:391-427`,
  `command_executor_gl.rs`) are complete and already used by `Shader`/`Mesh`
  elsewhere — the batch layer just never adopted them.
- `RenderCommand::DrawInstancedWithData` (true GPU instancing) and its executor
  (`command_executor_gl.rs:1229-1300`, persistent instance VBO with attribute
  divisors, `command_executor.rs:135-138`) are fully implemented and completely
  unused — no producer anywhere.
- `RenderCoreSystem.lua:renderInOrder` (`:263-285`) issues one shader
  start/stop and one `mesh:draw()` per entity per mesh, with no batching, no
  grouping, and no frustum culling at the Lua layer at all.

Decisions made with the user for this pass:
- **Rebuild the batch layer on `ResourceId`, fix uniforms, and add GPU
  instancing** (not just Lua-side material grouping) — the instancing command
  path already exists and is otherwise dead code.
- **Culling/sorting stays synchronous** on the calling thread (as
  `process_batch_intern` already does) — do not port the fork's worker pool;
  it gave no real parallelism there either (one task per flush, blocking
  recv). Revisit only if profiling demands it.
- All non-batch gaps found while comparing against the fork (missing `Draw`
  primitives, `RenderOverlay.lua`, postfx shaders, point-light ECS components,
  docs) are **out of scope** — tracked separately, not part of this plan.

---

## Plan

### Phase 1 — Rebuild the batch data types on `ResourceId`

**`engine/lib/phx/src/render/thread/entity_render_data.rs`**
Replace `mesh_vao: u32`, `shader_handle: u32`, `mvp_location: i32`,
`model_location: i32` with `mesh_id: ResourceId`, `shader_id: ResourceId`.
Keep `transform: Mat4`, `bounds_center: Vec3`, `bounds_radius: f32`,
`sort_key: u32`, `entity_id: u64` (still unused; carry over, don't remove —
out of scope).

**`engine/lib/phx/src/render/thread/render_batch.rs`**
`RenderBatch::add_entity` signature changes to accept `mesh_id: ResourceId`,
`shader_id: ResourceId` instead of `mesh_vao: u32`, `shader_handle: u32`; drop
the `mvp_location`/`model_location` parameters entirely.

**`engine/lib/phx/src/render/thread/renderer_shared.rs`** (`process_batch_intern`)
Rewrite the per-entity command emission to match this repo's actual uniform
contract instead of the fork's MVP-by-name scheme:
- `BindShaderByResource { id: entity.shader_id, shader_key: None }` on shader
  change (mirrors `Shader::start`, `shader.rs:509`, which also passes `None` —
  match its contract, don't invent a different one).
- A `SetUniformMat4ByWellKnownName { name: WellKnownUniformName, value }`
  command for `mWorld`/`mWorldIT` — **not** the `Arc<str>`-keyed
  `SetUniformMat4ByName` and **not** a thread-local cache (see revision below).
- `DrawMeshByResource { id: entity.mesh_id, index_count, primitive: Triangles }`.
- Keep frustum culling and `sort_key` sorting exactly as-is — `CameraRenderData`
  and `sphere_in_frustum` are already correct (the plane-normalization fix at
  `camera_render_data.rs:72-75` already landed).

Every `RenderCommand::SetUniform*ByName`/`BindShaderByResource`/`DrawMeshByResource`
variant used here already has a working executor implementation
(`command_executor.rs:391-427`) — this phase only needs to *produce* commands
that already work, not add new engine plumbing.

**Revision (applied):** the first pass used `SetUniformMat4ByName { name: Arc<str>, .. }`
with a `thread_local!` cache of two pre-built `Arc<str>` names so each entity
could clone the `Arc` instead of allocating a fresh one. This reintroduced a
`thread_local!` right after the codebase finished removing all of them from
the render path (`renderer_data.rs`'s `RendererData` fields are documented as
`was thread_local!` for exactly this reason). Replaced with:

- `WellKnownUniformName` — a small `Copy` enum (`render_command.rs`, next to
  `ResourceId`) with variants `MWorld`/`MWorldIT` and `as_str(self) -> &'static str`.
  Being `Copy`, a value costs nothing to construct or send across the
  render-thread channel — no allocation, no cache, no thread-local.
- `RenderCommand::SetUniformMat4ByWellKnownName { name: WellKnownUniformName, value: [f32; 16] }` —
  a fully `Copy` command payload.
- Executor (`command_executor_gl.rs`): `cmd_set_uniform_mat4_by_well_known_name`
  resolves the location through the existing per-program
  `HashMap<Arc<str>, i32>` cache (`uniform_caches`), but the lookup itself
  takes `&str` (`Arc<str>: Borrow<str>`) and only allocates an `Arc<str>` on a
  cache *miss*, to use as the insert key. This mirrors the split
  `shader.rs::resolve_uniform_location` (`shader.rs:606-620`) already uses for
  its own cache. `get_uniform_location_cached`/`get_uniform_location_for_program`
  were changed from taking `name: Arc<str>` to `name: &str`; the 9 existing
  (still producer-less) `cmd_set_uniform_*_by_name` methods were updated to
  pass `&name` — mechanical, no behavior change for them.
- `renderer_shared.rs`: the `thread_local!` block was deleted; `process_batch_intern`
  emits `SetUniformMat4ByWellKnownName { name: WellKnownUniformName::MWorld, .. }`
  / `::MWorldIT` directly.
- `SetUniformMat4ByName`/`Arc<str>` and its 8 Int/Float siblings were left in
  place as pre-existing scaffolding (zero producers before and after this
  work — out of scope to remove).

### Phase 2 — Give Lua a way to obtain `ResourceId`s

Batch entities need a `ResourceId` for their mesh and shader, which nothing
currently exposes outside `render/`:

**`engine/lib/phx/src/render/mesh.rs`**
Extract the resource-creation half of `draw_bind` (`:362-390`) into a new
internal helper `fn ensure_resource(&mut self, r: &mut Renderer) -> ResourceId`
that both `draw_bind` and a new public FFI method call. Add:
```rust
pub fn resource_id(&mut self, r: &mut Renderer) -> ResourceId {
    self.ensure_resource(r)
}
```
so Lua/batch code can lazily create (or reuse) the mesh's GPU resource and get
its id back, exactly like `draw_bind` does today but without also drawing.

**`engine/lib/phx/src/render/shader.rs`**
`ShaderShared::handle: ResourceHandle` is already created eagerly in
`Shader::new`/`from_preprocessed`, so this is just a getter:
```rust
pub fn resource_id(&self) -> ResourceId {
    self.shared.as_ref().handle.id()
}
```

Both are new `#[luajit_ffi_gen::luajit_ffi]` methods, so `resourceId(r)` /
`resourceId()` become callable from Lua and regenerate the corresponding
`ffi_gen`/`meta` bindings.

### Phase 3 — Rewire the `Renderer` batch FFI

**`engine/lib/phx/src/render/thread/renderer_ffi.rs`**
`add_entity` takes `mesh_id: u64, shader_id: u64` (raw `ResourceId` scalar,
same FFI convention as `bind_shader(handle: u32)` etc. elsewhere in this file)
plus the existing transform/bounds/sort_key params, and forwards to
`RenderBatch::add_entity` with `ResourceId(mesh_id)`/`ResourceId(shader_id)`.
`begin_batch`/`flush_batch`/`get_batch_stats` are unchanged (they don't touch
the removed fields).

### Phase 4 — Wire `RenderCoreSystem.lua` to the batch API

Replace the per-entity direct-draw loop in `renderInOrder`
(`script/Modules/Rendering/Systems/RenderCoreSystem.lua:263-285`) with a batched
path for the common case (entities with `getMeshes()`, no custom `renderFn`,
which still needs to fall back to direct calls):

1. Before the pass loops (near `cacheData()`, `:169`), call
   `Renderer:beginBatch(view, projection, eyeX, eyeY, eyeZ)` once per frame
   (camera data is already computed by `CameraManager`).
2. In `renderInOrder`, for entities with meshes: apply the existing
   `applyCachedVars(mat, entity)` (material/instance uniform caching stays —
   it covers *material* uniforms like textures/colors, which the batch API
   does not touch, only `mWorld`/`mWorldIT`/draw), then call
   `Renderer:addEntity(transform, cx, cy, cz, radius, mesh:resourceId(), shader:resourceId(), sortKey)`
   instead of `sh:start()` / `mesh:draw()` / `sh:stop()`.
   - **World-space bounds**: use the existing `mesh:getCenter()` /
     `mesh:getRadius()` FFI (`mesh.rs:433-454`, already implemented) combined
     with the entity's `TransformComponent` (`pos`, `scale` — scale is a single
     uniform scalar per `TransformComponent.lua:17`, so
     `worldRadius = mesh:getRadius() * transform:getScale()` needs no matrix
     decomposition). No new engine code needed for this.
   - **Sort key**: define a simple `u32` packing — shader `ResourceId` in the
     high bits, mesh `ResourceId` (or a running counter) in the low bits — so
     entities naturally group by shader first (minimizes `BindShaderByResource`
     calls in `process_batch_intern`), matching what
     `applyCachedVars`'s material-level cache already assumes about grouping.
     Keep this in one helper function, not inlined at each call site (existing
     ask from `ai/multithreaded_rendering.md`'s improvement list, still valid).
   - Entities with a custom `renderFn` keep calling it directly, unbatched
     (matches current behavior — `renderFn` may issue arbitrary GL work).
3. After the entity loop for a pass, call `Renderer:flushBatch()` before
   `passes[...]:stop()`.
4. `deferredLighting()`'s full-screen `Draw.Rect` calls and the UI/post chain
   are unaffected — only world-geometry draws route through the batch.

This keeps `applyCachedVars`'s per-material/per-instance uniform caching (it's
solving a different problem — arbitrary material uniforms, not just
transform/shader — and there's no reason to touch it) while removing the
per-entity shader start/stop and adding real frustum culling, which does not
exist in Lua today.

### Phase 5 — GPU instancing (`InstanceBatch`)

Add a small new Rust type mirroring the fork's `InstanceBatch`
(`ltheory-redux` `render/render_queue.rs:552-652`) but built on this repo's
`ResourceId`/`Renderer` model instead of the fork's global `submit_command()`:

**New file `engine/lib/phx/src/render/thread/instance_batch.rs`**
```rust
pub struct InstanceBatch {
    mesh_id: ResourceId,
    index_count: i32,
    instances: Vec<InstanceData>,
    primitive: CmdPrimitiveType,
}
```
- `create(mesh: &mut Mesh, r: &mut Renderer, primitive: CmdPrimitiveType) -> InstanceBatch`
  — calls `mesh.resource_id(r)` (Phase 2) and `mesh.get_index_count()`.
- `add_instance(&mut self, transform: &Matrix, r: f32, g: f32, b: f32, a: f32)`
  — pushes `InstanceData::from_transform_color(..)` (already exists,
  `render_command.rs:78-87`).
- `draw(&mut self, r: &mut Renderer)` — submits one
  `RenderCommand::DrawInstancedWithData { mesh_id, index_count, instances: self.instances.clone(), primitive }`.
  Needs a new `Renderer::draw_instanced_with_data_intern` producer method in
  both `renderer_threaded.rs` and `renderer_immediate.rs` (mirroring
  `draw_mesh_instanced_intern`'s shape) — the executor side needs no changes.
- `clear(&mut self)`, `flush(&mut self, r: &mut Renderer)` (= draw + clear),
  `instance_count(&self) -> usize`.
- `#[luajit_ffi_gen::luajit_ffi] impl InstanceBatch` for all of the above so
  Lua can build instance batches (e.g. for asteroid fields / debris —
  `script/Modules/CelestialObjects/Systems/AsteroidBeltRenderer.lua` currently
  hand-loops `lodMesh:draw(distSq)` per rock and is the natural first caller,
  though wiring it up is a follow-up, not required by this plan).

**GLSL** — port from the fork, adapted to this repo's UBO-based vertex
pipeline (the fork's versions predate the UBO migration and must be checked
against `res/shader/include/vertex.glsl`/`camera_ubo.glsl` rather than copied
verbatim):
- `res/shader/include/instanced.glsl` — per-instance attributes at locations
  4-7 (mat4 columns) and 8 (color), `VS_INSTANCED_BEGIN`/`VS_INSTANCED_END`
  macros providing `mWorld`/`mWorldIT`-equivalent values built from the
  instance matrix (replacing the fork's uniform `mWorld` with an attribute).
- `res/shader/vertex/wvp_instanced.glsl` — `wvp.glsl` (Phase 1 confirmed its
  exact contents above) with `VS_BEGIN`/instance-matrix substitution instead
  of the uniform `mWorld`/`mWorldIT`.
No material/Lua wiring beyond what's listed above — this phase delivers the
capability, not a specific visual feature to switch over.

### Phase 6 — Cleanup

- `render_batch.rs`'s `next_entity_id`/`entity_id` — still unused after this
  work (noted as a pre-existing leftover in `ai/multithreaded_rendering.md`
  §2.1); remove them now that the surrounding code is being touched anyway,
  or leave with a comment if out of scope — call this out in the PR rather
  than silently dropping/keeping.
- Update `ai/multithreaded_rendering.md` §1.3/§2.3/§2.7's batch section to
  reflect the new design (it currently describes the dead stub as the
  intended end state).

---

## Verification

1. `cargo check -p phx && cargo clippy -p phx` (both feature sets:
   default/threaded and `--features immediate`) — confirms the FFI surface
   and both `Renderer` backends still compile identically.
2. Regenerate FFI bindings (`ffi_gen`/`meta`) via the project's normal
   codegen step (see `build-and-run` skill) and commit them alongside the
   Rust changes, per existing convention in this branch.
3. Run the engine (`ltr`) with a scene that has many `RenderComponent`
   entities (e.g. an existing test state with several meshes) and confirm:
   - Geometry renders identically to before (visual diff / no missing
     objects — this catches a regressed `mWorld`/`mWorldIT` uniform or wrong
     `ResourceId`).
   - `Renderer:getBatchStats()` reports non-zero `entitiesVisible` and
     `entitiesCulled` changes when panning the camera off-scene (proves
     frustum culling is live, unlike today).
4. Add/extend a unit test in `render_batch.rs` or `renderer_shared.rs`
   asserting `process_batch_intern` emits `SetUniformMat4ByName{name:"mWorld"}`
   / `SetUniformMat4ByName{name:"mWorldIT"}` / `DrawMeshByResource` (not the
   old location-based/MVP commands) for a simple one-entity batch.
5. For instancing: a small test state or existing scene swapped to use
   `InstanceBatch` (e.g. a grid of cubes) to confirm the persistent instance
   VBO path (`command_executor_gl.rs:1229-1300`) actually executes end-to-end
   — this path has zero runtime coverage today.
