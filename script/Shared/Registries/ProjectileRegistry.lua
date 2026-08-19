---@class ProjectileRegistry
local ProjectileRegistry = {
    _definitions = {},
    _order = {},
}

---@param id integer
---@param definition ProjectileDefinition
---@return ProjectileDefinition
function ProjectileRegistry:new(id, definition)
    assert(type(id) == "number" and id > 0, "projectile ID must be a positive number")
    assert(rawtype(definition) == "table", "projectile definition must be a table")

    local existing = self._definitions[id]
    if existing then
        Log.Warn("Projectile already registered: %s", tostring(id))
        return existing
    end

    definition.id = id
    self._definitions[id] = definition
    table.insert(self._order, id)
    return definition
end

---@param id integer
---@return ProjectileDefinition|nil
function ProjectileRegistry:get(id)
    return self._definitions[id]
end

---@return integer[]
function ProjectileRegistry:getIds()
    return self._order
end

setmetatable(ProjectileRegistry, {
    __index = function(self, id)
        local definition = rawget(self, "_definitions")[id]
        if definition then
            return definition
        end
        return rawget(ProjectileRegistry, id)
    end,
})

return ProjectileRegistry
