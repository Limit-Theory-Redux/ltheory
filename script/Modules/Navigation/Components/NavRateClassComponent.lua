local Component = require("Core.ECS.Component")
local NavConfig = require("Config.Navigation.NavConfig")

---@class NavRateClassComponent: Component
---@overload fun(self: NavRateClassComponent, class: NavRateClassId): NavRateClassComponent subclass internal
---@overload fun(class: NavRateClassId): NavRateClassComponent subclass external
--- On a zone entity: the rate class a ship has while it is inside the zone. A ship inside several of
--- them has the highest rate of them.
local NavRateClassComponent = Subclass("NavRateClassComponent", Component, function(self, class)
    self:setComponentName("NavRateClass")
    self:setClass(class)
end)

---@param class NavRateClassId An `Enums.NavRateClass` value
function NavRateClassComponent:setClass(class)
    assert(NavConfig.rateClasses[class], "NavRateClass: not a rate class")
    self._class = class
end

---@return NavRateClassId
function NavRateClassComponent:getClass()
    return self._class
end

return NavRateClassComponent
