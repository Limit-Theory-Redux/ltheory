-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class MemoryReport
MemoryReport = {}

-- Clear the report store for a new sampling round. Called once per
-- sampling interval from Lua (NOT per render frame).
function MemoryReport.BeginFrame() end

-- Add one memory category row. Bytes are the Lua-side estimate for
-- the category (e.g. `#asteroids * sizeof(struct)` for SoA buffers).
---@param category string
---@param count integer
---@param bytes integer
function MemoryReport.Add(category, count, bytes) end

