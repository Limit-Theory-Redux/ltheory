# Multithreaded Rendering System

The multithreaded rendering system separates OpenGL operations from game logic by running GPU commands on a dedicated render thread. This allows the main thread to continue game logic while rendering happens in parallel.

## Architecture

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Main Thread   │     │  Worker Threads  │     │  Render Thread  │
│   (Lua, Game)   │     │  (Culling, etc)  │     │  (GL Context)   │
└────────┬────────┘     └────────┬─────────┘     └────────▲────────┘
         │                       │                        │
         └───────────────────────┴────────────────────────┘
                    Triple-Buffered Command Channel
```

## Key Components

### Rust Files

| File | Purpose |
|------|---------|
| `render_thread.rs` | Dedicated GL thread, command execution |
| `render_command.rs` | RenderCommand enum (50+ command types) |
| `render_queue.rs` | Command batching, Lua FFI bindings |
| `frame_ring.rs` | Triple-buffered frame pipelining |
| `render_mode.rs` | Dual-mode switching (direct GL vs command) |
| `render_worker.rs` | Worker pool for parallel culling |

### RenderCommand Categories

**State Management:**
- `SetViewport`, `SetScissor`, `EnableScissor`
- `SetBlendMode`, `SetCullFace`, `SetDepthTest`
- `SetWireframe`, `SetLineWidth`, `SetPointSize`

**Shader Operations:**
- `BindShader`, `BindShaderByResource`, `UnbindShader`
- `SetUniformInt/Float/Float2/Float3/Float4/Mat4` (by location)
- `SetUniform*ByName` variants (for command mode)

**Drawing:**
- `DrawMesh`, `DrawMeshInstanced`, `DrawMeshByResource`
- `DrawInstancedWithData`, `DrawImmediate`

**Resources:**
- `CreateShader`, `ReloadShader`, `CreateTexture2D`, `CreateMesh`
- `DestroyResource`

**UBOs:**
- `CreateCameraUBO`, `UpdateCameraUBO`
- `CreateMaterialUBO`, `UpdateMaterialUBO`
- `CreateLightUBO`, `UpdateLightUBO`

**Synchronization:**
- `SwapBuffers`, `Fence`, `Flush`, `Shutdown`

## Dual-Mode Operation

All rendering code works identically in both modes. The `is_command_mode()` function determines routing:

```rust
// In Mesh.draw(), Shader.start(), etc:
if is_command_mode() {
    submit_command(RenderCommand::DrawMesh { ... })
} else {
    unsafe { gl::DrawElements(...) }
}
```

## Triple Buffering

Three frames can be in-flight simultaneously:

```
Frame N:   [Main: Submit] [Workers: Prepare] [Render: Execute]
Frame N+1:                [Main: Submit] [Workers: Prepare] [Render: Execute]
Frame N+2:                               [Main: Submit] [Workers: Prepare]
```

- **FRAME_RING_SIZE = 3** (configurable)
- Lock-free producer/consumer with atomic indices
- Main thread blocks only if 3 frames already queued

## Lua API

### Engine Control

```lua
-- Start/stop render thread
Engine:startRenderThread() -> bool
Engine:stopRenderThread()
Engine:isRenderThreadActive() -> bool

-- Statistics
Engine:getRenderThreadCommands() -> number
Engine:getRenderThreadDrawCalls() -> number
Engine:getRenderThreadStateChanges() -> number
Engine:getRenderThreadFrameCount() -> number
Engine:getRenderThreadLastFrameTimeUs() -> number
```

### RenderQueue (Global)

```lua
-- State
RenderQueue:setViewport(x, y, width, height)
RenderQueue:setBlendMode(mode)  -- 0=Disabled, 1=Alpha, 2=Additive
RenderQueue:setCullFace(face)   -- 0=None, 1=Back, 2=Front
RenderQueue:setDepthTest(enable)

-- Drawing
RenderQueue:drawMesh(vao, indexCount)
RenderQueue:drawMeshInstanced(vao, indexCount, instanceCount)

-- Synchronization
RenderQueue:beginFrame()
RenderQueue:flush()
RenderQueue:sync()
RenderQueue:swapBuffers()

-- UBOs
RenderQueue:updateCameraUBO(view, proj, eye, starDir)
RenderQueue:updateLightUBO(pos, radius, r, g, b, intensity)
```

## Usage Example

```lua
function onInput()
    -- Toggle with 'R' key
    if Input:keyboard():isPressed(Button.KeyboardR) then
        if Engine:isRenderThreadActive() then
            Engine:stopRenderThread()
        else
            Engine:startRenderThread()
        end
    end
end

function onRender()
    -- Code works identically in both modes
    RenderCoreSystem:render(data)
    Draw:flush()
end
```

## Optimizations

1. **Texture Binding Cache**: Skips redundant `glBindTexture` calls
2. **Uniform Location Cache**: O(1) lookups for uniform names
3. **Instance VBO Reuse**: 50% headroom allocation, resizes only when needed
4. **Buffer Orphaning**: Uses `GL_STREAM_DRAW` for per-frame data

## Performance Characteristics

| Metric | Value |
|--------|-------|
| Command capacity | 8,192 per frame |
| Frame latency | Up to 3 frames |
| Fence buffer | 64 pending signals |
| Worker count | CPU cores - 2 (min 1) |

## Resource ID System

GPU resources created in command mode get unique `ResourceId`s:

```rust
CreateShader { id: ResourceId, vertex_src, fragment_src }
BindShaderByResource { id: ResourceId }
```

This enables deferred resource creation - main thread submits with ID, render thread stores with that ID for later binding.
