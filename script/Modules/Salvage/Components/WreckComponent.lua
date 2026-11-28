local Component = require("Core.ECS.Component")

---What is left of a lost ship: the kind and hull design it was (to build its hulk again), its name,
---what it yields to salvage (scrap from its hull and the cargo it carried) and how much more damage
---it takes before it breaks up. Its place and drift are its `VirtualBodyComponent`; `WreckSystem`
---keeps it.
---@class WreckComponent: Component
---@field kind string A key of `ShipKindDefs`
---@field seed integer Its hull design
---@field name string
---@field scrap number Units of scrap left in its hull
---@field cargo table<integer, integer> Item id -> units left in its hold
---@field integrity number
---@field maxIntegrity number
---@overload fun(self: WreckComponent, spec: table): WreckComponent subclass internal
---@overload fun(spec: table): WreckComponent subclass external
local WreckComponent = Subclass("WreckComponent", Component, function(self, spec)
    self:setComponentName("Wreck")

    self.kind = spec.kind
    self.seed = spec.seed
    self.name = spec.name
    self.scrap = spec.scrap or 0
    self.cargo = spec.cargo or {}
    self.maxIntegrity = spec.integrity or 100
    self.integrity = self.maxIntegrity
end)

---@return string
function WreckComponent:getKind()
    return self.kind
end

---@return integer
function WreckComponent:getSeed()
    return self.seed
end

---@return string
function WreckComponent:getName()
    return self.name
end

---@return number
function WreckComponent:getScrap()
    return self.scrap
end

---@param scrap number
function WreckComponent:setScrap(scrap)
    self.scrap = math.max(0, scrap)
end

---@return table<integer, integer>
function WreckComponent:getCargo()
    return self.cargo
end

---@return number
function WreckComponent:getIntegrity()
    return self.integrity
end

---@param integrity number
function WreckComponent:setIntegrity(integrity)
    self.integrity = math.max(0, math.min(self.maxIntegrity, integrity))
end

---@return number
function WreckComponent:getMaxIntegrity()
    return self.maxIntegrity
end

---Whether nothing is left to salvage.
---@return boolean
function WreckComponent:isEmpty()
    if self.scrap > 0 then return false end
    for _, quantity in pairs(self.cargo) do
        if quantity > 0 then return false end
    end
    return true
end

return WreckComponent
