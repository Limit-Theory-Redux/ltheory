-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Renderer
Renderer = {}

-- Begin a new frame
function Renderer:beginFrame() end

-- Flush all queued commands to the render thread
function Renderer:flush() end

-- Synchronize with the render thread (wait for all commands to complete)
---@return boolean
function Renderer:sync() end

---@param view number[]
---@param view_size integer
---@param projection number[]
---@param projection_size integer
---@param eyeX number
---@param eyeY number
---@param eyeZ number
function Renderer:beginBatch(view, view_size, projection, projection_size, eyeX, eyeY, eyeZ) end

---@param transform number[]
---@param transform_size integer
---@param boundsCenterX number
---@param boundsCenterY number
---@param boundsCenterZ number
---@param boundsRadius number
---@param meshVao integer
---@param indexCount integer
---@param shaderHandle integer
---@param sortKey integer
function Renderer:addEntity(transform, transform_size, boundsCenterX, boundsCenterY, boundsCenterZ, boundsRadius, meshVao, indexCount, shaderHandle, sortKey) end

function Renderer:flushBatch() end

---@return BatchStats?
function Renderer:getBatchStats() end

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
---@param mode integer
function Renderer:setBlendMode(mode) end

-- Set cull face (0=None, 1=Back, 2=Front)
---@param face integer
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

-- Bind a framebuffer
---@param handle integer
function Renderer:bindFramebuffer(handle) end

-- Bind the default framebuffer
function Renderer:bindDefaultFramebuffer() end

-- Clear color buffer
---@param r number
---@param g number
---@param b number
---@param a number
function Renderer:clearColor(r, g, b, a) end

-- Clear depth buffer
---@param depth number
function Renderer:clearDepth(depth) end

-- Clear both color and depth buffers
---@param r number
---@param g number
---@param b number
---@param a number
---@param depth number
function Renderer:clear(r, g, b, a, depth) end

-- Draw a mesh
---@param vao integer
---@param indexCount integer
function Renderer:drawMesh(vao, indexCount) end

-- Draw a mesh with a specific primitive type
---@param vao integer
---@param indexCount integer
---@param primitive integer
function Renderer:drawMeshPrimitive(vao, indexCount, primitive) end

-- Draw instanced mesh
---@param vao integer
---@param indexCount integer
---@param instanceCount integer
function Renderer:drawMeshInstanced(vao, indexCount, instanceCount) end

-- Signal resize
---@param width integer
---@param height integer
function Renderer:resize(width, height) end

-- Signal swap buffers (frame end)
function Renderer:swapBuffers() end

-- Create the camera UBO on the render thread
function Renderer:createCameraUBO() end

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
function Renderer:updateCameraUBO(mView, mProj, eyeX, eyeY, eyeZ, starDirX, starDirY, starDirZ) end

-- Create the material UBO on the render thread
function Renderer:createMaterialUBO() end

-- Update the material UBO with new material properties
---@param r number
---@param g number
---@param b number
---@param a number
---@param metallic number
---@param roughness number
---@param emission number
function Renderer:updateMaterialUBO(r, g, b, a, metallic, roughness, emission) end

-- Create the light UBO on the render thread
function Renderer:createLightUBO() end

-- Update the light UBO with light properties
---@param posX number
---@param posY number
---@param posZ number
---@param radius number
---@param r number
---@param g number
---@param b number
---@param intensity number
function Renderer:updateLightUBO(posX, posY, posZ, radius, r, g, b, intensity) end

