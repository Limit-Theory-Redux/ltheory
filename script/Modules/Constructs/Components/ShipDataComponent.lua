local Component = require("Core.ECS.Component")

---@class ShipDataComponent: Component
---@field private _shipType ShipType
---@field private _hull ShipHull
---@field private _res any
---@field private _variant string|nil
---@field private _class string|nil
---@overload fun(self: ShipDataComponent, shipType: ShipType, hull: any, res: any): ShipDataComponent subclass internal
---@overload fun(shipType: ShipType, hull: any, res: any): ShipDataComponent subclass external
local ShipDataComponent = Subclass("ShipDataComponent", Component, function(
    self,
    shipType,
    hull,
    res
)
    self:setComponentName("ShipDataComponent")

    self._shipType = shipType -- Fighter, Capital, Basic, etc.
    self._hull = hull         -- Hull data used for generation
    self._res = res           -- Resolution data used for generation
    self._variant = nil       -- Optional subtype (Surreal, Standard)
    self._class = nil         -- Ship class (e.g., "Interceptor")
end)

-- GETTERS
function ShipDataComponent:getShipType() return self._shipType end
function ShipDataComponent:getHull() return self._hull end
function ShipDataComponent:getRes() return self._res end
function ShipDataComponent:getVariant() return self._variant end
function ShipDataComponent:getClass() return self._class end

-- SETTERS
function ShipDataComponent:setShipType(v) self._shipType = v end
function ShipDataComponent:setHull(v) self._hull = v end
function ShipDataComponent:setRes(v) self._res = v end
function ShipDataComponent:setVariant(v) self._variant = v end
function ShipDataComponent:setClass(v) self._class = v end

return ShipDataComponent
