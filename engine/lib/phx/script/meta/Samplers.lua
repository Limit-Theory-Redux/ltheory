-- AUTO GENERATED. DO NOT MODIFY!
---@meta

-- Sampler presets. The discriminant is the `SamplerId`, so
-- `Samplers.LinearClamp` can be passed wherever a sampler is expected.
---@class Samplers
---@field Point integer 
---@field PointRepeat integer 
---@field LinearClamp integer 
---@field LinearRepeat integer 
---@field LinearMipClamp integer 
---@field LinearMipRepeat integer 
---@field LinearMipRepeatAniso integer `LinearMipRepeat` with 16x anisotropic filtering: the state the old `Texture` class gave every material texture.
Samplers = {
    Point = 0,
    PointRepeat = 1,
    LinearClamp = 2,
    LinearRepeat = 3,
    LinearMipClamp = 4,
    LinearMipRepeat = 5,
    -- `LinearMipRepeat` with 16x anisotropic filtering: the state the old
    -- `Texture` class gave every material texture.
    LinearMipRepeatAniso = 6,
}

