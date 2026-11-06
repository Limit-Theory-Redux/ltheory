# Renderer: contributor guide

The renderer runs on wgpu (Vulkan, Metal or DX12, the platform's native API).
This page is the entry point: how a draw gets from Lua to the GPU, who owns
what, and how to add things. Details live in
[render-thread.md](render-thread.md) (threading, fences, startup and shutdown),
[shader-system.md](shader-system.md) (shaders, `#group`, hot reload) and
[batch-rendering.md](batch-rendering.md) (scene list, instancing).

## From Lua to the GPU

```
Main thread                                  Render thread               GPU
───────────                                  ─────────────               ───
Lua: materials, scene, passes, Imm
  │  FFI (engine/lib/phx/script/ffi_gen, ffi_ext)
  ▼
Renderer front end (render/gpu/)
  records PassCmds, writes uniform and
  vertex staging rings, owns resource ids
  │  RenderCommand over a bounded channel
  ▼                                          WgpuCommandExecutor
                                             (render/thread/
                                              command_executor_wgpu/)
                                             owns every wgpu object,
                                             encodes passes, submits ──► queue
```

| What | Created by | Written by | Reaches the GPU | Freed |
|---|---|---|---|---|
| Mesh, texture, shader | Lua (`Mesh.*`, `Tex2D.Create`, `Cache.Shader`) | Lua via methods | `Create*`/`Update*` commands | when the Lua object is collected: its `ResourceHandle` queues a destroy, sent once per frame |
| `ViewBlock` (group 0) | `beginPass` | engine (`Renderer:setCamera`, pass size) | uniform ring, per pass | ring slot reused 3 frames later |
| `MaterialParams` + textures (group 1) | `Material` | Lua: `mat:params()` fields, `mat:setTexture` | `mat:commit()` (one write) | with the `Material` |
| `DrawBlock` (group 2) | `SceneList`, per drawn mesh | engine (transform), `perDraw` callback (`drawUser`) | uniform ring, per draw | ring slot reused 3 frames later |
| Custom `Params` (group 2) | `pass:alloc(T)` | Lua, before the next draw | uniform ring, per draw | ring slot reused 3 frames later |
| Pass inputs (group 3) | `pass:setInputs` | Lua | bound at the next draw | end of pass |
| Bind group | `Renderer:createBindGroup` | `BindGroupDesc` | at creation | when the returned object is collected |

Ring memory is per frame slot (`MAX_FRAMES_IN_FLIGHT` = 3): the executor
waits for the GPU to finish a slot's previous frame before reusing it.

## Two levels of API

**Game objects** use material instances and the scene: no passes, no bind
groups.

```lua
-- A material type (Shared/Definitions/MaterialDefs.lua): shader, state,
-- MaterialParams defaults, textures, optional perDraw.
MaterialType {
    name     = "DebugColor",
    shader   = { "wvp", "material/solidcolor" },
    state    = { blend = BlendMode.Disabled },
    defaults = { color = Vec3f(1.0, 0.0, 1.0) },
}

-- An instance per object; parameters are written, then committed once.
local material = Materials.DebugColor:instance()
local p = material:params()
p.color.x, p.color.y, p.color.z = 0.2, 0.8, 0.3
material:commit()            -- never per frame, never inside an open pass

entity:add(RenderComp({ { mesh = Primitive.IcoSphere(5), material = material } }))
```

The render system collects visible meshes into a `SceneList`, fills each
mesh's `DrawBlock` (`mWorld`, `mWorldIT`, `drawScale`, and `drawUser` through
the type's `perDraw(entity, user)`), sorts by material and draws them in the
scene passes. The shaders include the shared blocks
(`res/shader/vertex/wvp.glsl` reads `mWorld`, `mProj`, `mView`):

```glsl
#include fragment
#include deferred

#group 1
layout(std140) uniform MaterialParams {
  vec3 color;
};

void main() {
  setAlbedo(color);
  ...
}
```

**Renderer and effect code** uses explicit passes, pipelines and per-draw
parameter blocks:

```lua
-- once
local shader   = Cache.Shader("fullscreen_ndc", "filter/blur")
local Params   = shader:blockType("Params")          -- LuaJIT struct, std140 layout from the shader
local passDesc = RenderPassDesc.Create("Post.blur")
passDesc:color(0, target:view(), LoadOp.Clear, 0, 0, 0, 1)

-- every frame
local pass = Renderer:beginPass(passDesc)
pass:setPipeline(Pipelines.get(shader, { vertex = VertexLayout.Fullscreen }))
pass:setInputs(src:mipView(0), Samplers.LinearClamp) -- group 3
local p = pass:alloc(Params)                         -- group 2, zeroed
p.size.x, p.size.y = src:getSize().x, src:getSize().y
p.radius = 8
pass:drawFullscreen()
pass:finish()
```

```glsl
#group 2
layout(std140) uniform Params {
    vec2 dir;
    vec2 size;
    int radius;
    float variance;
};
#group 3
uniform sampler2D src;
```

## Rules

- **Every uniform lives in a `#group` block.** A shader that reads a uniform
  outside one fails to compile, naming it. There is no `shader:setFloat`.
- **Groups:** 0 view (per pass), 1 material, 2 draw (`DrawBlock` or a custom
  block from `pass:alloc`), 3 pass inputs. Up to 4 blocks and 3/7/2/4 samplers
  per group (`render/gpu/layout.rs`).
- **std140**: block types come from the shader (`shader:blockType(name)`), so
  Lua never restates offsets. `ViewBlock` and `DrawBlock` have Rust mirrors
  (`render/gpu/view_block.rs`, `draw_block.rs`) checked against the reflected
  block when a shader is created and in `layout_parity` tests.
- **Pointers:** a `pass:alloc` pointer is valid until the next draw is
  recorded: fill it, then draw. `material:params()` stays valid until the
  material's shader is reloaded with a different block layout (the material
  refreshes it).
