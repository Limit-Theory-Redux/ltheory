local Component = require('Component')

---@class SystemEconomy: Component
---@overload fun(): SystemEconomy subclass external
local SystemEconomy = Subclass(Component, function(self)
    ---@cast self SystemEconomy
    self:setComponentName("SystemEconomy")
end)

-- Actors

-- Assets

-- Flows

-- Resources

return SystemEconomy
