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
    R8 = 0, -- gl::R8
    R16 = 1, -- gl::R16
    R16F = 2, -- gl::R16F
    R32F = 3, -- gl::R32F
    RG8 = 4, -- gl::RG8
    RG16 = 5, -- gl::RG16
    RG16F = 6, -- gl::RG16F
    RG32F = 7, -- gl::RG32F
    RGBA8 = 8, -- gl::RGBA8
    RGBA16 = 9, -- gl::RGBA16
    RGBA16F = 10, -- gl::RGBA16F
    RGBA32F = 11, -- gl::RGBA32F
    Depth16 = 12, -- gl::DEPTH_COMPONENT16
    Depth24 = 13, -- gl::DEPTH_COMPONENT24
    Depth32F = 14, -- gl::DEPTH_COMPONENT32F
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

