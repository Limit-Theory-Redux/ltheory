-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class RenderTarget
RenderTarget = {}

---@param r Renderer
---@param sx integer
---@param sy integer
function RenderTarget.Push(r, sx, sy) end

---@param r Renderer
function RenderTarget.Pop(r) end

---@param r Renderer
---@param tex Tex2D
function RenderTarget.BindTex2D(r, tex) end

---@param r Renderer
---@param tex Tex2D
---@param level integer
function RenderTarget.BindTex2DLevel(r, tex, level) end

---@param r Renderer
---@param tex Tex3D
---@param layer integer
function RenderTarget.BindTex3D(r, tex, layer) end

---@param r Renderer
---@param tex Tex3D
---@param layer integer
---@param level integer
function RenderTarget.BindTex3DLevel(r, tex, layer, level) end

---@param r Renderer
---@param tex TexCube
---@param face CubeFace
function RenderTarget.BindTexCube(r, tex, face) end

---@param r Renderer
---@param tex TexCube
---@param face CubeFace
---@param level integer
function RenderTarget.BindTexCubeLevel(r, tex, face, level) end

---@param r Renderer
---@param tex Tex2D
function RenderTarget.PushTex2D(r, tex) end

---@param r Renderer
---@param tex Tex2D
---@param level integer
function RenderTarget.PushTex2DLevel(r, tex, level) end

---@param r Renderer
---@param tex Tex3D
---@param layer integer
function RenderTarget.PushTex3D(r, tex, layer) end

---@param r Renderer
---@param tex Tex3D
---@param layer integer
---@param level integer
function RenderTarget.PushTex3DLevel(r, tex, layer, level) end

