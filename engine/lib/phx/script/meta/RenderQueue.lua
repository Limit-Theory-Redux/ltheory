-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class RenderQueue
RenderQueue = {}

-- Begin a new frame
function RenderQueue:beginFrame() end

-- Flush all queued commands
function RenderQueue:flush() end

-- Wait for render thread to complete all commands
---@return boolean
function RenderQueue:sync() end

-- Set the viewport
---@param x integer
---@param y integer
---@param width integer
---@param height integer
function RenderQueue:setViewport(x, y, width, height) end

-- Set the scissor region
---@param x integer
---@param y integer
---@param width integer
---@param height integer
function RenderQueue:setScissor(x, y, width, height) end

-- Enable or disable scissor test
---@param enable boolean
function RenderQueue:enableScissor(enable) end

-- Set blend mode (0=Disabled, 1=Alpha, 2=Additive, 3=PreMultAlpha)
---@param mode integer
function RenderQueue:setBlendMode(mode) end

-- Set cull face (0=None, 1=Back, 2=Front)
---@param face integer
function RenderQueue:setCullFace(face) end

-- Enable or disable depth testing
---@param enable boolean
function RenderQueue:setDepthTest(enable) end

-- Enable or disable depth writing
---@param enable boolean
function RenderQueue:setDepthWritable(enable) end

-- Set wireframe mode
---@param enable boolean
function RenderQueue:setWireframe(enable) end

-- Bind a shader program
---@param handle integer
function RenderQueue:bindShader(handle) end

-- Unbind the current shader
function RenderQueue:unbindShader() end

-- Set an integer uniform
---@param location integer
---@param value integer
function RenderQueue:setUniformInt(location, value) end

-- Set a float uniform
---@param location integer
---@param value number
function RenderQueue:setUniformFloat(location, value) end

-- Set a vec2 uniform
---@param location integer
---@param x number
---@param y number
function RenderQueue:setUniformFloat2(location, x, y) end

-- Set a vec3 uniform
---@param location integer
---@param x number
---@param y number
---@param z number
function RenderQueue:setUniformFloat3(location, x, y, z) end

-- Set a vec4 uniform
---@param location integer
---@param x number
---@param y number
---@param z number
---@param w number
function RenderQueue:setUniformFloat4(location, x, y, z, w) end

-- Bind a 2D texture to a slot
---@param slot integer
---@param handle integer
function RenderQueue:bindTexture2D(slot, handle) end

-- Bind a 3D texture to a slot
---@param slot integer
---@param handle integer
function RenderQueue:bindTexture3D(slot, handle) end

-- Bind a cube texture to a slot
---@param slot integer
---@param handle integer
function RenderQueue:bindTextureCube(slot, handle) end

-- Unbind a texture from a slot
---@param slot integer
function RenderQueue:unbindTexture(slot) end

-- Bind a framebuffer
---@param handle integer
function RenderQueue:bindFramebuffer(handle) end

-- Bind the default framebuffer
function RenderQueue:bindDefaultFramebuffer() end

-- Clear color buffer
---@param r number
---@param g number
---@param b number
---@param a number
function RenderQueue:clearColor(r, g, b, a) end

-- Clear depth buffer
---@param depth number
function RenderQueue:clearDepth(depth) end

-- Clear both color and depth buffers
---@param r number
---@param g number
---@param b number
---@param a number
---@param depth number
function RenderQueue:clear(r, g, b, a, depth) end

-- Draw a mesh
---@param vao integer
---@param indexCount integer
function RenderQueue:drawMesh(vao, indexCount) end

-- Draw a mesh with a specific primitive type
---@param vao integer
---@param indexCount integer
---@param primitive integer
function RenderQueue:drawMeshPrimitive(vao, indexCount, primitive) end

-- Draw instanced mesh
---@param vao integer
---@param indexCount integer
---@param instanceCount integer
function RenderQueue:drawMeshInstanced(vao, indexCount, instanceCount) end

-- Signal resize
---@param width integer
---@param height integer
function RenderQueue:resize(width, height) end

-- Signal swap buffers (frame end)
function RenderQueue:swapBuffers() end

-- Create the camera UBO on the render thread
function RenderQueue:createCameraUBO() end

-- Update the camera UBO with new camera data
-- Parameters are the matrices and vectors that make up the camera state.
---@param mView Matrix
---@param mProj Matrix
---@param eyeX number
---@param eyeY number
---@param eyeZ number
---@param starDirX number
---@param starDirY number
---@param starDirZ number
function RenderQueue:updateCameraUBO(mView, mProj, eyeX, eyeY, eyeZ, starDirX, starDirY, starDirZ) end

-- Create the material UBO on the render thread
function RenderQueue:createMaterialUBO() end

-- Update the material UBO with new material properties
---@param r number
---@param g number
---@param b number
---@param a number
---@param metallic number
---@param roughness number
---@param emission number
function RenderQueue:updateMaterialUBO(r, g, b, a, metallic, roughness, emission) end

-- Create the light UBO on the render thread
function RenderQueue:createLightUBO() end

-- Update the light UBO with light properties
---@param posX number
---@param posY number
---@param posZ number
---@param radius number
---@param r number
---@param g number
---@param b number
---@param intensity number
function RenderQueue:updateLightUBO(posX, posY, posZ, radius, r, g, b, intensity) end

