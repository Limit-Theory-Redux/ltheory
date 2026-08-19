---@class BeamRegistry
local BeamRegistry = {
    _definitions = {},
    _order = {},
}

---@param id integer
---@param definition BeamDefinition
---@return BeamDefinition
function BeamRegistry:new(id, definition)
    assert(type(id) == "number" and id > 0, "beam ID must be a positive number")
    assert(rawtype(definition) == "table", "beam definition must be a table")

    local existing = self._definitions[id]
    if existing then
        Log.Warn("Beam already registered: %s", tostring(id))
        return existing
    end

    definition.id = id
    self._definitions[id] = definition
    table.insert(self._order, id)
    return definition
end

---@param id integer
---@return BeamDefinition|nil
function BeamRegistry:get(id)
    return self._definitions[id]
end

---@return integer[]
function BeamRegistry:getIds()
    return self._order
end

setmetatable(BeamRegistry, {
    __index = function(self, id)
        local definition = rawget(self, "_definitions")[id]
        if definition then
            return definition
        end
        return rawget(BeamRegistry, id)
    end,
})

return BeamRegistry
