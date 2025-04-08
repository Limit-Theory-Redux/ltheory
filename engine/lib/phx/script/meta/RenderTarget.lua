-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class RenderTarget
RenderTarget = {}

---@param sx integer
---@param sy integer
function RenderTarget.Push(sx, sy) end

function RenderTarget.Pop() end

---@param tex Tex2D
function RenderTarget.BindTex2D(tex) end

---@param tex Tex2D
---@param level integer
function RenderTarget.BindTex2DLevel(tex, level) end

---@param tex Tex3D
---@param layer integer
function RenderTarget.BindTex3D(tex, layer) end

---@param tex Tex3D
---@param layer integer
---@param level integer
function RenderTarget.BindTex3DLevel(tex, layer, level) end

---@param tex TexCube
---@param face CubeFace
function RenderTarget.BindTexCube(tex, face) end

---@param tex TexCube
---@param face CubeFace
---@param level integer
function RenderTarget.BindTexCubeLevel(tex, face, level) end

---@param tex Tex2D
function RenderTarget.PushTex2D(tex) end

---@param tex Tex2D
---@param level integer
function RenderTarget.PushTex2DLevel(tex, level) end

---@param tex Tex3D
---@param layer integer
function RenderTarget.PushTex3D(tex, layer) end

---@param tex Tex3D
---@param layer integer
---@param level integer
function RenderTarget.PushTex3DLevel(tex, layer, level) end

