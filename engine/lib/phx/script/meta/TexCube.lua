-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class TexCube
TexCube = {}

---@param r Renderer
---@param size integer
---@param format TexFormat
---@return TexCube
function TexCube.Create(r, size, format) end

---@param r Renderer
---@param path string
---@return TexCube
function TexCube.Load(r, path) end

-- View of the whole cube, for sampling.
---@return TexView
function TexCube:view() end

-- View of one face at mip level 0, usable as a render attachment.
---@param face CubeFace
---@return TexView
function TexCube:faceView(face) end

-- View of one face at the given mip level, usable as a render attachment.
---@param face CubeFace
---@param level integer
---@return TexView
function TexCube:faceMipView(face, level) end

---@param r Renderer
---@param red number
---@param green number
---@param blue number
---@param alpha number
function TexCube:clear(r, red, green, blue, alpha) end

---@param r Renderer
---@param path string
function TexCube:save(r, path) end

---@param r Renderer
---@param path string
---@param level integer
function TexCube:saveLevel(r, path, level) end

---@param r Renderer
---@param face CubeFace
---@param level integer
---@param tf TexFormat
---@param df DataFormat
---@return Bytes
function TexCube:getDataBytes(r, face, level, tf, df) end

---@return TexFormat
function TexCube:getFormat() end

---@return integer
function TexCube:getSize() end

---@param r Renderer
function TexCube:genMipmap(r) end

---@param r Renderer
---@param data Bytes
---@param face CubeFace
---@param level integer
---@param tf TexFormat
---@param df DataFormat
function TexCube:setDataBytes(r, data, face, level, tf, df) end

---@param r Renderer
---@param sampleCount integer
---@return TexCube
function TexCube:genIRMap(r, sampleCount) end

