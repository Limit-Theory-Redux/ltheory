# Shader System

The shader system provides GLSL shader compilation, caching, and hot-reloading for live development.

## Hot-Reload System

Enables live editing of shader files during development. Shaders are automatically recompiled without restarting the application.

### Architecture

```
File System Watch (Rust)
         ↓
   ShaderWatcher.Poll()
         ↓
   Changed shader detected
         ↓
  Cache.TryReloadShader()
         ↓
   ┌─────┴─────┐
   │           │
 Success    Failure
   │           │
   ↓           ↓
 Material   Keep last
 .reload()  working version
   │           │
   ↓           ↓
 Clear      Show error
 errors     overlay
```

### Rust Components

| File | Purpose |
|------|---------|
| `shader_watcher.rs` | File watching, dependency tracking |
| `shader_error.rs` | Thread-safe error queue |
| `shader.rs` | Compilation, fallback support |

### Lua Components

| File | Purpose |
|------|---------|
| `ShaderHotReload.lua` | Main orchestration, update loop |
| `ShaderErrorOverlay.lua` | Error banner display |
| `Cache.lua` | Shader caching, fallback versions |

## File Watching

The watcher monitors shader directories recursively and tracks `#include` dependencies:

```lua
-- Initialization (in RenderCoreSystem)
ShaderHotReload:init()

-- Per-frame update
ShaderHotReload:update()  -- Returns (reloaded_count, failed_count)
```

### Include Dependency Tracking

When a shader is registered, all its `#include` files are tracked:

```glsl
// In vertex/wvp.glsl
#include vertex
#include camera_ubo

// Changing include/vertex.glsl or include/camera_ubo.glsl
// will trigger reload of wvp.glsl
```

## Error Handling

### Error Queue (Rust)

Thread-safe queue limited to 10 errors (FIFO):

```rust
struct ShaderErrorInfo {
    shader_key: String,    // "vertex/wvp:fragment/solid"
    error_type: String,    // "vertex compile" | "fragment compile" | "link"
    message: String,       // OpenGL error message
    timestamp: u64,        // Frame number
}
```

### Error Overlay (Lua)

Auto-displays on compilation failure:

```
┌──────────────────────────────────────────────────────┐
│ SHADER ERROR: vertex/test:fragment/test (compile)   │
│ error: expected ';' at line 42                      │
│ Press ESC to dismiss | Fix shader to auto-clear     │
└──────────────────────────────────────────────────────┘
```

- Auto-shows when `ShaderError.HasNewErrors()` is true
- Auto-hides when shader is fixed
- ESC to dismiss manually

## Lua API

### ShaderHotReload

```lua
-- Lifecycle
ShaderHotReload:init()
ShaderHotReload:shutdown()
ShaderHotReload:isActive() -> bool

-- Per-frame
ShaderHotReload:update() -> reloaded, failed

-- Manual control
ShaderHotReload:reloadShader(vs, fs) -> bool
ShaderHotReload:pollChangedShaders() -> keys

-- Error access
ShaderHotReload:hasErrors() -> bool
ShaderHotReload:getLatestError() -> string
ShaderHotReload:acknowledgeErrors()
ShaderHotReload:clearErrors()

-- Material tracking
ShaderHotReload:registerMaterial(material, vs, fs)
ShaderHotReload:unregisterMaterial(material)
```

### ShaderError FFI

```lua
ShaderError.GetCount() -> number
ShaderError.HasNewErrors() -> bool
ShaderError.GetShaderKey(index) -> string
ShaderError.GetErrorType(index) -> string
ShaderError.GetMessage(index) -> string
ShaderError.Clear()
ShaderError.ClearForShader(key)
```

### ShaderWatcher FFI

```lua
ShaderWatcher.Init()
ShaderWatcher.Shutdown()
ShaderWatcher.IsActive() -> bool
ShaderWatcher.Register(key, vs_path, fs_path)
ShaderWatcher.Poll() -> count
ShaderWatcher.GetChanged(index) -> key
ShaderWatcher.ClearChanged()
```

## Render Thread Compatibility

Works in both modes:

**Direct GL Mode:**
```
File Change → poll() → Shader.TryLoad() (main thread)
```

**Command Mode:**
```
File Change → poll() → Engine.reloadShaderOnRenderThread() (render thread)
```

Both modes:
1. Re-register with watcher post-reload
2. Clear previous errors
3. Fall back to last working version on failure

## Shader Caching

### Cache.lua

```lua
-- Get/create shader
local shader = Cache.Shader('vertex_name', 'fragment_name')

-- Key format: "vertex_name:fragment_name"

-- Reload with fallback
local shader, success = Cache.TryReloadShader(key)

-- State maintained:
shaders = {}              -- Active cache
shaderInfo = {}           -- {vs, fs, vsPath, fsPath}
lastWorkingShaders = {}   -- Fallback on error
```

## Debugging

Enable verbose logging:

```bash
RUST_LOG=debug ./bin/ltr RenderingTest
```

Key log lines:
```
ShaderWatcher: Watching directory "/path/to/res/shader"
ShaderWatcher: Registered shader 'wvp:solidcolor' watching 3 files
ShaderWatcher: File event Modify for ["/path/vertex/wvp.glsl"]
Shader compile error for 'vertex/wvp:fragment/solidcolor': error: ...
ShaderHotReload: Successfully recompiled 'wvp:solidcolor'
```

## UBO System

Uniform Buffer Objects provide efficient data transfer to shaders.

### Camera UBO (Binding 0)

```glsl
// camera_ubo.glsl
layout(std140) uniform CameraUBO {
    mat4 ubo_view;
    mat4 ubo_proj;
    mat4 ubo_viewInv;
    mat4 ubo_projInv;
    vec4 ubo_eye;
    vec4 ubo_starDir;
};

#define eye ubo_eye.xyz
#define mView ubo_view
#define mProj ubo_proj
// etc.
```

### Material UBO (Binding 1)

```glsl
// material_ubo.glsl
layout(std140) uniform MaterialUBO {
    vec4 ubo_albedoAlpha;
    vec4 ubo_metalRoughEmission;
};

#define matAlbedo ubo_albedoAlpha.rgb
#define matAlpha ubo_albedoAlpha.a
#define matMetallic ubo_metalRoughEmission.x
#define matRoughness ubo_metalRoughEmission.y
```

### Light UBO (Binding 2)

```glsl
// light_ubo.glsl
layout(std140) uniform LightUBO {
    vec4 ubo_positionRadius;
    vec4 ubo_colorIntensity;
};

#define lightPos ubo_positionRadius.xyz
#define lightRadius ubo_positionRadius.w
#define lightColor (ubo_colorIntensity.rgb * ubo_colorIntensity.w)
```

### Lua Usage

```lua
-- Camera (updated per-frame by CameraManager)
Engine:updateCameraUBO(view, proj, viewInv, projInv, eye, starDir)

-- Material (updated per-material)
Engine:updateMaterialUBO(r, g, b, a, metallic, roughness, emission)

-- Light (updated per-light in deferred pass)
Engine:updateLightUBO(x, y, z, radius, r, g, b, intensity)
```
