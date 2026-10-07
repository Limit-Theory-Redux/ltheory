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
   ┌─────┴─────┐                     Shader clone picks it up
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
simply isn't touched, so every clone of that `Shader`
automatically keeps rendering the previous program with no separate fallback cache.

### Rust Components

| File | Purpose |
|------|---------|
| `render/shader_watcher.rs` | File watching (`notify` crate), `#include` dependency tracking |
| `render/shader_error.rs` | Capped (10, FIFO) compile/reload error queue |
| `render/shader.rs` | Preprocessing (`#include`/`#group`), compilation, in-place reload, error-shader fallback |
| `render/thread/command_executor_gl.rs` | Actual GL shader compile/link (`create_shader`), including applying the `#group` layout and reflecting the blocks |
| `render/gpu/layout.rs` | `ShaderLayout` (recorded `#group` declarations), `BlockLayout` (reflected blocks) |

Both `ShaderWatcherInner` and the error queue (`ShaderErrorQueue`) are fields on
`RendererData` (`render/thread/renderer_data.rs`) — reached as `r.data.shader_watcher` /
`r.data.shader_errors` — **not** global `static`s. This repo removed its remaining Rust
globals. Every FFI method on `ShaderWatcher`/`ShaderError` takes
`r: &Renderer` or `r: &mut Renderer` as its first argument;
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
#include vertex   -- itself #includes view_block

-- Editing include/vertex.glsl or include/view_block.glsl triggers a
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

## Binding Layout (`#group`) and the UBOs

GLSL 330 has no `layout(binding=)`, so the preprocessor (`GLSLCode::preprocess`) owns the binding
assignment. A `#group N` line (N in 0..3) marks the uniform blocks and samplers that follow it as
belonging to bind group N; the directive is stripped from the GLSL, the declarations are recorded in a
`ShaderLayout`, and at link the GL executor applies the layout (`glUniformBlockBinding`, and `glUniform1i`
for each sampler) and reflects every active block (`glGetActiveUniformBlockiv`/`glGetActiveUniformsiv`).
Includes inherit the includer's group; a `#group` inside an include does not leak out of it.

| group | block bindings | texture units | contents |
|---|---|---|---|
| 0 frame | 0..3 | 0..2 | `ViewBlock` (per pass), `envMap`, `irMap` |
| 1 material | 4..7 | 3..9 | material params and textures (bind group) |
| 2 draw | 8..11 | 10..11 | per-draw block (`pass:alloc(T)`) |
| 3 pass inputs | 12..15 | 12..15 | `pass:setInputs(...)` textures |

The k-th block (sampler) declared in a group gets binding `group*4+k` (unit `first_unit(group)+k`).
A shader's own blocks are reachable from Lua: `shader:blockType('Params')` returns a LuaJIT struct type
with the block's exact byte layout, and `pass:alloc(Params)` returns a zeroed `Params*` that becomes the
group-2 block of the next draw. Loose `uniform` declarations outside any group still work through the old
`Shader:set*` API until S6.

### View block (group 0, binding 0)

`res/shader/include/view_block.glsl`, included by `include/vertex.glsl`, `include/fragment.glsl` and
`include/instanced.glsl` (so nearly every shader gets it for free). The engine allocates one per render pass
(`Renderer:beginPass`) from the camera set by `Renderer:setCamera`:

```glsl
#group 0
layout(std140) uniform ViewBlock {
    mat4 ubo_mView;
    mat4 ubo_mProj;
    mat4 ubo_mViewInv;
    mat4 ubo_mProjInv;
    vec4 ubo_eye;      // xyz = eye position, w = 1
    vec4 ubo_starDir;  // xyz = star direction, w = padding
    mat4 ubo_mProjUI;      // orthographic projection of the pass
    mat4 ubo_mWorldViewUI; // pass:setUiTransform
    vec4 ubo_viewport;     // x, y, w, h in pixels
    vec4 ubo_time;         // reserved
};

#define mView ubo_mView
#define mProj ubo_mProj
#define mViewInv ubo_mViewInv
#define mProjInv ubo_mProjInv
#define eye ubo_eye.xyz
#define starDir ubo_starDir.xyz
#define mProjUI ubo_mProjUI
#define mWorldViewUI ubo_mWorldViewUI
```

The Rust mirror is `ViewBlock` (`render/gpu/view_block.rs`, 448 bytes); every time a shader containing the
block links, its reflected size, member offsets and types are asserted against the struct. `fragment.glsl` also
declares `envMap` and `irMap` in group 0 (units 0 and 1); `Renderer:setEnvironment(env, ir)` supplies them and
every pass binds them. There are no per-shader environment variables any more.

Rendering is camera-relative: `eye` is always `(0,0,0)`, and
`vertex/fullscreen_ray.glsl` (used by every deferred-lighting fullscreen pass) reconstructs ray
direction as `mat3(mViewInv) * ...` with `worldOrigin = vec3(0)` rather than a world-space
point far from the origin — avoiding the precision loss that would come from reconstructing
and then subtracting back out a large coordinate.

**`mViewInv` is derived, not passed through.** `Renderer:setCamera` takes only the view and
projection matrices and the star direction; `ViewBlock::new` computes `mViewInv` as `view.inverse()`
in Rust rather than accepting an explicit value. This is deliberate: the two Lua camera paths
historically disagreed on whether a "real" `mViewInv` should carry the camera's true world-space
translation or zero translation, and the only consumer (`fullscreen_ray.glsl`) only ever uses the *rotation*
part via `mat3(mViewInv)` — so deriving it sidesteps the inconsistency instead of picking one
convention. If a future shader needs `mViewInv`'s translation, this will need to become an
explicit parameter instead.

