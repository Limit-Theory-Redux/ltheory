local Component = require("Core.ECS.Component")

local ShipComponents = {}

---@class ShipDataComponent: Component
---@overload fun(self: ShipDataComponent, shipType: integer, hull: any, res: any): ShipDataComponent subclass internal
---@overload fun(shipType: ShipType, hull: any, res: any): ShipDataComponent subclass external
local ShipDataComponent = Subclass("ShipDataComponent", Component, function(self, shipType, hull, res)
    self:setComponentName("ShipDataComponent")

    self.shipType = shipType -- Type of ship (Fighter=1, Capital=2, Basic=3)
    self.hull = hull         -- Hull data used for generation
    self.res = res           -- Resolution data used for generation
    self.variant = nil       -- Variant within type (e.g., Standard, Surreal for fighters)
    self.faction = nil       -- Faction the ship belongs to
    self.class = nil         -- Ship class name
end)

function ShipDataComponent:setVariant(variant)
    self.variant = variant
end

function ShipDataComponent:getVariant()
    return self.variant
end

function ShipDataComponent:setFaction(faction)
    self.faction = faction
end

function ShipDataComponent:getFaction()
    return self.faction
end

function ShipDataComponent:setClass(class)
    self.class = class
end

function ShipDataComponent:getClass()
    return self.class
end

return ShipDataComponent
