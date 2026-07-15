# Shader System — Fork Analysis & Porting Plan

Source: `ltheory-redux` fork, branch `feat/multithreaded_rendering`, range `0ea724c0..HEAD`
(shader work lands almost entirely in `b2ee4ea2` "multithreaded rendering" and `92adf595`
"post-processing, shaders, and documentation").
Target: this repo, branch `feat/multithreaded_rendering`.
Companion doc: `ai/multithreaded_rendering.md` (the render-thread port plan; referenced as
"main roadmap" below).

---

## 1. Overview

The fork's shader changes decompose into five units with different dependencies:

| Unit | What | Depends on | Status here |
|---|---|---|---|
| **A** | Hot-reload: file watcher + error queue + error overlay | nothing (works in direct GL mode) | partial overlap — this repo has its own manual F5 reload |
| **B** | Camera/Light UBOs + GLSL migration to uniform blocks | nothing (dual-mode) — but GLSL+Lua must land atomically | Rust half exists but direct-mode part is commented out; GLSL/Lua absent |
| **C** | Dual-mode (command-mode) shader path: `ResourceId`, name-based uniforms, render-thread reload | `render_mode.rs` + interception substrate (main roadmap Phase 2) | render-thread side exists; main-thread `shader.rs` side missing |
| **D** | GLSL extras: post-fx rework, deferred helpers, instancing shaders | mostly none; instancing needs batch API | absent |
| **E** | `doc/engine/shader-system.md` | A + B | absent |

Recommended porting order: **A → B → (C with main roadmap Phase 2) → D (selective) → E**.
A and B are immediately useful in today's single-threaded rendering and don't wait for the
render thread work.

Both repos evolved the shader system independently since the common ancestor `0ea724c0`:
the fork built the units above; this repo added manual hot-reload (`731f1b0d`), the
`DynamicShaderVar`/material system, and ~13 new game shaders. **This is a reconciliation,
not a copy** — the conflict points are called out per unit.

---

## 2. Unit-by-unit analysis

### Unit A — Hot-reload system (watcher + error queue + overlay)

**What the fork built** (all paths relative to the fork):

- `engine/lib/phx/src/render/shader_watcher.rs` (new, 322 lines) — file watching via the
  **`notify` 6.1** crate (`default-features = false, features = ["macos_kqueue"]`; also adds
  `parking_lot 0.12` to `Cargo.toml`). OS-native events delivered through an `mpsc` channel;
  Lua **pulls** once per frame via `ShaderWatcher.Poll()`. A global
  `Mutex<Option<ShaderWatcherInner>>` keeps canonicalized bidirectional `file ↔ shader` maps
  and watches every `Resource::get_folders(ResourceType::Shader)` recursively.
  `register_shader` records vs/fs **plus all transitively `#include`d files**
  (`collect_shader_includes` re-parses `#include` lines), so editing an include reloads every
  dependent shader. FFI: `Init` (idempotent), `Shutdown`, `IsActive`, `Register(key, vs, fs)`,
  `Poll() -> count`, `GetChanged(i)`, `ClearChanged`. Independent of the render thread.
- `engine/lib/phx/src/render/shader_error.rs` (new, 175 lines) — global
  `Mutex<VecDeque<ShaderErrorInfo>>` capped at 10 (FIFO drop-oldest), entries
  `{shader_key, error_type, message, timestamp}`. `push_shader_error()` is called from
  main-thread compile failures **and** from the render thread's `ReloadShader` handler — the
  queue doubles as the cross-thread error channel. FFI class `ShaderError`: `GetCount`,
  `HasNewErrors`, `AcknowledgeErrors`, `GetShaderKey/GetErrorType/GetMessage/GetTimestamp(i)`,
  `Clear`, `ClearAt`, `ClearForShader(key)`, `Update` (frame counter), `GetLatestMessage`.
- `shader.rs` additions: fallible `try_from_preprocessed` (compile vs → fs → link, each
  failure pushed to the error queue with a cleaned GL info-log; fork `shader.rs:81-130`);
  `create_error_shader` — a minimal magenta shader so broken loads render pink instead of
  panicking (`shader.rs:529`); `load()` = `try_load().unwrap_or_else(create_error_shader)`;
  FFI `TryLoad`, `CreateErrorShader`, `Invalidate`.
