---Registry of cost profile definitions (id -> definition).
---Filled by Shared/Definitions/CostProfileDefs.lua; read through CostProfileComponent.
---@class CostProfileRegistry
local CostProfileRegistry = {
    _definitions = {},
    _order = {},
}

---@param id CostProfileId
---@param definition CostProfile
---@return CostProfile
function CostProfileRegistry:new(id, definition)
    assert(type(id) == "number" and id > 0, "cost profile ID must be a positive number")
    assert(rawtype(definition) == "table", "cost profile definition must be a table")

    local existing = self._definitions[id]
    if existing then
        Log.Warn("Cost profile already registered: %s", tostring(id))
        return existing
    end

    self._definitions[id] = definition
    table.insert(self._order, id)
    return definition
end

---@param id CostProfileId
---@return CostProfile
function CostProfileRegistry:get(id)
    local definition = self._definitions[id]
    assert(definition, "unknown cost profile: " .. tostring(id))
    return definition
end

---@param id CostProfileId
---@return boolean
function CostProfileRegistry:has(id)
    return self._definitions[id] ~= nil
end

---@return CostProfileId[]
function CostProfileRegistry:ids()
    return self._order
end

return CostProfileRegistry
