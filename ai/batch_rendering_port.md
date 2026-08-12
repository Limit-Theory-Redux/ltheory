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
- A `SetUniformMat4ByGenericName { name: GenericUniformName, value }`
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

- `GenericUniformName` — a small `Copy` enum (`render_command.rs`, next to
  `ResourceId`) with variants `MWorld`/`MWorldIT` and `as_str(self) -> &'static str`.
  Being `Copy`, a value costs nothing to construct or send across the
  render-thread channel — no allocation, no cache, no thread-local.
- `RenderCommand::SetUniformMat4ByGenericName { name: GenericUniformName, value: [f32; 16] }` —
  a fully `Copy` command payload.
- Executor (`command_executor_gl.rs`): `cmd_set_uniform_mat4_by_generic_name`
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
  emits `SetUniformMat4ByGenericName { name: GenericUniformName::MWorld, .. }`
  / `::MWorldIT` directly.
- `SetUniformMat4ByName`/`Arc<str>` and its 8 Int/Float siblings were left in
  place as pre-existing scaffolding (zero producers before and after this
  work — out of scope to remove).

### Phase 2 — Give Lua a way to obtain `ResourceId`s

Batch entities need a `ResourceId` for their mesh and shader, which nothing
currently exposes outside `render/`:

**`engine/lib/phx/src/render/mesh.rs`**
Extracted the resource-creation half of `draw_bind` into a private
`fn ensure_resource(&mut self, r: &mut Renderer) -> ResourceId` helper, placed
in `Mesh`'s plain (non-FFI) `impl` block alongside other Rust-internal helpers
like `add_plane` - the `#[luajit_ffi_gen::luajit_ffi]` block exposes *every*
method inside it to Lua regardless of `pub`/private, so a helper meant to stay
internal has to live outside it. `draw_bind` now just calls the helper; a new
public FFI method also calls it:
```rust
pub fn resource_id(&mut self, r: &mut Renderer) -> u64 {
    self.ensure_resource(r).0
}
```
so Lua/batch code can lazily create (or reuse) the mesh's GPU resource and get
its id back, exactly like `draw_bind` does today but without also drawing.

**`engine/lib/phx/src/render/shader.rs`**
`ShaderShared::handle: ResourceHandle` is already created eagerly in
`Shader::new`/`from_preprocessed`, so this is just a getter:
```rust
pub fn resource_id(&self) -> u64 {
    self.shared.as_ref().handle.id().0
}
```

**Revision (applied):** both getters return a plain `u64`, not `ResourceId`
itself. The first pass returned `ResourceId` directly from these
FFI-exposed methods; the codegen doesn't know `ResourceId` is a bare `Copy`
newtype (it has no `#[luajit_ffi_gen::luajit_ffi]` impl block of its own -
`Tex1D`/`Tex2D`/`Tex3D`/`TexCube`'s existing `resource_id()` getters, which
predate this work, are correspondingly *not* FFI-exposed, Rust-internal only)
and defaulted to boxed/managed-object semantics: it generated
`Core.ManagedObject(_instance, libphx.ResourceId_Free)` in the Lua bindings,
but `ResourceId_Free` is never declared or exported anywhere - that call
would have failed at runtime. Returning `u64` instead matches this codebase's
established convention for handles crossing the FFI boundary (e.g.
`bind_shader(&mut self, handle: u32)` in `renderer_ffi.rs`) and regenerates
as a plain `uint64` return with a direct `libphx.Mesh_ResourceId`/
`Shader_ResourceId` call - no managed object, nothing to free.

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

**What actually shipped differs from the original plan in three ways**,
each discovered mid-implementation and resolved with the user before
writing code:

1. **World transform: `RigidBodyComponent`, not `TransformComponent`.**
   This engine renders camera-relative (floating origin, for float precision
   far from world origin) — `CameraManager:updateViewMatrix()` zeroes the
   view matrix's translation, and every entity's existing `mWorld` auto var
   (`ShaderVarFuncs.mWorldFunc`) already computes
   `entity:get(RigidBodyComponent):getRigidBody():getToWorldMatrix(eye)`,
   not raw `TransformComponent` coordinates. Batched entities use the exact
   same call, so geometry lands in the same space the camera (and every
   other entity) already uses.
2. **`begin_batch`/`add_entity` take `&Matrix`, not `&[f32; 16]`.**
   Retroactive fix to Phase 3: `update_camera_ubo` already established
   `&Matrix` (a `#[repr(C)]` newtype over `glam::Mat4`,
   `typedef = "float m[16]"`) as this FFI layer's idiom for passing matrices
   — cheap, and Lua just passes a `Matrix` object directly instead of
   flattening one into a raw array + size param. `render_batch.rs`'s
   `RenderBatch::new`/`add_entity` and `renderer_ffi.rs`'s
   `begin_batch`/`add_entity` were updated together with Phase 3's other
   changes (same files, folded in before Phase 4 started).
