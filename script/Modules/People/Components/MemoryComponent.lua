local Component = require("Core.ECS.Component")

---@class MemoryEntry
---@field event string A `LifeEventDefs` id
---@field subjectId integer|nil Who or what it was about
---@field at number Simulated seconds

---What a person remembers and thinks of others: the events they lived through that count
---(`LifeEventDefs.remember`), the latest `capacity` of them, and an opinion, -1 to 1, of each person,
---faction or other entity events were about.
---@class MemoryComponent: Component
---@field entries MemoryEntry[]
---@field capacity integer
---@field opinions table<integer, number>
---@overload fun(self: MemoryComponent, capacity: integer|nil): MemoryComponent subclass internal
---@overload fun(capacity: integer|nil): MemoryComponent subclass external
local MemoryComponent = Subclass("MemoryComponent", Component, function(self, capacity)
    self:setComponentName("Memory")

    self.entries = {}
    self.capacity = capacity or 32
    self.opinions = {}
end)

---@param entry MemoryEntry
function MemoryComponent:remember(entry)
    self.entries[#self.entries + 1] = entry
    if #self.entries > self.capacity then table.remove(self.entries, 1) end
end

---@return MemoryEntry[]
function MemoryComponent:getEntries()
    return self.entries
end

---@param subjectId integer
---@return number
function MemoryComponent:getOpinion(subjectId)
    return self.opinions[subjectId] or 0
end

---@param subjectId integer
---@param opinion number
function MemoryComponent:setOpinion(subjectId, opinion)
    self.opinions[subjectId] = math.max(-1, math.min(1, opinion))
end

return MemoryComponent
