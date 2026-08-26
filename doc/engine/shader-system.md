# Shader System

The shader system provides GLSL compilation and caching (`render/shader.rs`), automatic
hot-reload via file watching (`render/shader_watcher.rs`, `render/shader_error.rs`), and
two uniform buffer objects shared by every shader (`render/thread/ubo.rs`).

There is a single rendering path: all GL work happens on the render thread and is reached
through `Renderer`/`RenderCommand`, whether that thread is the dedicated render thread
(default) or running inline on the calling thread (`immediate` cargo feature, for
debugging/comparison). Nothing here branches on that distinction — it's an implementation
detail of `Renderer`, not something shader code or Lua callers need to know about.

## Hot-Reload System

Enables live editing of shader files during development. Shaders are automatically
recompiled without restarting the application; F5 (`GeneralActions.ReloadShaders`) remains
available as a manual fallback that reloads every cached shader unconditionally.

### Architecture

```
File System Watch (Rust, via `notify`)
         ↓
   ShaderWatcher.Poll()          -- once per frame, from Application:onPreRender
         ↓
   Changed shader key detected
         ↓
  Cache.GetShader(key):reload()  -- in-place: swaps the GL program inside the
         │                           shared Rf<ShaderShared>, so every existing
   ┌─────┴─────┐                     Shader clone/ShaderState picks it up
   │           │
 Success    Failure
   │           │
   ↓           ↓
 Material   Old program kept (reload never replaces a working
 .reload()  program with a broken one); error pushed to the queue
   │           │
   ↓           ↓
 Clear      Error overlay shows the compile error
 errors
```

Unlike a cache-swap design, this repo's `Shader::reload()` mutates the existing `Shader`'s
GL handle in place inside its shared `Rf<ShaderShared>` cell. That's what makes "keep the
last working version on failure" free: on success the handle is swapped; on failure it
simply isn't touched, so every clone of that `Shader` (and every `ShaderState` built from
it) automatically keeps rendering the previous program with no separate fallback cache.

### Rust Components

| File | Purpose |
|------|---------|
| `render/shader_watcher.rs` | File watching (`notify` crate), `#include` dependency tracking |
| `render/shader_error.rs` | Capped (10, FIFO) compile/reload error queue |
| `render/shader.rs` | Preprocessing (`#include`/`#autovar`), compilation, in-place reload, error-shader fallback |
| `render/thread/command_executor_gl.rs` | Actual GL shader compile/link (`create_shader`), including UBO block binding |

Both `ShaderWatcherInner` and the error queue (`ShaderErrorQueue`) are fields on
`RendererData` (`render/thread/renderer_data.rs`) — reached as `r.data.shader_watcher` /
`r.data.shader_errors` — **not** global `static`s. This repo removed its remaining Rust
globals; `ShaderVarMap` (the auto-var push/pop stack) went through the same change earlier
and is the pattern these two follow. Every FFI method on `ShaderWatcher`/`ShaderError` takes
`r: &Renderer` or `r: &mut Renderer` as its first argument, exactly like `ShaderVar`'s do;
the hand-written `ffi_ext/ShaderWatcher.lua` / `ffi_ext/ShaderError.lua` wrappers inject the
global Lua `Renderer` so call sites don't need to pass it explicitly.

### Lua Components

| File | Purpose |
|------|---------|
| `script/Render/ShaderHotReload.lua` | Orchestration: init, per-frame poll + reload, material re-link |
| `script/Shared/Tools/ShaderErrorOverlay.lua` | Error banner (drawn inside `Application:immediateUI`) |
| `script/Render/Cache.lua` | Shader/texture/font caching, canonical `vs:fs` key |

## File Watching

```lua
-- Once, at the very start of Application:appInit() - before any shader is
-- loaded, so every later Cache.Shader() call self-registers. Also runs a
-- catch-up pass registering shaders that were already cached at this point
-- (materials loaded eagerly via MaterialDefs at `require`-time do exist
-- before this call, so the catch-up pass is not optional).
ShaderHotReload:init()

-- Once per frame, from Application:onPreRender
ShaderHotReload:update()  -- returns (reloadedCount, failedCount)
```

### Include Dependency Tracking

When a shader is registered, every file it (transitively) `#include`s is tracked, so editing
a shared include reloads all dependent shaders:

```glsl
-- res/shader/vertex/wvp.glsl
#include vertex   -- itself #includes camera_ubo

-- Editing include/vertex.glsl or include/camera_ubo.glsl triggers a
-- reload of wvp.glsl (and every other shader that includes either file).
```

`ShaderWatcher::register`'s dependency walk (`collect_shader_includes` in
`shader_watcher.rs`) is a separate, small re-implementation of the `#include` parsing that
`GLSLCode::preprocess` (`shader.rs`) already does for compilation — kept intentionally
duplicated rather than plumbing a visited-file list out of the compile path.

## Error Handling

### Error Queue (Rust)

