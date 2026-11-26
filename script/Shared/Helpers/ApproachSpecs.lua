---Which approach spec a ship follows to a target: the one an order names, else the default of the
---target's kind, else a stand-off of `NavConfig.genericGapM` from the hull. The kinds are found by their marker
---components (the specs themselves are data: Shared/Definitions/ApproachSpecDefs.lua).
-- Design: doc/script/nav/navigation.md

local CelestialComponents = require("Modules.CelestialObjects.Components")
local ConstructComponents = require("Modules.Constructs.Components")
local ApproachSpec        = require("Shared.Types.ApproachSpec")
local NavConfig           = require("Config.Navigation.NavConfig")

-- The default spec of each kind of target, by marker component.
local DEFAULTS = {
    { marker = CelestialComponents.Planet,       spec = Enums.ApproachSpec.PlanetStandOff },
    { marker = CelestialComponents.Moon,         spec = Enums.ApproachSpec.MoonStandOff },
    { marker = CelestialComponents.Asteroid,     spec = Enums.ApproachSpec.AsteroidStandOff },
    { marker = ConstructComponents.SpaceStation, spec = Enums.ApproachSpec.StationApproach },
}

---@type ApproachSpec|nil The spec of a target that is none of the kinds above, made on first use
local generic = nil

---@class ApproachSpecs
local ApproachSpecs = {}

---@param target Entity
---@param named ApproachSpec|nil The spec the order names
---@return ApproachSpec
function ApproachSpecs.For(target, named)
    if named then return named end
    for _, default in ipairs(DEFAULTS) do
        if target:get(default.marker) then return ApproachSpec.Of(default.spec) end
    end
    generic = generic or ApproachSpec.new { name = "generic", kind = Enums.ApproachKind.StandOff,
        gapM = NavConfig.genericGapM }
    return generic
end

return ApproachSpecs
