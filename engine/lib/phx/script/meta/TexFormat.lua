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
---@field RGB8 integer 
---@field RGBA8 integer 
---@field RGBA16 integer 
---@field RGBA16F integer 
---@field RGBA32F integer 
---@field Depth16 integer 
---@field Depth24 integer 
---@field Depth32F integer 
TexFormat = {
    R8, -- gl::R8
    R16, -- gl::R16
    R16F, -- gl::R16F
    R32F, -- gl::R32F
    RG8, -- gl::RGB
    RG16, -- gl::RG16
    RG16F, -- gl::RG16F
    RG32F, -- gl::RG32F
    RGB8, -- gl::RGB8
    RGBA8, -- gl::RGBA8
    RGBA16, -- gl::RGBA16
    RGBA16F, -- gl::RGBA16F
    RGBA32F, -- gl::RGBA32F
    Depth16, -- gl::DEPTH_COMPONENT16
    Depth24, -- gl::DEPTH_COMPONENT24
    Depth32F, -- gl::DEPTH_COMPONENT32F
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

