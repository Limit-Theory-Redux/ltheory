---@class LaserProfileRegistry
local LaserProfileRegistry = {
    _definitions = {},
    _order = {},
}

---@param id integer
---@param definition LaserProfileDefinition
---@return LaserProfileDefinition
function LaserProfileRegistry:new(id, definition)
    assert(type(id) == "number" and id > 0, "laser profile ID must be a positive number")
    assert(rawtype(definition) == "table", "laser profile definition must be a table")

    local existing = self._definitions[id]
    if existing then
        Log.Warn("Laser profile already registered: %s", tostring(id))
        return existing
    end

    definition.id = id
    self._definitions[id] = definition
    table.insert(self._order, id)
    return definition
end

---@param id integer
---@return LaserProfileDefinition|nil
function LaserProfileRegistry:get(id)
    return self._definitions[id]
end

---@param id integer
---@return boolean
function LaserProfileRegistry:has(id)
    return self._definitions[id] ~= nil
end

---@return integer[]
function LaserProfileRegistry:getIds()
    return self._order
end

setmetatable(LaserProfileRegistry, {
    __index = function(self, id)
        local definition = rawget(self, "_definitions")[id]
        if definition then
            return definition
        end
        return rawget(LaserProfileRegistry, id)
    end,
})

return LaserProfileRegistry
