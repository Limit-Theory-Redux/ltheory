local Component = require('Components.Component')

---@class SeedComponent: Component
---@overload fun(self:SeedComponent, seed: integer): SeedComponent subclass internal
---@overload fun(seed: integer): SeedComponent subclass external
local SeedComponent = Subclass("SeedComponent", Component, function(self, seed)
    self:setComponentName("GenerationSeedComponent")

    -- Set Component Archetype
    self:setArchetype(Enums.ComponentArchetype.SeedComponent)

    ---@diagnostic disable-next-line: invisible
    self:init(seed)
end)

---@private
function SeedComponent:init(seed)
    self.seed = seed
end

---@return integer
function SeedComponent:getSeed()
    return self.seed
end

return SeedComponent
