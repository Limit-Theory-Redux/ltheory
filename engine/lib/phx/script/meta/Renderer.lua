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

