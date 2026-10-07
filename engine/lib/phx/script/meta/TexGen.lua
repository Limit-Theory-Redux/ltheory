-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class TexGen
TexGen = {}

-- A new cube map of `size` and `format` with `mips` levels (0 = the full
-- chain, the caller then generates it), generated with `desc`.
---@param r Renderer
---@param desc GenDesc
---@param size integer
---@param format TexFormat
---@param mips integer
---@return TexCube
function TexGen.Cube(r, desc, size, format, mips) end

-- Regenerate an existing cube map (ping-pong generation).
---@param r Renderer
---@param desc GenDesc
---@param cube TexCube
function TexGen.CubeInto(r, desc, cube) end

-- A new `size`^3 volume of `format`, generated with `desc`.
---@param r Renderer
---@param desc GenDesc
---@param size integer
---@param format TexFormat
---@return Tex3D
function TexGen.Volume(r, desc, size, format) end