- Lua:
  - `script/Render/ShaderHotReload.lua` (new, 265) — orchestrator. `init()` →
    `ShaderWatcher.Init` + re-register everything already in `Cache`; per-frame `update()` →
    poll changed keys → `Cache.TryReloadShader(key)` → on success reload registered materials.
    Also manual `reloadShader(vs, fs)` and error passthroughs.
  - `script/Render/Cache.lua` rework (+163) — cache key changed `vs..fs` → `vs..':'..fs`;
    new `shaderInfo` (paths per key) and `lastWorkingShaders` (keep-last-working fallback);
    `Cache.Shader` registers with the watcher; `TryReloadShader` branches: render thread
    active → `Engine:reloadShaderOnRenderThread` + `shader:invalidate()`, else main-thread
    `Shader.TryLoad` + cache swap.
  - `script/Shared/Tools/ShaderErrorOverlay.lua` (new, 187) — red top banner, auto-shows on
    `HasNewErrors`, shows newest error + "+N more", ESC/click dismisses via
    `AcknowledgeErrors`.
  - Wired into the fork's `RenderCoreSystem` (init in ctor, `update()` per frame, overlay
    draw/input), and test states `DeferredTest.lua` / `RenderingTest.lua`.

**What this repo already has** (commit `731f1b0d`):

- Manual reload, triggered by **F5** (`GeneralActions.ReloadShaders`, consumed in
  `script/States/App/Tests/SolarSystemPlayable.lua:462-465`) →
  `Cache.ReloadShaders()` + `Material.ReloadAll()`.
- `Shader::reload()` (`shader.rs:275-331`) — **in-place** reload: re-loads from the stored
  `vs_name`/`fs_name`, compiles with non-panicking `try_create_gl_shader`/
  `try_create_gl_program` (`shader.rs:628-706`), and **keeps the old program on failure**.
  On success it swaps GL handles inside the shared `Rf<ShaderShared>`, so every `Shader`
  clone and `ShaderState` picks up the new program without re-fetching.
- `Material.ReloadAll` with a weak-valued material registry
  (`script/Shared/Rendering/Material.lua:35-52`) + `Material:reloadShader` that rebuilds the
  `ShaderState` and re-caches uniform ints.
- No file watching, no error queue (failures only `warn!` to the log), no overlay.

**Conflicts & reconciliation (recommended):**

1. **Keep this repo's in-place `Shader::reload()` as the direct-mode reload primitive.**
   It is simpler than the fork's `TryLoad` + cache-swap: because the GL handles swap inside
   the shared `Rf`, existing `ShaderState`s and materials keep working, and the fork's
   `lastWorkingShaders` fallback map becomes unnecessary (keep-old-on-failure already gives
   "last working"). Port from the fork only what's missing: the **watcher** (auto-trigger),
   the **error queue** (report from `try_create_gl_shader/program` instead of just
   `warn!`), the **overlay**, and optionally `create_error_shader` for first-load failures
   (initial `Shader::load` still panics here today).
2. **Cache key format**: adopt the fork's `vs..':'..fs` key and `shaderInfo` map in
   `script/Render/Cache.lua` (needed for watcher registration and error attribution). This
   repo's current key is `vs..fs` (`Cache.lua:50-58`).
3. **Fix the fork's key-format bug while porting** — three formats disagree in the fork:
   Cache/watcher use `vs..':'..fs`, `ShaderHotReload:registerMaterial` builds `vs..fs`
   (no separator), and `Shader::start` strips `vertex/`/`fragment/` prefixes for the
   command-mode bind key. Result: `materialsByShader` lookups can miss and materials don't
   refresh. Pick **one** canonical key (recommend `vs:fs` with prefixes) and use it for
   watcher registration, material registration, error attribution, and (later, Unit C) the
   render-thread `shader_key`.
4. **F5 stays** as the manual fallback path alongside the watcher.

### Unit B — Camera/Light UBO system + GLSL migration

**What the fork built:**

- `engine/lib/phx/src/render/ubo.rs` (new, 463) — std140 `#[repr(C, align(16))]` structs:
  `CameraUboData` (288 bytes: `mView`/`mProj`/`mViewInv`/`mProjInv` + `eye` + `starDir`,
  binding point **0**; size asserted by unit test), `MaterialUboData` (32 B, binding **1**),
  `LightUboData` (32 B, binding **2**); a `UniformBuffer` GL wrapper; thread-local
  `CAMERA_UBO`/`LIGHT_UBO` for **direct GL mode** with `update_global_camera_ubo()` /
  `update_global_light_ubo()` helpers.
