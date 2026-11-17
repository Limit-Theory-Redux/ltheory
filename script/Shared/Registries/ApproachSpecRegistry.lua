---Registry of approach spec definitions (id -> definition).
---Filled by Shared/Definitions/ApproachSpecDefs.lua; read through Shared/Helpers/ApproachSpecs.lua.
---@class ApproachSpecRegistry
local ApproachSpecRegistry = {
    _definitions = {},
    _order = {},
}

---@param id ApproachSpecId
---@param definition ApproachSpec
---@return ApproachSpec
function ApproachSpecRegistry:new(id, definition)
    assert(type(id) == "number" and id > 0, "approach spec ID must be a positive number")
    assert(rawtype(definition) == "table", "approach spec definition must be a table")

    local existing = self._definitions[id]
    if existing then
        Log.Warn("Approach spec already registered: %s", tostring(id))
        return existing
    end

    self._definitions[id] = definition
    table.insert(self._order, id)
    return definition
end

---@param id ApproachSpecId
---@return ApproachSpec
function ApproachSpecRegistry:get(id)
    local definition = self._definitions[id]
    assert(definition, "unknown approach spec: " .. tostring(id))
    return definition
end

---@param id ApproachSpecId
---@return boolean
function ApproachSpecRegistry:has(id)
    return self._definitions[id] ~= nil
end

---@return ApproachSpecId[]
function ApproachSpecRegistry:ids()
    return self._order
end

return ApproachSpecRegistry
