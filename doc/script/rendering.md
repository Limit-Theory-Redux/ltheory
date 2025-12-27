# Rendering System (Lua)

The rendering system is orchestrated by `RenderCoreSystem.lua` which manages the multi-pass rendering pipeline, deferred lighting, and post-processing.

## Render Pipeline

```
┌─────────────────────────────────────────────────────────────┐
│                    GEOMETRY PASSES                          │
├─────────────────────────────────────────────────────────────┤
│ Opaque Pass    → G-Buffer (albedo, normals, depth)         │
│ Lighting Pass  → Light accumulation (if point lights)      │
│ Composite Pass → albedo × lighting                         │
│ Additive Pass  → Additive blended objects                  │
│ Alpha Pass     → Transparent objects                       │
├─────────────────────────────────────────────────────────────┤
│                  POST-PROCESSING (HDR)                      │
├─────────────────────────────────────────────────────────────┤
│ Aberration     → Chromatic aberration                      │
│ Bloom          → Bright area glow                          │
│ Color Grade    → Color science presets                     │
│ Tonemap        → HDR → LDR conversion                      │
├─────────────────────────────────────────────────────────────┤
│                  POST-PROCESSING (LDR)                      │
├─────────────────────────────────────────────────────────────┤
│ FXAA           → Edge antialiasing                         │
│ Sharpen        → Contrast adaptive sharpening              │
│ Dither         → Banding reduction                         │
│ Panini         → FOV distortion correction                 │
│ Vignette       → Edge darkening                            │
│ Radial Blur    → Motion effect                             │
├─────────────────────────────────────────────────────────────┤
│ Present        → Display to screen                         │
└─────────────────────────────────────────────────────────────┘
```

## G-Buffer Architecture

| Buffer | Format | Contents |
|--------|--------|----------|
| `buffer0` | RGBA16F | Albedo (RGB) + Alpha |
| `buffer1` | RGBA16F | Normal (encoded 2D) + Roughness + Material ID |
| `zBufferL` | R32F | Linear depth (camera distance) |
| `lightAccum` | RGBA16F | Accumulated lighting |

### Material Types

```glsl
Material_Diffuse = 0.0   // Lambertian diffuse
Material_Metal   = 0.25  // PBR Cook-Torrance
Material_Ice     = 0.50  // (reserved)
Material_NoShade = 0.75  // Unlit
```

### Writing to G-Buffer (Material Shaders)

```glsl
#include deferred

void main() {
    setAlbedo(color);               // → buffer0.rgb
    setAlpha(1.0);                  // → buffer0.a
    setNormal(N);                   // → buffer1.xy (encoded)
    setRoughness(roughness);        // → buffer1.z
    setMaterial(Material_Metal);    // → buffer1.w
    setDepth();                     // → zBufferL (linear distance)
}
```

## Deferred Lighting

### Point Light Component

```lua
local PointLightComp = require("Modules.Rendering.Components").PointLight

-- Create entity with light
local light = Entity.Create("MyLight",
    Physics.Transform(pos, rot),
    PointLightComp(
        Vec3f(1, 0.8, 0.6),  -- color
        2.0,                  -- intensity
        50.0                  -- radius
    )
)
```

### Lighting Pass Flow

```lua
-- In RenderCoreSystem:render()
if hasPointLights then
    self:renderGlobalLight()    -- Ambient/directional
    self:renderPointLights()    -- Per-light contribution
    self:compositeDeferredLighting()
end
```

### Light UBO Update

```lua
-- Camera-relative position for precision
local worldPos = transform:getPos()
local pos = worldPos:relativeTo(eye)

Engine:updateLightUBO(
    pos.x, pos.y, pos.z, radius,
    color.x, color.y, color.z, intensity
)
```

### Point Light Shader

```glsl
// light/point.glsl
#include deferred_read
#include light_ubo

void main() {
    // Read G-buffer
    vec3 N = decodeNormal(texture(texNormalMat, uv).xy);
    float depth = texture(texDepth, uv).x;

    // Reconstruct position (camera at origin)
    vec3 p = worldOrigin + depth * normalize(worldDir);

    // Light contribution
    vec3 L = lightPos - p;
    float dist = length(L);
    float attenuation = 1.0 / dist;

    // Apply based on material type
    if (mat == Material_Diffuse) {
        light = lightColor * attenuation * dot(N, normalize(L));
    } else if (mat == Material_Metal) {
        light = lightColor * attenuation * cookTorrance(...);
    }
}
```

## Post-Processing Configuration

All settings in `Config/Render/PostFxConfig.lua`:

### Bloom

```lua
bloom = {
    enable = true,
    radius = 32     -- Blur kernel size
}
```

3-stage process: bright extract → separable blur (3 iterations) → composite

### FXAA

