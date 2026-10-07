-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Renderer
Renderer = {}

-- Synchronize with the render thread (wait for all commands to complete)
---@return boolean
function Renderer:sync() end

-- Block until the GPU has finished everything submitted so far
-- (`glFinish`), e.g. to time a piece of GPU work.
function Renderer:gpuFinish() end

-- Draw calls of the last frame (mesh + immediate + instanced).
---@return integer
function Renderer:statsDrawCalls() end

-- Render-thread execute time of the last frame, in microseconds.
---@return integer
function Renderer:statsFrameTimeUs() end

-- Time the render thread sat blocked waiting for commands in the last
-- frame (producer starvation), microseconds.
---@return integer
function Renderer:statsRecvWaitUs() end

-- Time the render thread spent blocked in the buffer swap (vsync/GPU
-- back-pressure) in the last frame, microseconds.
---@return integer
function Renderer:statsPresentWaitUs() end

-- Frames the render thread has completed (to de-duplicate stats samples).
---@return integer
function Renderer:statsFrameCount() end

-- Commands the render thread processed in the last frame.
---@return integer
function Renderer:statsCommands() end

-- Time the main thread spent blocked in the last frame end, microseconds.
---@return integer
function Renderer:statsMainWaitUs() end

---@return integer
function Renderer:statsVertices() end

-- Render passes begun in the last frame.
---@return integer
function Renderer:statsPasses() end

-- Pipeline (program) switches in the last frame: binds that changed the program.
---@return integer
function Renderer:statsPipelineSwitches() end

-- Bind groups bound in the last frame.
---@return integer
function Renderer:statsBindGroupSwitches() end

-- Immediate-mode (UI) vertices in the last frame.
---@return integer
function Renderer:statsImmVertices() end

-- Resource census: pipelines, samplers, bind groups, textures, meshes
-- (`u64::MAX` = n/a on this backend).
---@return integer
function Renderer:statsPipelines() end

---@return integer
function Renderer:statsSamplers() end

---@return integer
function Renderer:statsBindGroups() end

---@return integer
function Renderer:statsTextures() end

---@return integer
function Renderer:statsMeshes() end

-- Approximate GPU memory of all live textures, bytes (`u64::MAX` = n/a).
---@return integer
function Renderer:statsTextureBytes() end

-- The backend times render passes (and `LTHEORY_GPU_TIMING` is not 0).
---@return boolean
function Renderer:statsGpuAvailable() end

-- Frames of GPU timing measured so far (changes when a new one arrives).
---@return integer
function Renderer:statsGpuFrames() end

-- GPU time of the last measured frame, first pass start to last pass
-- end, microseconds.
---@return integer
function Renderer:statsGpuTotalUs() end

-- Exponential average of `stats_gpu_total_us`.
---@return integer
function Renderer:statsGpuTotalSmoothUs() end

-- Sum of the GPU time of all passes of the last measured frame (no gaps).
---@return integer
function Renderer:statsGpuBusyUs() end

-- Number of passes in the per-pass list, heaviest first (at most 32).
---@return integer
function Renderer:statsGpuPassCount() end

-- Label of the `i`-th heaviest pass (0-based), empty if out of range.
---@param i integer
---@return string
function Renderer:statsGpuPassLabel(i) end

-- GPU time of the `i`-th heaviest pass in the last measured frame, microseconds.
---@param i integer
---@return integer
function Renderer:statsGpuPassUs(i) end

-- Exponential average of the GPU time of the `i`-th heaviest pass, microseconds.
---@param i integer
---@return integer
function Renderer:statsGpuPassSmoothUs(i) end

-- The `n` heaviest passes (smoothed ms) as `label=ms` pairs separated by `,`
-- (for logs and capture output); empty when n/a.
---@param n integer
---@return string
function Renderer:statsGpuSummary(n) end

-- Uniform ring bytes allocated in the last completed frame.
---@return integer
function Renderer:statsUniformBytes() end

-- Vertex ring bytes allocated in the last completed frame.
---@return integer
function Renderer:statsVertexBytes() end

-- Startup backend description as `key=value` lines: `backend`
-- (`OpenGL 3.3` or `wgpu`), then GL strings or the wgpu adapter info.
-- Empty until the render thread is up; query once and cache.
---@return string
function Renderer:backendInfo() end

-- Read the `w` x `h` texels at `x`, `y` of `view` (its mip level, face or
-- layer) as `fmt` and wait for them: rows from the first up, tightly
-- packed, in the layout of `fmt` (the texture is converted if it is
-- stored differently). **Stalls until the GPU has produced the data**:
-- for screenshots, tests and tools only, never in a frame. Empty `Bytes`
-- if the read failed.
---@param view TexView
---@param x integer
---@param y integer
---@param w integer
---@param h integer
---@param fmt TexFormat
---@return Bytes
function Renderer:readSync(view, x, y, w, h, fmt) end

-- Start reading the `w` x `h` texels at `x`, `y` of `view` as `fmt`
-- without waiting. Poll the ticket (`:ready()`) once per frame; the data
-- arrives two or three frames later. See `ReadbackTicket`.
---@param view TexView
---@param x integer
---@param y integer
---@param w integer
---@param h integer
---@param fmt TexFormat
---@return ReadbackTicket
function Renderer:readAsync(view, x, y, w, h, fmt) end

-- Begin a render pass on `desc`'s attachments. Only one pass may be open
-- at a time; end it with `RenderPass:finish()`.
---@param desc RenderPassDesc
---@return RenderPass
function Renderer:beginPass(desc) end

-- The open pass, for code that records into it without owning it (for
-- example UI widgets calling `pass:setUiTransform`). It cannot `finish`
-- the pass. Errors if no pass is open.
---@return RenderPass
function Renderer:currentPass() end

-- Signal resize
---@param width integer
---@param height integer
function Renderer:resize(width, height) end

-- Signal swap buffers (frame end)
function Renderer:swapBuffers() end

-- Set the camera of the passes that begin from now on (and of the open
-- pass): view and projection matrices and the direction towards the
-- primary light. Rendering is camera-relative, so the eye is the origin.
-- Replaces the old shader-variable stack and the camera UBO update.
---@param view Matrix
---@param proj Matrix
---@param starDir Vec3f
function Renderer:setCamera(view, proj, starDir) end

-- Set the environment cube maps (`envMap` and `irMap` of group 0) of the
-- passes that begin from now on (and of the open pass). Replaces
-- the old per-shader `envMap`/`irMap` variables.
---@param envMap TexCube
---@param irMap TexCube
function Renderer:setEnvironment(envMap, irMap) end

-- Create a bind group from `desc`; bind it in a pass with
-- `pass:setBindGroup(group, id)`.
---@param desc BindGroupDesc
---@return integer
function Renderer:createBindGroup(desc) end