### Draw block (group 2, binding 8) and material parameters (group 1)

Scene meshes get their per-draw data from one fixed block, `res/shader/include/draw_block.glsl`
(vertex side: `include/vertex_scene.glsl`, which is `vertex.glsl` with the block in place of the loose
`mWorld`/`mWorldIT` uniforms):

```glsl
#group 2
layout(std140) uniform DrawBlock {
    mat4 mWorld;
    mat4 mWorldIT;     // inverse transpose, computed once per transform
    vec4 drawScale;    // x = the body's uniform scale
    vec4 drawUser[7];  // material-defined per-draw values (MaterialType.perDraw)
};
```

`SceneList:submit` writes one `DrawBlock` (256 bytes, one ring stride) per drawn mesh straight into the
uniform ring; its Rust mirror is `DrawBlock` (`render/gpu/draw_block.rs`) and every shader that links it
is asserted against the struct. It must be the first block a shader declares in group 2.

A material's own parameters are a `MaterialParams` block in group 1, next to its samplers:

```glsl
#group 1
layout(std140) uniform MaterialParams { vec3 color1; float heightMult; /* ... */ };
uniform samplerCube surface;
```

Lua gets the block as a typed struct (`mat:params()`, the type is `shader:blockType('MaterialParams')`),
fills it and calls `mat:commit()`, one write per change. Anything that changes between draws of one
material goes in the draw block instead, through `drawUser`:
`#define time (drawUser[0].x)` with `perDraw = function(entity, user) user[0].x = ... end`. Define such
macros after the includes (a macro would also rewrite identifiers inside an include). Other code that draws
with its own pipeline (`Render/Pipelines.lua`) allocates a `DrawBlock` per draw with `pass:alloc` and uses
`drawUser` the same way (see `vertex/billboard/quad_draw.glsl`).

### Point light block (group 2)

`res/shader/include/light_block.glsl`, included only by `fragment/light/point.glsl`. It is the shader's
group-2 block (binding 8), written with `pass:alloc` before each light's `drawFullscreen`:

```glsl
#group 2
layout(std140) uniform PointLight {
    vec4 positionRadius;    // xyz = position, w = radius (0 = no falloff)
    vec4 colorIntensity;    // rgb = color, w = intensity
};

#define lightPos positionRadius.xyz
#define lightRadius positionRadius.w
#define lightColor (colorIntensity.rgb * colorIntensity.w)
#define lightIntensity colorIntensity.w
```

`fragment/light/directional.glsl` has its own group-2 `Params { vec3 lightDir; vec3 lightColor; }`: a
directional light has no position or radius, so it does not fit the point-light block's shape.

### Lua Usage

```lua
-- Camera: once per frame, from CameraManager:beginDraw(). Every pass that
-- begins afterwards renders with it.
Renderer:setCamera(mView, mProj, starDir)

-- starDir comes from CameraManager:setStarDir(dir), not a per-frame parameter
-- to the caller - set it once when it changes (e.g. on entering a system) and
-- it's picked up on the next beginDraw().

-- Environment maps (group 0): when the skybox's nebula is generated.
Renderer:setEnvironment(envMap, irMap)

-- Light: once per point light in the deferred lighting pass
-- (RenderCoreSystem:deferredLighting()), immediately before its drawFullscreen:
local p = pass:alloc(pointShader:blockType('PointLight'))
p.positionRadius.x, p.positionRadius.y, p.positionRadius.z, p.positionRadius.w = x, y, z, radius
p.colorIntensity.x, p.colorIntensity.y, p.colorIntensity.z, p.colorIntensity.w = r, g, b, intensity
pass:drawFullscreen()
```

### Nebula Generation — `genStarDir`, Not `starDir`

`res/shader/fragment/gen/nebula*.glsl` bake a cubemap using a caller-supplied "sun
direction" that's semantically a *generation parameter*, unrelated to the live camera's
`starDir`. Before the UBO migration this worked by accident (both were the same plain
`uniform vec3 starDir`); now that `starDir` is a `#define` resolving to the view block, the
nebula generators use a distinctly-named `uniform vec3 genStarDir` instead, set by
`script/Shared/Generation/Nebula1.lua` through the shader's `Params` block (`p.genStarDir`).

### No Loose Uniforms; the Immediate (UI) Shaders

Every active uniform of a program must be a member of a `#group` uniform block or a sampler:
`create_shader` (GL) fails the link otherwise, naming the offending uniforms, and the shader
falls back to the error shader. There is no `shader:setFloat(name, ...)` any more.

The UI shaders (`fragment/ui/*`, drawn through `Imm`, see `render-api-v2.md` section 3d and the
S6 notes) have no uniform at all: `vertex/imm2d.glsl` passes the vertex color and the shape
parameters as `flat` varyings (`imm_color`, `imm_p`, `imm_q`, declared by `include/imm.glsl`), and
each fragment shader names the parameters it reads with `#define`s over them. The glyph atlas
page (`ui/text`) and images (`ui/image`, `ui/icon`) are the group 3 sampler of slot 0. Mesh-based
UI shaders (`vertex/mappoints`, `vertex/hologram3d`) declare a `Params` block under `#group 2`
and forward their color to the same `imm_color` varying.