3. **No material-group batching — flush per entity.** The plan called for
   grouping entities by `Material` object to bind/apply material uniforms
   once per group. Traced `Materials.Asteroid()`-style factories
   (`Shared/Registries/Materials.lua`'s `__call = function(_, ...) return
   template:clone() end`) and confirmed every entity gets its own `Material`
   clone — grouping by object identity never groups more than one entity in
   this codebase, so the machinery bought nothing. Separately, grouping
   turned out to be load-bearing for *correctness*, not just an optimization:
   `applyCachedVars`'s uniform-set commands submit immediately, while the
   batch's commands only reach the render thread on `Renderer:flush()` — call
   `addEntity` for several entities before flushing and every entity's
   uniforms land before any of their draws (last-instance-wins). Flushing
   after every single entity sidesteps this entirely (nothing is deferred
   across entities), at the cost of no shader-bind-count reduction versus
   today — the real, kept benefit is CPU-side frustum culling, which does
   not exist in Lua today (see `AsteroidBeltRenderer.lua`'s hand-rolled
   distance culling for the kind of thing this could eventually replace).

**Changes:**
- `script/Modules/Cameras/Managers/CameraManager.lua` — added
  `getViewMatrix()`/`getProjectionMatrix()` getters (thin wrappers over
  `activeCameraData:getView()`/`getProjection()`); nothing previously
  exposed the camera matrices themselves outside `CameraManager`.
- `script/Modules/Rendering/Systems/RenderCoreSystem.lua` — `renderInOrder`
  calls `Renderer:beginBatch(...)` once per pass (reuses the batch's backing
  storage across entities — calling it per-entity would reallocate a
  1024-capacity `Vec` every entity). Per meshed entity (unchanged for
  `renderFn` entities): `sh:start()` → `applyCachedVars(mat, entity)` →
  compute `transform = rb:getToWorldMatrix(eye)`,
  `center = transform:mulPoint(mesh:getCenter())`,
  `radius = mesh:getRadius() * rb:getScale()` → `Renderer:addEntity(...)` →
  `Renderer:flushBatch()` → `Renderer:flush()` → `sh:stop()` (replaces the
  old `mesh:draw()` call).
- `engine/lib/phx/script/ffi_ext/Mesh.lua` — added a `resourceId` override
  (`mt.__index.resourceId = function(self) return
  libphx.Mesh_ResourceId(self, Renderer) end`), matching the file's existing
  pattern for `drawBind`/`draw`/etc. (auto-inject the global `Renderer` so
  call sites read `mesh:resourceId()`, not `mesh:resourceId(Renderer)`).

**Verification performed:**
- `cargo check -p phx` (default + `immediate`), `cargo clippy -p phx
  --all-features -- -D warnings` — clean.
- `luajit -e "loadfile(...)"` on all three edited Lua files — parses clean.
- Ran `RenderingTest` (a 216-box grid with `RigidBody` physics + `DebugColor`
  material) via `cargo run -p ltr -- RenderingTest`: no crash, no panic, no
  Lua runtime error, stable 59 FPS for the run duration.
- Added temporary tracing (removed before landing) at both the Lua call site
  and the Rust executor (`cmd_bind_shader_by_resource`,
  `cmd_set_uniform_float3`, `cmd_draw_mesh_by_resource`) to confirm the full
  pipeline end-to-end: correct `ResourceId`s resolved, `BatchStats` showing
  `visible=1, culled=0` for on-screen entities (frustum culling genuinely
  runs and passes), correct GL program bound, correct uniform value
  (`(1.0, 0.0, 1.0)`, `DebugColor`'s magenta) set on the correct program,
  `glDrawElements` called with a valid VAO and the right index count.
- **Could not get a screenshot-level visual confirmation.** `RenderingTest`
  renders solid black in this environment both before and unrelated to this
  change: it's a deferred-lit scene (`deferredLighting()` composites
  `buffer0`/`buffer1` G-buffer output through global/directional/point light
  passes into the final image) and `RenderingTest.lua` sets up zero lights
  (confirmed: zero matches for `"light"` in that file) — the `<irMap>`/
  `<envMap>` "shader variable stack does not contain variable" warnings
  present from the very first log line, before any batch code runs, are the
  same symptom. A scene with no lights produces a black final composite
  regardless of what's drawn to the G-buffer, so this is a pre-existing
  property of this test scene's content, not a rendering regression — but it
  means the GL-trace verification above is the strongest evidence obtained
  this session; an actual lit test scene (e.g. one with a star/sun light)
  would be needed for a true pixel-level before/after comparison.

### Phase 5 — GPU instancing (`InstanceBatch`) — done

Everything in the original plan below shipped as designed; the notes call
out the one thing that had to be discovered rather than assumed (vertex
attribute binding) and the verification performed.

**`engine/lib/phx/src/render/thread/instance_batch.rs`** (new file)
```rust
pub struct InstanceBatch {
    mesh_id: ResourceId,
    index_count: i32,
    instances: Vec<InstanceData>,
    primitive: CmdPrimitiveType,
}
```
`#[luajit_ffi_gen::luajit_ffi] impl InstanceBatch`: `create(mesh: &mut Mesh,
r: &mut Renderer, primitive: CmdPrimitiveType) -> InstanceBatch` (calls
`mesh.resource_id(r)` + `mesh.get_index_count()`); `add_instance(&mut self,
transform: &Matrix, r: f32, g: f32, b: f32, a: f32)` (converts via
`transform.to_cols_array()` into `InstanceData::from_transform_color`,
matching the `&Matrix`-at-the-boundary convention from Phase 3's revision);
`draw(&mut self, r: &mut Renderer)` (submits one
`RenderCommand::DrawInstancedWithData`); `clear`; `flush` (draw + clear);
`instance_count(&self) -> i32`. Registered in `render/thread/mod.rs`.

**`renderer_threaded.rs` / `renderer_immediate.rs`** — added
`draw_instanced_with_data_intern`, mirroring `draw_mesh_by_resource`'s shape
in each backend (`self.submit(...)` vs `self.executor.cmd_draw_instanced_with_data(...)`).
The executor implementation itself (`command_executor_gl.rs:1241-1330` —
persistent instance VBO, attribute locations 4-7/8, correct divisors) already
existed from before this port and needed no changes.

**Discovered mid-implementation: vertex attribute *names* must be bound to
locations 4-8 in Rust, the same way locations 0-3 already are.** GLSL 330
vertex attribute locations here aren't set via `layout(location=N)` in the
shader — they're bound by name via `gl::BindAttribLocation` before linking
(`command_executor_gl.rs`'s `create_shader`, previously only binding
`vertex_position`/`vertex_normal`/`vertex_uv`/`vertex_color` to 0-3). Added
five more calls binding `instance_matrix_col0..3`/`instance_color` to 4-8,
right next to the existing ones — a no-op for shaders that don't declare
those names, so this doesn't affect anything else.

**GLSL** — not a verbatim port from the fork (its version predates the
camera-UBO migration): `res/shader/include/instanced.glsl` duplicates
`vertex.glsl`'s varyings/`logDepth` but declares `mWorld`/`mWorldIT` as
`mat4` *local variables* built from the per-instance attributes inside a new
`VS_INSTANCED_BEGIN` macro, instead of `vertex.glsl`'s `uniform mat4 mWorld;`
(a per-draw uniform can't hold a different value per instance within one
instanced draw call, hence the separate include rather than reusing
`vertex.glsl` as-is). `res/shader/vertex/wvp_instanced.glsl` mirrors
`wvp.glsl` exactly, swapping `#include vertex`/`VS_BEGIN`/`VS_END` for
`#include instanced`/`VS_INSTANCED_BEGIN`/`VS_INSTANCED_END`.

**Verification performed:**
- `cargo check -p phx` (default + `immediate`), `cargo clippy -p phx
  --all-features -- -D warnings`, `cargo test -p phx --lib render::` — all
  clean; FFI bindings regenerated (`ffi_gen`/`meta/InstanceBatch.lua`) with
  the expected surface (`InstanceBatch.Create(mesh, r, primitive)` →
  `:addInstance`/`:draw`/`:clear`/`:flush`/`:instanceCount()`).
- Live shader-compile check: temporarily added
  `Cache.Shader('wvp_instanced', 'material/solidcolor')` to
  `RenderingTest.lua:onInit` and ran it — logged a valid `Shader*` cdata
  pointer, no compile/link error. Reverted before landing (`git diff` on
  that file is empty).

No material/Lua wiring beyond the above — this phase delivers the
capability, not a specific visual feature to switch over. A natural first
caller is `script/Modules/CelestialObjects/Systems/AsteroidBeltRenderer.lua`
(currently hand-loops `lodMesh:draw(distSq)` per rock), left as a follow-up.

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