- Engine FFI (`engine.rs`): `CreateCameraUBO`, `UpdateCameraUBO(mView, mViewInv, mProj,
  eye, starDir)`, `CreateLightUBO`, `UpdateLightUBO(pos, radius, rgb, intensity)` — each
  branches: command mode → submit `Update*UBO` command; direct mode → `update_global_*_ubo`.
- `shader.rs::create_gl_program` binds uniform blocks at link time: `CameraUBO` → 0,
  `LightUBO` → 2 (fork `shader.rs:778-788`); the render thread's own `create_shader` does
  the same.
- GLSL (`res/shader/`):
  - New `include/camera_ubo.glsl` / `light_ubo.glsl` / `material_ubo.glsl` — `layout(std140)`
    blocks with `#define` accessors so existing shader code keeps using the same identifiers
    (`mView`, `eye`, `starDir`, `lightPos`, …).
  - `include/vertex.glsl` and `include/fragment.glsl` now `#include camera_ubo` and
    **delete** the individual `uniform mat4 mView/mViewInv/mProj/mProjInv; uniform vec3
    eye/starDir;` declarations → **every runtime shader now reads camera state from the
    UBO**.
  - `#autovar` lines for those uniforms stripped from ~15 shaders (`vertex/wvp.glsl`,
    `vp.glsl`, `farplane.glsl`, `worldray.glsl`, 4× `billboard/*.glsl`, materials
    asteroid/atmosphere/devmatenv/metal/moon/ore/planet/planetring/uv_metal,
    `effect/dustfleck`).
  - `vertex/worldray.glsl` additionally switched to **camera-relative rays**
    (`worldOrigin = 0`, `worldDir = mat3(mViewInv) * unproject`) — precision fix for large
    coordinates; the deferred light shaders depend on it.
  - `fragment/light/point.glsl` migrated to `light_ubo` + `deferred_read`.
  - `gen/nebula*.glsl` renamed their `starDir` uniform → `genStarDir` (collides with the
    `camera_ubo` macro otherwise) + `Nebula1.lua` updated.
- Lua drivers: `CameraManager:beginDraw` calls `Engine:updateCameraUBO(...)` **every frame**
  (while still pushing the legacy ShaderVars for compatibility); the fork's
  `RenderCoreSystem` deferred pass calls `Engine:updateLightUBO(...)` per light;
  `PlanetTest.lua` calls `Engine:createCameraUBO()`.
- **`MaterialUboData` is dead scaffolding even in the fork**: no GLSL consumer, no Engine
  FFI. Skip it (keep the struct if it comes along with the file; don't wire it).

**What this repo already has:**

- `engine/lib/phx/src/render/thread/ubo.rs` (465 lines) **is the fork's `ubo.rs`** with the
  direct-GL-mode half **commented out** (thread-locals + `update_global_*` helpers, lines
  12-16 and 375-447). The render-thread half is wired: `Renderer`
  `CreateCameraUBO`/`UpdateCameraUBO`/... FFI (`renderer_queue.rs:316-415`), commands, and
  `render_thread.rs` handlers (incl. link-time block binding in its `create_shader`).
- **Nothing consumes it**: zero GLSL references to uniform blocks
  (`grep std140|CameraUBO` over `res/shader/` is empty), no hand-written Lua calls the
  `Renderer` UBO methods, and there is no Engine-level (direct-mode) FFI at all.

**Conflicts & hazards:**

1. **This repo's ~13 post-ancestor shaders must be migrated too.** `star.glsl`,
   `traveldrive.glsl` (vertex+fragment+filter), `lensflare.glsl`, `light/directional.glsl`,
   `ui/annulus|mappoints|solidcolor|trail3d.glsl`, `hologram3d.glsl`, `mappoints.glsl` all
   predate the UBO world and use the individual uniforms/autovars via `include/vertex|
   fragment.glsl`. Once the includes switch to `camera_ubo`, any of these that *also*
   declares `uniform vec3 eye` etc. locally will fail to compile (redefinition vs macro),
   and any that relies on `#autovar mView` will silently keep working only if the UBO is
   updated — audit each one.
2. **Atomicity**: after the include change, a frame without `Engine:updateCameraUBO` renders
   with a stale/identity camera. The GLSL changes, the Engine FFI, and the
   `CameraManager:beginDraw` update call must land in the same change.
3. **Keep this repo's `include/common.glsl`** (`farPlane = 1.0e8`; the fork still has
   `1.0e6`) — do not let the fork's version overwrite it.
