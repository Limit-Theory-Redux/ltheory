-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class SamplerDesc
SamplerDesc = {}

-- Linear filtering, no mips, clamp: the common starting point.
---@return SamplerDesc
function SamplerDesc.Create() end

---@param filter SamplerFilter
function SamplerDesc:min(filter) end

---@param filter SamplerFilter
function SamplerDesc:mag(filter) end

---@param filter MipFilter
function SamplerDesc:mip(filter) end

-- Wrap mode on all three axes.
---@param mode TexWrapMode
function SamplerDesc:wrap(mode) end

---@param s TexWrapMode
---@param t TexWrapMode
---@param r TexWrapMode
function SamplerDesc:wrapAxes(s, t, r) end

---@param max integer
function SamplerDesc:anisotropy(max) end

---@param min integer
---@param max integer
function SamplerDesc:lodRange(min, max) end

-- Depth-comparison sampler (shadow lookups).
---@param func CompareFn
function SamplerDesc:compare(func) end