- **Passes** do not nest. A `RenderPass` handle records only into its own pass;
  using it after `finish` is an error naming both passes. `Renderer:currentPass()`
  returns a handle that can record but not finish.
- **No state stacks** except `ClipRect`: pipelines carry blend, cull and depth
  state; passes carry targets and load ops. The scissor of a draw is the
  `ClipRect` stack's rectangle within the pass's own `pass:setScissor`
  rectangle (`clearScissor` drops it; a new pass starts without one).
- **Resource writes** (texture uploads, `material:commit`) go outside open
  passes.
- **Data textures** read only with `texelFetch` should be created with
  `TexUsage.Data` (`Tex2D.Create(w, h, TexFormat.RGBA32F, { usage = bit.bor(TexUsage.Data, TexUsage.CopyDst) })`):
  32-bit floats then keep full precision on adapters that cannot filter them.

## Waiting

| Call | Waits for |
|---|---|
| `Renderer:sync()` | the render thread has executed everything sent before (not the GPU) |
| `Renderer:gpuFinish()` | the GPU has finished everything recorded so far; finished async reads are delivered |
| `ticket:ready()` | never waits: true once an async read is done |
| `ticket:wait(Renderer)` | blocks until the async read is done (deterministic captures only) |
| frame end | at most 3 frames in flight (`PacingFence`, from the GPU's completion) |

None of these belong in per-frame rendering except the automatic frame pacing.

## Adding things

- **A material parameter:** add the member to the shader's `MaterialParams`
  block, give it a default in the `MaterialType`, write it via
  `material:params()` and `commit()`. Hot reload copies existing members by
  name.
- **A material texture:** declare `uniform sampler2D name;` under `#group 1`,
  list it in the type's `textures` (or `material:setTexture(name, tex)`).
- **A per-object value that changes per frame:** use `drawUser` from the
  type's `perDraw(entity, user)` callback rather than a material parameter.
- **A render pass:** a `RenderPassDesc` with its attachments and load ops,
  `beginPass`, pipeline, inputs, draws, `finish` (see `RenderCoreSystem`).

## Debugging

- `LTHEORY_WGPU_FATAL=1` makes the first wgpu validation error panic (the
  validation scripts set it); otherwise errors are logged.
- `LTHEORY_WGPU_DIAG`, `LTHEORY_WGPU_DUMP` (`command_executor_wgpu/diag.rs`):
  per-pass waits and dumps. `WGPU_BACKEND=vulkan|metal|dx12` picks the API.
- Ctrl+Shift+F3: render info overlay (backend, adapter, frame and GPU times).
- A failed shader compile keeps the last working program and shows the error
  overlay; the log names the shader and the naga error.

## Tests

```sh
cargo test -p phx --lib                                     # unit tests (headless renderer)
cargo test -p phx --lib -- --ignored --test-threads=1      # GPU tests (needs an adapter)
cargo test -p luajit-ffi-gen                                # FFI generator
python tools/render_validation/run_all.py wgpu              # supervisors, captures, compare
python tools/render_validation/hot_reload_probe.py wgpu
python tools/render_validation/auto_exposure_test.py wgpu
```

Captures differ between drivers and platforms: compare a change against
captures of the previous commit built on the same machine
(`tools/render_validation/README.md`).
