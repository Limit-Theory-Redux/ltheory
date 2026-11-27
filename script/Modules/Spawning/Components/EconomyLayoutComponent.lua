local Component = require("Core.ECS.Component")

---@class LayoutPlace
---@field planet integer Index of the star system's planet the place is by (1 is the first)
---@field angleDeg number In the planet's equatorial plane
---@field radii number From the planet's centre, in planet radii
---@field heightRadii number Over the equatorial plane, in planet radii

---@class LayoutStation: LayoutPlace
---@field kind string A key of `StationEconomyDefs`

---@class LayoutFaction
---@field type integer An `Enums.FactionType`
---@field seed integer Names the faction and its captains
---@field stations LayoutStation[]
---@field haulers integer
---@field miners integer
---@field fighters integer

---@class LayoutOreField: LayoutPlace
---@field count integer Rocks

---@class EconomyLayout
---@field factions LayoutFaction[]
---@field oreFields LayoutOreField[]
---@field marauders { pirates: integer, place: LayoutPlace }|nil

---A star system's economy as its generation planned it (`EconomyPlanner`): data only, built into the
---scene by `SpawnerSystem:populate`.
---@class EconomyLayoutComponent: Component
---@overload fun(self: EconomyLayoutComponent, layout: EconomyLayout): EconomyLayoutComponent subclass internal
---@overload fun(layout: EconomyLayout): EconomyLayoutComponent subclass external
local EconomyLayoutComponent = Subclass("EconomyLayoutComponent", Component, function(self, layout)
    self:setComponentName("EconomyLayout")

    self.layout = layout
end)

---@return EconomyLayout
function EconomyLayoutComponent:getLayout()
    return self.layout
end

return EconomyLayoutComponent
