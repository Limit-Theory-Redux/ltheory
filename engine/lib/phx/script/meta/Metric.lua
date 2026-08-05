-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Metric
---@field None integer 
---@field DrawCalls integer 
---@field Immediate integer 
---@field PolysDrawn integer 
---@field TrisDrawn integer 
---@field VertsDrawn integer 
---@field Flush integer 
---@field FBOSwap integer 
---@field SIZE integer 
Metric = {
    None = 0,
    DrawCalls = 1,
    Immediate = 2,
    PolysDrawn = 3,
    TrisDrawn = 4,
    VertsDrawn = 5,
    Flush = 6,
    FBOSwap = 7,
    SIZE = 8,
}

---@param this Metric
---@return integer
function Metric.Get(this) end

---@param this Metric
---@return string?
function Metric.GetName(this) end

