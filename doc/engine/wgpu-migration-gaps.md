# wgpu Migration — Modernization List

Tracked during the feat/wgpu port (stages 1-5). Items are things found while
implementing that are either (a) known gaps in the wgpu backend that should be
fixed for full GL parity, or (b) places where the engine's legacy GL-era design
could be modernized now that wgpu is in the picture. Appended as found.

## Functional gaps (wgpu backend vs GL — "should fix")

1. **Texture readbacks are graceful no-ops** (`cmd_read_texture_*`): the GL
   executor returns real pixels; wgpu replies an empty Vec. Needs COPY_SRC
   usage on textures + a staging buffer + `device.poll`/`queue.on_submitted_work_done`
   before mapping. Engine features relying on readback (screenshots, CPU
   texture inspection) return empty data on the wgpu path.

2. **Hot-reload pipelines are stale**: `ReloadShader` compiles a fresh module
   pair into `hot_reloaded_shaders`, but the pipeline cache key is the shader
   RESOURCE id — the first pipeline created for a shader keeps the OLD modules
   forever. A hot reload changes nothing on screen until the process restarts.
   Fix: include the hot pair identity (e.g. a reload counter / key hash) in
   `PipelineKey`, or clear the affected pipeline entries on reload.

3. **Bind-group cache never invalidated on texture destruction**:
   `bind_group_cache` keys on (shader id, sampler-slot hash) — when a texture
   is destroyed and a new one takes the same slot, the cached group still
   references the dead view (wgpu keeps the resource alive via the group, so
   no crash — but the new texture never appears until the slot hash changes).
   Fix: invalidate on `DestroyResources` / `BindTexture` with a generation
   counter per slot.

4. **Mipmap generation is a no-op** (`cmd_generate_mipmap_2d`): textures are
   created with `mip_level_count: 1`; the GL path generates full mip chains.
   Linear filtering without mips degrades (aliasing in minification). Fix: a
   blit-based mip chain (render pass per level) or `mip_level_count` +
   COPY_DST levels uploaded from CPU-side generation.

5. **Texture wrap mode is ignored** (`cmd_set_texture_2d_wrap_mode`): the
   sampler is rebuilt on filter changes but always uses the default Clamp
   wrap. GL's Repeat/MirroredRepeat/ClampToEdge semantics are lost. Fix: track
   wrap state per texture and apply in `recreate_sampler`.

6. **Anisotropy is ignored** (`cmd_set_texture_2d_anisotropy`): no-op.
   `wgpu::SamplerDescriptor::max_anisotropy` exists — needs device limits
   (MAX_SAMPLER_ANISOTROPY) + re-create sampler on change.

7. **3D textures are placeholders** (`cmd_create_texture_3d` creates a 2D
   texture): any shader sampling a 3D texture will fail pipeline creation.
   Fix: real 3D textures + `TextureViewDimension::D3` in the bind-group
   layout (per-sampler dimension from reflection).

8. **1D textures are 1xN 2D textures** (`cmd_create_texture_1d`): sampling
   works only because naga GLSL maps sampler1D to texture_1d — the layout
   entry declares D2, so 1D samplers fail validation. Fix: D1 views +
   per-sampler view dimension from reflection.

9. **Cube framebuffer attachments ignore the face**:
   `cmd_framebuffer_attach_texture_cube` attaches the whole-cube view; GL
   attaches one face. Rendering into a cube face (shadow maps, environment
   updates) writes all faces. Fix: per-face views via
   `TextureViewDescriptor { base_array_layer, array_layer_count }`.

10. **Fences reply immediately** (`cmd_fence`/`cmd_pacing_fence`): the pacing
    protocol counts in-flight frames but wgpu never paces the GPU queue — a
    slow GPU can accumulate unbounded queued work. Fix: reply after
    `queue.on_submitted_work_done` for real pacing.

