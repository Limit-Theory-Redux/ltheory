-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Renderer
Renderer = {}

-- Synchronize with the render thread (wait for all commands to complete)
---@return boolean
function Renderer:sync() end

---@param view Matrix
---@param projection Matrix
---@param eye Vec3f
function Renderer:beginBatch(view, projection, eye) end

-- Add a cull-only entity to the active batch: bounds + sort key, no
-- mesh/shader to draw. For callers that want frustum culling and sort
-- ordering from `cull_batch` without going through the (unused) batch
-- draw path - see `RenderCoreSystem` in Lua, which still applies its
-- own per-entity material uniforms and issues its own draws.
-- 
-- `radius < 0.0` is a sentinel meaning "never cull" (e.g. no bounds
-- source available for this entity).
---@param boundsCenter Vec3f
---@param boundsRadius number
---@param sortKey integer
---@param userId integer
function Renderer:addCullEntity(boundsCenter, boundsRadius, sortKey, userId) end

-- Frustum-cull and sort the active batch, writing survivors' `user_id`s
-- into `out_indices` in sort-key order. Returns the number written
-- (never more than `out_indices`'s length). Emits no draw commands and
-- does not clear the batch - `flush_batch` still works afterward.
---@param outIndices integer[]
---@param outIndices_size integer
---@return integer
function Renderer:cullBatch(outIndices, outIndices_size) end

---@return BatchStats?
function Renderer:getBatchStats() end

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

-- Set the viewport
---@param x integer
---@param y integer
---@param width integer
---@param height integer
function Renderer:setViewport(x, y, width, height) end

-- Set the scissor region
---@param x integer
---@param y integer
---@param width integer
---@param height integer
function Renderer:setScissor(x, y, width, height) end

-- Enable or disable scissor test
---@param enable boolean
function Renderer:enableScissor(enable) end

-- Set blend mode (0=Disabled, 1=Alpha, 2=Additive, 3=PreMultAlpha)
---@param mode BlendMode
function Renderer:setBlendMode(mode) end

-- Set cull face (0=None, 1=Back, 2=Front)
---@param face CullFace
function Renderer:setCullFace(face) end

-- Enable or disable depth testing
---@param enable boolean
function Renderer:setDepthTest(enable) end

-- Enable or disable depth writing
---@param enable boolean
function Renderer:setDepthWritable(enable) end

-- Set wireframe mode
---@param enable boolean
function Renderer:setWireframe(enable) end

-- Bind a shader program
---@param handle integer
function Renderer:bindShader(handle) end

-- Unbind the current shader
function Renderer:unbindShader() end

-- Set an integer uniform
---@param location integer
---@param value integer
function Renderer:setUniformInt(location, value) end

-- Set a float uniform
---@param location integer
---@param value number
function Renderer:setUniformFloat(location, value) end

-- Set a vec2 uniform
---@param location integer
---@param x number
---@param y number
function Renderer:setUniformFloat2(location, x, y) end

-- Set a vec3 uniform
---@param location integer
---@param x number
---@param y number
---@param z number
function Renderer:setUniformFloat3(location, x, y, z) end

-- Set a vec4 uniform
---@param location integer
---@param x number
---@param y number
---@param z number
---@param w number
function Renderer:setUniformFloat4(location, x, y, z, w) end

-- Bind a 2D texture to a slot
---@param slot integer
---@param handle integer
function Renderer:bindTexture2D(slot, handle) end

-- Bind a 3D texture to a slot
---@param slot integer
---@param handle integer
function Renderer:bindTexture3D(slot, handle) end

-- Bind a cube texture to a slot
---@param slot integer
---@param handle integer
function Renderer:bindTextureCube(slot, handle) end

-- Unbind a texture from a slot
---@param slot integer
function Renderer:unbindTexture(slot) end

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

-- Draw a mesh
---@param vao integer
---@param indexCount integer
function Renderer:drawMesh(vao, indexCount) end

-- Draw a mesh with a specific primitive type
---@param vao integer
---@param indexCount integer
---@param primitive CmdPrimitiveType
function Renderer:drawMeshPrimitive(vao, indexCount, primitive) end

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

-- Create the light UBO on the render thread
function Renderer:createLightUbo() end

-- Update the light UBO with light properties
---@param posX number
---@param posY number
---@param posZ number
---@param radius number
---@param r number
---@param g number
---@param b number
---@param intensity number
function Renderer:updateLightUbo(posX, posY, posZ, radius, r, g, b, intensity) end