```lua
fxaa = {
    enable = true,
    strength = 1.0,           -- Sub-pixel AA blend
    edgeThreshold = 0.063,    -- Minimum contrast
    edgeThresholdMin = 0.0312 -- Dark area threshold
}
```

Quality 3.11 implementation with edge-tracing search.

### Panini Projection

```lua
panini = {
    enable = true,
    distance = 0.5,   -- 0=off, 1=full cylindrical
    scale = 1.0       -- Vertical compensation
}
```

Reduces peripheral stretching at high FOV. Recommended:
- 70-90° FOV: distance = 0.3-0.4
- 90-110° FOV: distance = 0.5-0.6
- 120°+ FOV: distance = 0.7-1.0

### Tonemapping

```lua
tonemap = {
    enable = true,
    mode = Enums.Tonemappers.Illustris,  -- Space-optimized
    exposure = 1.2,
    autoExpose = {
        enable = false,
        speedUp = 0.5,
        speedDown = 0.3,
        minTarget = 0.4,
        maxTarget = 2.5
    }
}
```

Available tonemappers:
- `Linear`, `Reinhard`, `ACES`, `Filmic`, `Uncharted2`
- `Lottes`, `Uchimura`, `GranTurismo`, `NarkowiczACES`
- `ReinhardExtended`, `ReinhardLuminance`, `AgX`
- `Illustris` (space game optimized)

### Color Grading

```lua
colorgrade = {
    enable = true,
    mode = Enums.ColorGrades.Space,
    preExposure = 1.2,
    temperature = -0.05,    -- -1=blue, +1=orange
    tint = 0.0,             -- -1=green, +1=magenta
    saturation = 0.82,
    contrast = 1.0,
    brightness = 0.005,
    vibrance = 0.125,
    lift = { 0.0, 0.0, 0.01 },   -- Shadow RGB
    gamma = { 1.0, 1.0, 1.0 },   -- Midtone power
    gain = { 1.0, 1.0, 1.0 }     -- Highlight multiply
}
```

Presets: `Neutral`, `Cinematic`, `Space`, `Warm`, `Cool`, `Vibrant`, `Bleach`

### Other Effects

```lua
vignette = {
    enable = true,
    strength = 0.35,
    hardness = 17.0
}

sharpen = {
    enable = true,
    strength = 0.5
}

dither = {
    enable = true,
    strength = 0.3
}

aberration = {
    enable = true,
    strength = 0.3
}

radialblur = {
    enable = false,
    strength = 1.0,
    center = { 0.5, 0.5 }
}
```

## RenderCoreSystem API

### Initialization

```lua
local RenderCoreSystem = require("Modules.Rendering.Systems").RenderCoreSystem

function onInit()
    RenderCoreSystem:init()
end
```

### Rendering

```lua
function onRender(data)
    RenderCoreSystem:render(data)
end
```

### Debug Buffers

```lua
-- Toggle buffer visualization
Config.render.debug.showBuffers = true

-- Shows: buffer0, buffer1, zBufferL, lightAccum, etc.
```

## Coordinate System

**Camera-Relative Rendering:**

```
World Position → Camera-Relative = WorldPos - CameraPos
Camera always at origin (0, 0, 0)
```

This avoids float precision issues with large world coordinates (millions of units).

**Depth Storage:**
```glsl
setDepth():  zBufferL = length(fragmentPos - cameraPos)
```

**Position Reconstruction:**
```glsl
// In lighting shaders
vec3 fragmentPos = worldOrigin + depth * normalize(worldDir);
// worldOrigin = vec3(0) in camera-relative space
```

## Render Overlay

Debug overlay showing render statistics:

```lua
local RenderOverlay = require("Shared.Tools.RenderOverlay")

-- In render loop
RenderOverlay:draw()
```

Displays:
- FPS and frame time
- Draw calls and state changes
- Render thread statistics
- Memory usage

## Material System

### Creating Materials

```lua
local Material = require("Shared.Rendering.Material")

local mat = Material:new("vertex/wvp", "fragment/material/metal", BlendMode.Disabled)
mat:setFloat3("baseColor", 0.8, 0.6, 0.4)
mat:setFloat("roughness", 0.3)
mat:setFloat("metallic", 1.0)
```

### Material Definitions

Pre-defined materials in `Shared/Definitions/MaterialDefs.lua`:

```lua
MaterialDefs.Metal = {
    shader = { "wvp", "material/metal" },
    vars = {
        { name = "baseColor", type = "float3", value = {0.8, 0.8, 0.8} },
        { name = "roughness", type = "float", value = 0.3 },
    }
}
```

### Auto-Variables

Materials support automatic shader variable binding:

```lua
-- Per-object (updated each draw)
material:setAutoVar("objPos", function(entity)
    return entity:get(Transform):getPos():relativeTo(eye)
end)

-- Per-frame (updated once per frame)
material:setStaticVar("time", GameState.time)
```