11. **Line width / point size are dropped**: wgpu renders 1px lines and
    point sprites; GL honored `glLineWidth`/`glPointSize`. `line_width` is
    tracked but unused. Feature loss, documented.

12. **Unbounded CPU upload churn**: camera/material/light UBOs are written to
    the GPU on EVERY draw (no per-frame dirty check); instance/immediate
    scratch buffers are grow-only single buffers. A ring of per-frame buffers
    (or dynamic offsets) is the modern pattern.

13. **One command encoder per draw**: each draw submits its own encoder +
    render pass (LOAD). Batching all draws for a framebuffer into one pass
    would cut submission overhead (the GL path's draw-call stats exist
    precisely because batching matters there).

14. **`instance`/`adapter` fields on WgpuCommandExecutor are dead**: nothing
    reads them. Remove or use for device-info logging.

15. **No device error callback**: wgpu validation failures currently land in
    the engine's panic hook (the run died on the first bad `write_texture`).
    `device.set_uncaptured_error_callback` (or `on_uncaptured_error`) +
    logging would surface errors without killing the app.

16. **Unbound sampler slots sample a 1x1 white texture**: GL would sample
    whatever was last bound. The white fallback is deterministic (good), but
    a missing `BindTexture` for a shader's sampler is invisible — a warn-once
    per (shader, sampler) would aid debugging.

17. **Stats dashboard category timing is a no-op on wgpu**
    (`set_category_timing` accepted for interface parity): the dashboard's
    per-category breakdown shows nothing for wgpu runs.

18. **`cmd_set_texture_2d_mip_range` no-op**: GL restricts the sampled mip
    range; wgpu sampler base/max mip-level fields exist
    (`lod_min_clamp`/`lod_max_clamp`) — wire them.

## Engine-side modernization opportunities (found during the port)

19. **winit 0.30 `Window` is not Clone** (owned, Drop-closes): the wgpu
    surface must be created from raw handles (`SurfaceTargetUnsafe::RawHandle`)
    with a documented lifetime contract instead of the clean owned-target
    form. If winit ever re-adds Arc-windows, switch to
    `SurfaceTarget::from_window_without_display` and delete the unsafe block.

20. **Texture uploads still speak GL enums**: `UpdateTexture2DData` carries
    `internal_format`/`pixel_format`/`data_format` as GL constants; the wgpu
    executor re-maps them. A format-agnostic command payload (TexFormat +
    bpp) would serve both backends without the GL enum layer.

21. **Render-thread executor branch**: `RenderThread` now holds both the GL
    `CommandExecutor` and an `Option<WgpuCommandExecutor>` with a runtime
    branch in the loop. When the GL path is feature-flagged out (cleanup
    stage), the branch collapses to a single executor type.

22. **The 57-variant command surface is a monolith**: RenderCommand + the
    executor dispatch grew organically (UBO bytes, GL enums, reply channels
    mixed in). The wgpu backend re-implements it faithfully; a future
    refactor could split transport (channel/reply) from semantics.

23. **`GetUniformLocationByResource` protocol is location-based** (GL
    legacy): the wgpu backend serves indices into a reflection table. The
    engine's Lua-facing `Shader:SetFloat(name, ...)` resolves through cached
    locations — a name-based command path would remove the -1/1000+ index
    games entirely.

24. **Benchmark scene caps**: `BENCH_BELT_COUNT = 100000` instanced asteroids
    is the parity stress test; the wgpu instance buffer upload (84 B/instance
    = 8.4 MB/frame) is the bottleneck candidate — ring-buffer + reuse (item
    12) is the lever.

25. **`present_mode` mapping is done twice** (engine `PresentMode` →
    wgpu): once in `present_mode.rs` (`into()`), once implicitly via
    `get_default_config`. Single source of truth would be cleaner.

26. **naga GLSL version adaptation (330→440) is a documented workaround**:
    naga 30 accepts only 440/450/460. If the engine ever bumps its GLSL
    baseline to 440 on the GL side too, the adaptation layer shrinks.
