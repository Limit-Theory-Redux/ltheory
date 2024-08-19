local Component = require("GameObjects.Component")

---@class RandomNumberGeneratorComponent: Component
---@overload fun(self: table, seed: integer|nil, isManaged: boolean) subclass internal
---@overload fun(seed: integer|nil, isManaged: boolean): RandomNumberGeneratorComponent subclass external
---@return RandomNumberGeneratorComponent
local RandomNumberGeneratorComponent = Subclass(Component, function(self, seed, isManaged)
    ---@cast self RandomNumberGeneratorComponent
    self:setComponentName("RandomNumberGenerator")

    if isManaged then
        self:addManagedRNG(seed)
    else
        self:addRNG(seed)
    end
end)

-- providing no seed will create a rng from current time
---@param seed integer|nil
function RandomNumberGeneratorComponent:addRNG(seed)
    if seed then
        ---@type RandomNumberGenerator
        self.rng = RNG.Create(seed)
    else
        self.rng = RNG.FromTime()
    end
end

function RandomNumberGeneratorComponent:addManagedRNG(seed)
    ---@type RandomNumberGenerator
    self.rng = RNG.Create(seed):managed()
end

---@return RandomNumberGenerator
function RandomNumberGeneratorComponent:getRNG()
    return self.rng
end

return RandomNumberGeneratorComponent