4. This repo's `light/directional.glsl` and its deferred pipeline in `RenderCoreSystem`
   diverged from the fork's `light/global|composite` rewrites — Unit B should port **only**
   `camera_ubo`/`light_ubo`/`point.glsl`-style migration mechanics; the fork's lighting
   *look* changes belong to Unit D (optional).

### Unit C — Dual-mode (command-mode) shader path

Gated on the `render_mode.rs` + interception substrate — this is part of **main roadmap
Phase 2** in `ai/multithreaded_rendering.md`; do not port before it. Shader-specific deltas
to bring across at that point (fork refs):

- `ShaderShared` gains `resource_id: Option<ResourceId>`, retained `vs_src`/`fs_src` (for
  deferred render-thread compile), `pending_uniforms_by_name`; `ShaderAutoVar.name`
  `String → Arc<str>` for O(1) per-frame cloning (fork `shader.rs:27-57`).
- `start()` (fork `shader.rs:579-673`): command mode lazily submits `CreateShader { id }`
  then `BindShaderByResource { id, shader_key }`; applies pending + auto uniforms **by
  name** (`SetUniform*ByName`) since render-thread locations differ; Tex2D binds via
  `BindTexture2DByResource` with `ensure_resource_id()` lazy migration (`shader.rs:326-343`);
  direct mode unchanged. `stop()` submits `UnbindShader`. `invalidate()` clears
  `resource_id` after hot-reload.
- `shader_state.rs`: `elems: Vec<(i32, Data)>` → `Vec<UniformEntry { index, name: Arc<str>,
  data }>` storing both the GL location (direct) and name (command); `start()` branches by
  mode.
- `Engine:reloadShaderOnRenderThread(shaderKey, vs, fs)` (fork `engine.rs:307`) →
  `Shader::get_preprocessed_source` → blocking `handle.reload_shader`.
- **Already present in this repo** (from the earlier port): the entire render-thread side —
  `ReloadShader` command handling, `hot_reloaded_shaders` map with shader_key hot-swap
  preference in `BindShaderByResource`, per-program uniform caches (cleared on reload),
  `ShaderReloadResult` channel (`render/thread/render_thread.rs`,
  `shader_reload_result.rs`, `Renderer::reload_shader`). Only the main-thread half is
  missing.
- Unify the `shader_key` with the Unit A canonical key (see A.3) instead of the fork's
  prefix-stripping in `start()`.

### Unit D — GLSL extras (optional, independently portable)

- **Post-fx** (safe, additive): `fxaa.glsl` rewritten as full FXAA 3.11 (196 lines); new
  `fxaa_fast.glsl` (5-tap), `blur_fast.glsl` (7-sample bilinear Gaussian for bloom),
  `panini.glsl` (high-FOV projection); `bloomcomposite.glsl` simplified to
  `original + bloom * intensity`; `tonemap.glsl` gains a final LDR `clamp` (so FXAA gets
  LDR input); `PostFxConfig.lua`/`RenderingEnums.lua` updated. This repo's post-fx stack
  diverged (own tonemappers, lensflare, traveldrive filters) → port as **opt-in additions**
  keyed by new filter names; don't overwrite existing filters wholesale, and check
  `script/Config/Render/PostFxConfig.lua` for parameter shape changes (fork fxaa takes
  `{strength, edgeThreshold, edgeThresholdMin}`).
- **Deferred**: `include/deferred_read.glsl` (G-buffer read helpers + material-ID
  constants), `material/solidcolor_deferred.glsl`, `DebugDeferred` material def,
  `DeferredTest.lua`, plus rewritten `light/global|composite.glsl`. This repo has its own
  deferred lighting; treat these as a visual change to be diffed on screen, not a mechanical
  port.
- **Instancing**: `include/instanced.glsl` (instance attributes at locations 4-8: mat4
  model + vec4 color; includes `camera_ubo`), `vertex/wvp_instanced.glsl`,
  `fragment/instanced_color.glsl`. Consumers are the fork's `InstanceBatch` /
  `DrawInstancedWithData` path — tie this to the batch-API work (main roadmap Phase 4 /
  batch revival), not to the shader port.

### Unit E — Documentation

Port `doc/engine/shader-system.md` (275 lines: hot-reload architecture diagram, component
tables, include-dependency tracking, error queue + overlay, Lua API reference, dual-mode
reload paths, UBO reference). **Fix while porting**: the doc's UBO field names are stale
(`ubo_view`, `matAlbedo`) vs the code (`ubo_mView`, `matColor`); update to whatever this
repo lands. Rewrite the reload sections to describe the in-place `Shader::reload()` model
chosen in Unit A rather than the fork's TryLoad/cache-swap.

