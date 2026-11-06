---Registry of flight profile definitions (id -> definition).
---Filled by Shared/Definitions/FlightProfileDefs.lua; read through
---FlightProfileComponent by the flight systems.
---@class FlightProfileRegistry
local FlightProfileRegistry = {
    _definitions = {},
    _order = {},
}

---@param id FlightProfileId
---@param definition FlightProfile
---@return FlightProfile
function FlightProfileRegistry:new(id, definition)
    assert(type(id) == "number" and id > 0, "flight profile ID must be a positive number")
    assert(rawtype(definition) == "table", "flight profile definition must be a table")

    local existing = self._definitions[id]
    if existing then
        Log.Warn("Flight profile already registered: %s", tostring(id))
        return existing
    end

    self._definitions[id] = definition
    table.insert(self._order, id)
    return definition
end

---@param id FlightProfileId
---@return FlightProfile
function FlightProfileRegistry:get(id)
    local definition = self._definitions[id]
    assert(definition, "unknown flight profile: " .. tostring(id))
    return definition
end

---@param id FlightProfileId
---@return boolean
function FlightProfileRegistry:has(id)
    return self._definitions[id] ~= nil
end

---@return FlightProfileId[]
function FlightProfileRegistry:getIds()
    return self._order
end

return FlightProfileRegistry