```rust
pub struct ShaderErrorInfo {
    pub shader_key: String,   // canonical "vertex/wvp:fragment/material/metal"
    pub error_type: String,   // "compile" (this repo doesn't yet push "link" separately)
    pub message: String,      // OpenGL info-log, null bytes stripped
    pub timestamp: u64,       // ShaderError::Update()'s frame counter
}
```

Capped at 10 entries, oldest evicted first. Pushed from two call sites in `shader.rs`:
`Shader::from_preprocessed` (first-load failure) and `Shader::reload` (reload failure).

### First-Load Failure — Error Shader

Unlike reload (which has an old program to keep), a shader's *first* compile has nothing to
fall back to. Rather than panic, `Shader::from_preprocessed` pushes the error and returns a
tiny built-in "error shader" — a trivial vertex/fragment pair that always compiles and
renders solid magenta — so a broken shader is visibly wrong instead of crashing the app.

### Error Overlay (Lua)

Auto-shows a red banner across the top of the screen when
`ShaderError.HasNewErrors()` is true; auto-clears once the error queue empties (i.e. once
the shader is fixed and reloads successfully). ESC or a click dismisses it manually. Drawn
from `Application:onPostRender` inside `self:immediateUI(...)`, and dismiss input is handled
from `Application:onInput`.

## Lua API

### ShaderHotReload

```lua
-- Lifecycle
ShaderHotReload:init()
ShaderHotReload:shutdown()
ShaderHotReload:isActive() -> bool

-- Per-frame
ShaderHotReload:update() -> reloadedCount, failedCount

-- Material tracking (so a reloaded shader re-links the materials using it)
ShaderHotReload:registerMaterial(material, vs, fs)
ShaderHotReload:unregisterMaterial(material)

-- Error access (thin passthroughs to ShaderError)
ShaderHotReload:hasErrors() -> bool
ShaderHotReload:getErrorCount() -> number
ShaderHotReload:getLatestError() -> string|nil
ShaderHotReload:acknowledgeErrors()
ShaderHotReload:clearErrors()
```

There is no `reloadShader(vs, fs)` manual-trigger method here — F5's
`Cache.ReloadShaders()` already covers manual reload of everything, so a second
single-shader manual entry point wasn't added.

### ShaderError FFI

```lua
ShaderError.GetCount() -> number
ShaderError.HasNewErrors() -> bool
ShaderError.AcknowledgeErrors()
ShaderError.GetShaderKey(index) -> cstr
ShaderError.GetErrorType(index) -> cstr
ShaderError.GetMessage(index) -> cstr
ShaderError.GetTimestamp(index) -> number
ShaderError.Clear()
ShaderError.ClearAt(index)
ShaderError.ClearForShader(key)
ShaderError.Update()             -- called once per frame, advances the frame counter
ShaderError.GetLatestMessage() -> cstr
ShaderError.GetLatestShaderKey() -> cstr
```

`cstr`-returning methods can be `nil` (a null pointer, which compares equal to Lua `nil`);
guard with `ptr and ffi.string(ptr)` before use, as `ShaderErrorOverlay:draw()` does.

### ShaderWatcher FFI

```lua
ShaderWatcher.Init() -> bool
ShaderWatcher.Shutdown()
ShaderWatcher.IsActive() -> bool
ShaderWatcher.Register(shaderKey, vsPath, fsPath)
ShaderWatcher.Poll() -> count
ShaderWatcher.GetChanged(index) -> cstr
ShaderWatcher.ClearChanged()
```

## Shader Caching

### Cache.lua

```lua
-- Get/create a shader; canonical key is 'vs:fs' (colon-separated - a plain
-- 'vs..fs' concatenation collides, e.g. Cache.Shader('a','bc') vs
-- Cache.Shader('ab','c')).
local shader = Cache.Shader('wvp', 'material/metal')

-- Look up an already-cached shader/its source paths by key (used by
-- ShaderHotReload, not typically called directly).
Cache.GetShader(key) -> Shader|nil
Cache.GetShaderKeys() -> string[]
Cache.GetShaderInfo(key) -> { vsPath, fsPath }|nil

-- F5 manual fallback: reload every cached shader unconditionally.
Cache.ReloadShaders() -> reloadedCount, failedCount
```

There's no `lastWorkingShaders` fallback table here (unlike a cache-swap design would need)
because `Shader::reload()` already keeps the old program in place on the Rust side.

## Debugging

```bash
RUST_LOG=debug ./bin/ltr RenderingTest
```

Representative log lines:

```
INFO  ShaderWatcher: Watching directory "./res/shader"
DEBUG ShaderWatcher: Registered shader 'wvp:material/metal' watching 9 files
DEBUG ShaderWatcher: file changed "/…/res/shader/fragment/material/metal.glsl" -> shader 'wvp:material/metal'
INFO  [lua] ShaderHotReload: Reloading 'wvp:material/metal'
ERROR Shader compile error for 'vertex/wvp:fragment/material/metal': Fragment shader error: 0(596) : error C0000: syntax error, unexpected reserved word "this" at token "this"
WARN  Shader '[vs: vertex/wvp, fs: fragment/material/metal]' reload failed: Fragment shader error: …
INFO  Reloaded shader [vs: vertex/wvp, fs: fragment/material/metal]   -- on the next successful reload
```