---

## 3. Porting phases

### Phase S1 — Hot-reload upgrade (Unit A; no render-thread dependency)
1. Add `notify` (6.x, `default-features = false`, `macos_kqueue` on macOS) to
   `engine/lib/phx/Cargo.toml`.
2. Port `shader_watcher.rs` and `shader_error.rs` into `engine/lib/phx/src/render/`
   (adjust module paths; register in `render/mod.rs`; run the FFI generator).
3. `shader.rs`: report compile/link failures from `try_create_gl_shader` /
   `try_create_gl_program` into `push_shader_error` (they currently only `warn!`);
   optionally add `create_error_shader` + `TryLoad` so *initial* loads stop panicking.
4. `Cache.lua`: switch key to `vs..':'..fs`, add `shaderInfo`, register with
   `ShaderWatcher` in `Cache.Shader`, add `Cache.TryReloadShader(key)` built on the
   **in-place** `shader:reload()` (not the fork's TryLoad swap); keep
   `Cache.ReloadShaders()` for F5.
5. Port `ShaderHotReload.lua` (drive `TryReloadShader` from watcher polls; use the single
   canonical key everywhere — fixes the fork's key-format bug) and
   `ShaderErrorOverlay.lua`; wire `init`/`update`/overlay into this repo's
   `RenderCoreSystem` and keep the F5 binding as manual fallback.

### Phase S2 — Camera/Light UBO migration (Unit B; atomic landing)
1. Re-enable the direct-GL half of `render/thread/ubo.rs` (uncomment thread-locals +
   `update_global_*_ubo`; the fork's `render/ubo.rs` is the reference for the exact code).
2. Add Engine-level dual-mode FFI: `CreateCameraUBO` / `UpdateCameraUBO` / `CreateLightUBO`
   / `UpdateLightUBO` — direct mode → `update_global_*`; command mode → forward to the
   existing `Renderer` methods. (Until main-roadmap Phase 2 lands, "command mode" is simply
   `Engine.renderer.is_some()`.)
3. `shader.rs::create_gl_program`: bind `CameraUBO` → 0 and `LightUBO` → 2 after linking
   (the render thread's compiler already does this).
4. GLSL: copy `include/camera_ubo.glsl` + `light_ubo.glsl` from the fork; apply the fork's
   diffs to `include/vertex.glsl` / `fragment.glsl` (add include, delete individual camera
   uniforms); strip the now-redundant `#autovar` lines from the ~15 shaders the fork
   touched; port the `worldray.glsl` camera-relative fix and the `gen/nebula*` →
   `genStarDir` rename (+ `Nebula1.lua`). **Keep this repo's `common.glsl`.**
5. Audit + migrate this repo's 13 post-ancestor shaders (star, traveldrive×3, lensflare,
   light/directional, ui/×4, hologram3d, mappoints) for local camera-uniform declarations
   and autovars.
6. Lua: `CameraManager:beginDraw` → `Engine:updateCameraUBO(...)` per frame (keep pushing
   legacy ShaderVars during the transition, as the fork does); light passes in
   `RenderCoreSystem` → `Engine:updateLightUBO(...)` where `light_ubo`-based shaders are
   used.
7. Skip `MaterialUboData` wiring (dead in the fork too); leave the struct as-is.

### Phase S3 — Command-mode shader path (Unit C)
Execute as part of `ai/multithreaded_rendering.md` **Phase 2** (needs `render_mode.rs` /
global submission). Bring the §2.C deltas into `shader.rs` / `shader_state.rs` /
`engine.rs`; unify `shader_key` with the S1 canonical key; then `Cache.TryReloadShader`
gains its render-thread branch (`Engine:reloadShaderOnRenderThread` + `shader:invalidate()`).

### Phase S4 — Optional GLSL extras (Unit D)
Post-fx additions first (fxaa rework behind config, fxaa_fast/blur_fast/panini as new
filters); deferred-look changes and instancing shaders only alongside their respective
feature efforts.

### Phase S5 — Docs (Unit E)
Port and correct `doc/engine/shader-system.md`; cross-link with
`ai/multithreaded_rendering.md`.

---

## 4. Risks & verification

**Risks**
- S2 is the risky one: after the include rewrite every shader depends on per-frame UBO
  updates — a missed `updateCameraUBO` call site (multiple camera managers? render-to-
  texture passes with their own view?) renders garbage. Search for every place that
  currently pushes `mView`/`mProj` ShaderVars (`ShaderVar.PushMatrix` callers) and mirror
  each with a UBO update or confirm it's covered by `CameraManager:beginDraw`.
- Shader macro collisions: `camera_ubo.glsl` `#define`s (`mView`, `eye`, `starDir`, …)
  will break any shader that declares those names locally — the nebula `genStarDir` rename
  is the known case; this repo's extra shaders may hide more.
- `notify` adds a native dependency; verify Linux (inotify) works in this repo's dev
  environment and that watching is disabled gracefully in shipped builds
  (`ShaderWatcher.Init` is opt-in from Lua, so simply not calling it suffices).

**Verification**
- S1: run `SolarSystemPlayable` (or `RenderingTest`); edit a material `.glsl` → shader
  reloads automatically within a frame; edit an *include* file → dependent shaders reload;
  introduce a syntax error → overlay shows the compile log, rendering keeps the old
  program; fix it → overlay clears. F5 still force-reloads everything.
  `cargo test -p phx` for the watcher/error unit tests that come with the fork files.
- S2: `cargo test -p phx` (288-byte `CameraUboData` layout test already exists in
  `thread/ubo.rs`); run `PlanetTest`, `SolarSystemPlayable`, and the post-fx pipeline and
  visually compare against pre-migration screenshots; grep `res/shader` for leftover
  `uniform mat4 mView` / `#autovar mat4 mView` stragglers (should be zero outside
  `camera_ubo.glsl`).
- S3: covered by the main roadmap's Phase 2/3 verification (runtime toggle, identical
  output in both modes; hot-reload while the render thread is active).

**Upstreaming note**: the key-format bug fix (A.3) and the stale doc field names (E) are
worth sending back to the fork if it stays alive.

---

## 5. Quick reference — fork file → target

| Fork (`ltheory-redux`) | This repo | Unit / phase |
|---|---|---|
| `render/shader_watcher.rs` | `render/shader_watcher.rs` (new) | A / S1 |
| `render/shader_error.rs` | `render/shader_error.rs` (new) | A / S1 |
| `render/shader.rs` (error paths, `create_error_shader`, `TryLoad`) | `render/shader.rs` (merge into existing reload code) | A / S1 |
| `script/Render/ShaderHotReload.lua` | same path (new; adapt to in-place reload) | A / S1 |
| `script/Render/Cache.lua` (key/`shaderInfo`/watcher/TryReload) | same path (merge; drop `lastWorkingShaders`) | A / S1 |
| `script/Shared/Tools/ShaderErrorOverlay.lua` | same path (new) | A / S1 |
| `render/ubo.rs` (direct-mode half) | `render/thread/ubo.rs` (uncomment + adapt) | B / S2 |
| `engine/engine.rs` UBO FFI | `engine/engine.rs` (new dual-mode methods) | B / S2 |
| `render/shader.rs` link-time block binding | `render/shader.rs::create_gl_program` | B / S2 |
| `res/shader/include/{camera_ubo,light_ubo}.glsl` | same paths (new) | B / S2 |
| `res/shader/include/{vertex,fragment}.glsl` diffs | same paths (apply diff; keep local additions) | B / S2 |
| `res/shader/vertex/worldray.glsl`, `gen/nebula*.glsl`, autovar strips | same paths + audit this repo's 13 extra shaders | B / S2 |
| `script/Modules/Cameras/Managers/CameraManager.lua` (UBO update) | same path (merge) | B / S2 |
| `render/shader.rs` command-mode (`resource_id`, by-name uniforms) | `render/shader.rs` | C / S3 (with main roadmap Phase 2) |
| `render/shader_state.rs` `UniformEntry` | `render/shader_state.rs` | C / S3 |
| `engine/engine.rs::reload_shader_on_render_thread` | `engine/engine.rs` | C / S3 |
| `res/shader/fragment/filter/{fxaa,fxaa_fast,blur_fast,panini,bloomcomposite,tonemap}.glsl` | same paths (selective / opt-in) | D / S4 |
| `res/shader/include/{deferred_read,instanced}.glsl`, `wvp_instanced`, `instanced_color`, `solidcolor_deferred` | same paths (with their feature efforts) | D / S4 |
| `doc/engine/shader-system.md` | `doc/engine/shader-system.md` (new; corrected) | E / S5 |
