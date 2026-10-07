# Scene List and GPU Instancing

How the scene passes get their draws, and how many copies of one mesh are drawn at once. It sits on
top of the [render thread](render-thread.md) and implements steps S3 and S4 of the
[render API v2](render-api-v2.md) (sections 3b and 1.2).

The old `RenderBatch` (a cull + sort service for `RenderCoreSystem`), its draw-emitting half and
`InstanceBatch` are gone. `SceneList` replaces all of them.

## SceneList

`render/gpu/scene_list.rs`, Lua face `script/ffi_ext/SceneList.lua`. `RenderCoreSystem` owns one:

```lua
local scene = SceneList.Create()

-- once per frame (buildPassLists)
scene:reset()
for each mesh entity do
    local t = scene:addTransform()           -- SceneTransform* written in place
    t.cx, t.cy, t.cz, t.scale = ...          -- cull sphere centre (camera-relative) and the body's scale
    rb:getToWorldMatrixInto(eye, t.world)    -- straight into the list
    for each mesh do scene:addItem(t.index, mesh, material, entity) end
end

-- once per scene pass (renderInOrder)
scene:submit(pass, BlendMode.Disabled, frustumCulling)
```

`addItem` puts the item in the bucket of its material's blend mode (opaque, additive, alpha) and
commits the material if it changed, so it must not run inside an open pass. `scale < 0` is the "never
cull" sentinel for entities without a rigid body.

`submit` for a bucket does, in this order:

1. **Cull** (unless disabled): sphere against the frustum of the camera set by `Renderer:setCamera`.
   The sphere is the mesh's local-origin bound times the transform's scale, centred on the body.
2. **Sort** the survivors by `(pipeline, material, mesh)`, stably. The alpha bucket keeps insertion order,
   because sorting blended draws would change which one wins at equal depth, that is, pixels.
3. **Per-draw callbacks**: `MaterialType.perDraw(entity, user)` runs for each surviving item whose
   material has one, so culled entities never have per-draw values computed. `user` is a `Vec4f[7]`, the
   item's `drawUser`. The callback is called by the Lua wrapper between Rust's prepare and emit steps
   (`SceneList_Prepare`, `SceneList_Emit`), so no C-to-Lua call exists.
4. **Emit** into the open pass: `SetPipeline` when the pipeline changes, `SetBindGroup(1, ...)` when the
   material changes, then per draw one `SetDraw` (a 256-byte `DrawBlock` written into the uniform ring)
   and one `DrawMesh`. `mWorldIT` is computed once per transform, for survivors only.

Cull statistics (`getSubmitted`, `getVisible`, `getCulled`) accumulate until `reset`.

## Materials

A `Material` (`render/gpu/material.rs`) owns a slice of a parameter arena, the group-1 bind group and
the pipeline template of one set of material parameters; Lua wraps it with the typed parameter struct.
`MaterialType` (`Shared/Types/MaterialType.lua`, defined in `Shared/Definitions/MaterialDefs.lua`) is
the shared part: shader, state, `MaterialParams` defaults, default textures and the `perDraw` callback.
See render-api-v2.md 3a and the "S4 notes" there.

## Instancing

Both instanced draws are pass commands whose data goes through the **vertex ring**, so nothing is
copied per call into a command and chunk memory is recycled like the uniform ring's:

- `pass:drawMeshInstanced(mesh, instances, count)`: `instances` is an `InstanceData[?]` array (model
  matrix, color, scale; 84 bytes each) read as divisor-1 vertex attributes 4..9 (`include/instanced.glsl`,
  `vertex/wvp_instanced.glsl`).
- `pass:drawInstancedIndices(mesh, indices, count)`: `indices` is a `uint32_t[?]` array, 4 bytes per
  instance, read as the divisor-1 attribute at location 10. The vertex shader pulls the transform from a
  static data texture with `texelFetch` (`vertex/wvp_instanced_tex.glsl`, used by `AsteroidBeltRenderer`
  for 100k+ asteroids on GL 3.3). The data texture and the per-draw block (`InstanceParams`) are group 2.
  This maps one to one onto a storage buffer under wgpu.

The ring itself: `render/gpu/uniform_ring.rs` (`StagingRing`, `UniformRing`, `VertexRing`). The main
thread allocates into fixed-capacity staging chunks, a chunk that fills up moves to the executor whole,
and the executor hands the memory back after uploading it (threaded: a return channel, immediate:
inline). GL buffers of a frame slot are reused behind a fence: `SwapBuffers` inserts `glFenceSync`
after the frame's last pass and `BeginFrame{slot}` waits (`glClientWaitSync`) before the slot is reused.