## UBO System

Two uniform buffer objects are shared by every shader that includes them, avoiding
per-draw-call uniform pushes for data that's constant for the whole frame (camera) or set
once per light in a deferred pass. Binding points are assigned at link time in
`command_executor_gl.rs::create_shader` via `glUniformBlockBinding` (GLSL 330 has no
`binding` layout qualifier). There is no Material UBO in this repo — the fork that this
system was ported from has one, but nothing consumes it there either; it wasn't ported.

### Camera UBO (binding 0)

`res/shader/include/camera_ubo.glsl`, included by `include/vertex.glsl` and
`include/fragment.glsl` (so nearly every shader gets it for free):

```glsl
layout(std140) uniform CameraUBO {
    mat4 ubo_mView;
    mat4 ubo_mProj;
    mat4 ubo_mViewInv;
    mat4 ubo_mProjInv;
    vec4 ubo_eye;      // xyz = eye position, w = padding
    vec4 ubo_starDir;  // xyz = star direction, w = padding
};

#define mView ubo_mView
#define mProj ubo_mProj
#define mViewInv ubo_mViewInv
#define mProjInv ubo_mProjInv
#define eye ubo_eye.xyz
#define starDir ubo_starDir.xyz
```

Rendering is camera-relative: `eye` is always pushed as `(0,0,0)`, and
`vertex/worldray.glsl` (used by every deferred-lighting fullscreen pass) reconstructs ray
direction as `mat3(mViewInv) * ...` with `worldOrigin = vec3(0)` rather than a world-space
point far from the origin — avoiding the precision loss that would come from reconstructing
and then subtracting back out a large coordinate.

**`mViewInv` is derived, not passed through.** `Renderer::update_camera_ubo`
(`render/thread/renderer_ffi.rs`) takes only `mView`/`mProj` plus eye/starDir components; it
computes `mViewInv` as `view.inverse()` in Rust rather than accepting an explicit value. This
is deliberate: the two Lua call sites that populate the camera (see below) disagree on
whether a "real" `mViewInv` should carry the camera's true world-space translation or zero
translation, and the only consumer (`worldray.glsl`) only ever uses the *rotation* part via
`mat3(mViewInv)` — so deriving it sidesteps the inconsistency instead of picking one
convention. If a future shader needs `mViewInv`'s translation, this will need to become an
explicit parameter instead.

### Light UBO (binding 2)

`res/shader/include/light_ubo.glsl`, included only by `fragment/light/point.glsl`:

```glsl
layout(std140) uniform LightUBO {
    vec4 ubo_positionRadius;    // xyz = position, w = radius (currently unused)
    vec4 ubo_colorIntensity;    // rgb = color, w = intensity
};

#define lightPos ubo_positionRadius.xyz
#define lightRadius ubo_positionRadius.w
#define lightColor (ubo_colorIntensity.rgb * ubo_colorIntensity.w)
#define lightIntensity ubo_colorIntensity.w
```

`fragment/light/directional.glsl` (this repo's, not present in the fork this was ported
from) intentionally stays on plain `uniform vec3 lightDir/lightColor` — a directional light
has no position or radius, so it doesn't fit the point-light UBO's shape.

### Lua Usage

```lua
-- Once at startup (script/Main.lua, right after Renderer = Engine:renderer()):
Renderer:createCameraUbo()
Renderer:createLightUbo()

-- Camera: once per frame, from both CameraManager:beginDraw() (used by
-- RenderCoreSystem-driven app states) and the legacy Camera:beginDraw()
-- (used by GameView / the default LTheoryRedux app) - both live camera
-- paths must stay in lockstep, since they share the same shaders.
Renderer:updateCameraUbo(mView, mProj, eyeX, eyeY, eyeZ, starDirX, starDirY, starDirZ)

-- starDir comes from CameraManager:setStarDir(dir) / Camera:setStarDir(dir),
-- not a per-frame parameter to the caller - set it once when it changes
-- (e.g. on entering a system) and it's picked up on the next beginDraw().

-- Light: once per point light in the deferred pass (both
-- RenderCoreSystem:deferredLighting() and GameView:draw() call this in
-- their point-light loop, immediately before drawing that light's pass):
Renderer:updateLightUbo(posX, posY, posZ, radius, r, g, b, intensity)
```

### Nebula Generation — `genStarDir`, Not `starDir`

`res/shader/fragment/gen/nebula*.glsl` bake a cubemap using a caller-supplied "sun
direction" that's semantically a *generation parameter*, unrelated to the live camera's
`starDir`. Before the UBO migration this worked by accident (both were the same plain
`uniform vec3 starDir`); now that `starDir` is a `#define` resolving to the Camera UBO, the
nebula generators use a distinctly-named `uniform vec3 genStarDir` instead, set by
`script/Legacy/Systems/Gen/Nebula/Nebula1.lua` via `ss:setFloat3('genStarDir', ...)`.
