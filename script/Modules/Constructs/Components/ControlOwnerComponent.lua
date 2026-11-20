local Component = require("Core.ECS.Component")

--- Who owns a ship's control input: the highest of the owners that want it (manual input, then a
--- direct-control claim, then orders). Data only; `ControlOwnerSystem` changes it.
---@class ControlOwnerComponent: Component
---@overload fun(): ControlOwnerComponent
local ControlOwnerComponent = Subclass("ControlOwnerComponent", Component, function(self)
    self:setComponentName("ControlOwner")

    self._wants = {
        [Enums.ControlOwner.Orders] = false,
        [Enums.ControlOwner.Claim]  = false,
        [Enums.ControlOwner.Manual] = false,
    }
    self._owner = Enums.ControlOwner.None
end)

---@return ControlOwnerId
function ControlOwnerComponent:getOwner() return self._owner end

---@param owner ControlOwnerId `Orders`, `Claim` or `Manual`
---@return boolean
function ControlOwnerComponent:wants(owner) return self._wants[owner] end

---@param owner ControlOwnerId `Orders`, `Claim` or `Manual`
---@param on boolean
function ControlOwnerComponent:setWants(owner, on)
    assert(self._wants[owner] ~= nil, "ControlOwner: not an owner that can want the controls")
    self._wants[owner] = on
end

---@param owner ControlOwnerId
function ControlOwnerComponent:setOwner(owner) self._owner = owner end

return ControlOwnerComponent
