-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class TexFormat
---@field Undefined integer 
---@field R8 integer 
---@field R16 integer 
---@field R16F integer 
---@field R32F integer 
---@field RG8 integer 
---@field RG16 integer 
---@field RG16F integer 
---@field RG32F integer 
---@field RGB8 integer 
---@field RGBA8 integer 
---@field RGBA16 integer 
---@field RGBA16F integer 
---@field RGBA32F integer 
---@field Depth16 integer 
---@field Depth24 integer 
---@field Depth32F integer 
TexFormat = {
    Undefined = 0,
    R8 = 33321,
    R16 = 33322,
    R16F = 33325,
    R32F = 33326,
    RG8 = 6407,
    RG16 = 33324,
    RG16F = 33327,
    RG32F = 33328,
    RGB8 = 32849,
    RGBA8 = 32856,
    RGBA16 = 32859,
    RGBA16F = 34842,
    RGBA32F = 34836,
    Depth16 = 33189,
    Depth24 = 33190,
    Depth32F = 36012,
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

---@param this TexFormat
---@return boolean
function TexFormat.IsValid(this) end

