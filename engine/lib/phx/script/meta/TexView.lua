-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class TexView
TexView = {}

---@return integer
function TexView:getWidth() end

---@return integer
function TexView:getHeight() end

---@return integer
function TexView:getBaseMip() end

-- 0 = all remaining levels.
---@return integer
function TexView:getMipCount() end

-- The same texture restricted to `mipCount` levels from `baseMip`
-- (`mipCount` 0 = all remaining), for sampling. The extent follows the
-- base level.
---@param baseMip integer
---@param mipCount integer
---@return TexView
function TexView:mips(baseMip, mipCount) end

