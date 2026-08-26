local Component = require("Core.ECS.Component")

---@class TidalLockComponent: Component
---@overload fun(self: TidalLockComponent, isLocked: boolean): TidalLockComponent subclass internal
---@overload fun(isLocked: boolean): TidalLockComponent subclass external
local TidalLockComponent = Subclass("TidalLockComponent", Component, function(self, isLocked)
    self:setComponentName("CelestialTidalLockComponent")
    self.isLocked = isLocked or false
end)

---@return boolean
function TidalLockComponent:isTidallyLocked()
    return self.isLocked
end

---@param locked boolean
function TidalLockComponent:setTidallyLocked(locked)
    self.isLocked = locked
end

return TidalLockComponent
