--- Parameters of the celestial body materials (`PlanetSurface`, `PlanetAtmosphere`,
--- `PlanetRing`, `MoonSurface`), written once into their `MaterialParams`.
--- The values used to come from the entity's gen component and rigid body
--- through per-instance shader variables every frame.
local PlanetMaterials = {}

--- A planet and its atmosphere.
---@param matPlanet Material a `PlanetSurface` instance
---@param matAtmo Material|nil a `PlanetAtmosphere` instance
---@param gen table `color1..4`, `oceanLevel`, `atmoScale` (the planet's gen options)
---@param scale number the planet's scale (its rigid body's)
function PlanetMaterials.planet(matPlanet, matAtmo, gen, scale)
    local rAtmo = scale * gen.atmoScale

    local p = matPlanet:params()
    p.color1, p.color2, p.color3, p.color4 = gen.color1, gen.color2, gen.color3, gen.color4
    p.oceanLevel = gen.oceanLevel
    p.rAtmo = rAtmo
    matPlanet:commit()

    if matAtmo then
        matAtmo:params().rAtmo = rAtmo
        matAtmo:commit()
    end
end

--- A moon.
---@param matMoon Material a `MoonSurface` instance
---@param gen table `highlandColor`, `mariaColor` (the moon's gen options)
function PlanetMaterials.moon(matMoon, gen)
    local p = matMoon:params()
    p.highlandColor = gen.highlandColor
    p.mariaColor = gen.mariaColor
    matMoon:commit()
end

--- The procedural band of a planetary ring.
---@param matRing Material a `PlanetRing` instance
---@param innerRadius number
---@param outerRadius number
---@param seed integer the ring entity's seed
function PlanetMaterials.ring(matRing, innerRadius, outerRadius, seed)
    local p = matRing:params()
    p.rMin = innerRadius
    p.rMax = outerRadius
    p.seed = seed
    matRing:commit()
end

return PlanetMaterials
