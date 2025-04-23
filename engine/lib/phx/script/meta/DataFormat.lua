-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class DataFormat
---@field I8 integer 
---@field U8 integer 
---@field I16 integer 
---@field U16 integer 
---@field I32 integer 
---@field U32 integer 
---@field Float integer 
DataFormat = {
    I8 = 0, -- gl::BYTE
    U8 = 1, -- gl::UNSIGNED_BYTE
    I16 = 2, -- gl::SHORT
    U16 = 3, -- gl::UNSIGNED_SHORT
    I32 = 4, -- gl::INT
    U32 = 5, -- gl::UNSIGNED_INT
    Float = 6, -- gl::FLOAT
}

-- Size in bytes of single element
---@param this DataFormat
---@return integer
function DataFormat.GetSize(this) end

