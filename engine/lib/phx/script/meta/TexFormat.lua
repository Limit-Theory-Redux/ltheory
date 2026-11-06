-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class TexFormat
---@field R8 integer
---@field R16 integer
---@field R16F integer
---@field R32F integer
---@field RG8 integer
---@field RG16 integer
---@field RG16F integer
---@field RG32F integer
---@field RGBA8 integer
---@field RGBA16 integer
---@field RGBA16F integer
---@field RGBA32F integer
---@field Depth16 integer
---@field Depth24 integer
---@field Depth32F integer
TexFormat = {
    R8 = 0,
    R16 = 1,
    R16F = 2,
    R32F = 3,
    RG8 = 4,
    RG16 = 5,
    RG16F = 6,
    RG32F = 7,
    RGBA8 = 8,
    RGBA16 = 9,
    RGBA16F = 10,
    RGBA32F = 11,
    Depth16 = 12,
    Depth24 = 13,
    Depth32F = 14,
}

---@param this TexFormat
---@return integer
function TexFormat.Components(this) end

---@param this TexFormat
---@return integer
function TexFormat.GetSize(this) end

---@param this TexFormat
---@return boolean
function TexFormat.IsColor(this) end

---@param this TexFormat
---@return boolean
function TexFormat.IsDepth(this) end
