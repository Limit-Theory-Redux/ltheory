-- Calculates 3D positions for celestial objects using orbital mechanics

---@class PositionCalculator
local PositionCalculator = {}

---Convert degrees to radians
---@param degrees number Angle in degrees
---@return number radians Angle in radians
local function degToRad(degrees)
    return degrees * math.pi / 180.0
end

---Solve Kepler's equation for eccentric anomaly (iterative method)
---@param meanAnomaly number Mean anomaly in radians
---@param eccentricity number Orbital eccentricity (0-1)
---@return number eccentricAnomaly Eccentric anomaly in radians
local function solveKeplersEquation(meanAnomaly, eccentricity)
    local E = meanAnomaly -- Initial guess
    local tolerance = 1e-6
    local maxIterations = 30

    for i = 1, maxIterations do
        local dE = (E - eccentricity * math.sin(E) - meanAnomaly) / (1.0 - eccentricity * math.cos(E))
        E = E - dE
        if math.abs(dE) < tolerance then
            break
        end
    end

    return E
end

---Calculate 3D Cartesian position from orbital elements
---@param semiMajorAxis gameunit Semi-major axis (in game units)
---@param eccentricity number Eccentricity (0-1)
---@param inclination number Inclination in degrees (0-180)
---@param longAscNode number Longitude of ascending node in degrees (0-360)
---@param argPeriapsis number Argument of periapsis in degrees (0-360)
---@param meanAnomaly number Mean anomaly in degrees (0-360)
---@return table position Position object {x, y, z}
function PositionCalculator.orbitalPosition(semiMajorAxis, eccentricity, inclination,
                                            longAscNode, argPeriapsis, meanAnomaly)
    -- Convert angles to radians
    local i = degToRad(inclination)
    local Omega = degToRad(longAscNode)
    local omega = degToRad(argPeriapsis)
    local M = degToRad(meanAnomaly)

    -- Solve Kepler's equation for eccentric anomaly
    local E = solveKeplersEquation(M, eccentricity)

    -- Calculate true anomaly
    local nu = 2.0 * math.atan2(
        math.sqrt(1.0 + eccentricity) * math.sin(E / 2.0),
        math.sqrt(1.0 - eccentricity) * math.cos(E / 2.0)
    )

    -- Calculate distance from focus
    local r = semiMajorAxis * (1.0 - eccentricity * math.cos(E))

    -- Position in orbital plane
    local x_orb = r * math.cos(nu)
    local y_orb = r * math.sin(nu)

    -- Rotation matrices to convert to 3D space
    local cosOmega = math.cos(Omega)
    local sinOmega = math.sin(Omega)
    local cosI = math.cos(i)
    local sinI = math.sin(i)
    local cosOmega_small = math.cos(omega)
    local sinOmega_small = math.sin(omega)

    -- Apply rotations: R_z(Omega) * R_x(i) * R_z(omega)
    local x = (cosOmega * cosOmega_small - sinOmega * sinOmega_small * cosI) * x_orb +
        (-cosOmega * sinOmega_small - sinOmega * cosOmega_small * cosI) * y_orb

    local y = (sinOmega * cosOmega_small + cosOmega * sinOmega_small * cosI) * x_orb +
        (-sinOmega * sinOmega_small + cosOmega * cosOmega_small * cosI) * y_orb

    local z = (sinOmega_small * sinI) * x_orb + (cosOmega_small * sinI) * y_orb

    return Position(x, y, z)
end

---Calculate positions for binary stars around their barycenter
---@param mass1 number Mass of first star (solar masses)
---@param mass2 number Mass of second star (solar masses)
---@param separation number Separation distance in game units
---@return table pos1 Position of star 1
---@return table pos2 Position of star 2
function PositionCalculator.binaryStarPositions(mass1, mass2, separation)
    local totalMass = mass1 + mass2

    -- Distance from barycenter to each star
    local r1 = separation * (mass2 / totalMass)
    local r2 = separation * (mass1 / totalMass)

    -- Place stars on X-axis for simplicity (can be rotated later if needed)
    local pos1 = Position(-r1, 0, 0)
    local pos2 = Position(r2, 0, 0)

    return pos1, pos2
end

---Calculate position for trinary star system (hierarchical)
---@param mass1 number Mass of inner binary star 1 (solar masses)
---@param mass2 number Mass of inner binary star 2 (solar masses)
---@param mass3 number Mass of outer star (solar masses)
---@param innerSeparation number Separation of inner binary in game units
---@param outerSeparation number Separation of outer star from inner barycenter in game units
---@return table pos1 Position of star 1
---@return table pos2 Position of star 2
---@return table pos3 Position of star 3
function PositionCalculator.trinaryStarPositions(mass1, mass2, mass3, innerSeparation, outerSeparation)
    -- Inner binary around their barycenter
    local pos1, pos2 = PositionCalculator.binaryStarPositions(mass1, mass2, innerSeparation)

    -- Outer star relative to inner binary barycenter
    local innerMass = mass1 + mass2
    local totalMass = innerMass + mass3

    local r_inner = outerSeparation * (mass3 / totalMass)
    local r_outer = outerSeparation * (innerMass / totalMass)

    -- Offset inner binary
    pos1.x = pos1.x - r_inner
    pos2.x = pos2.x - r_inner

    -- Place outer star
    local pos3 = Position(r_outer, 0, 0)

    return pos1, pos2, pos3
end

---Generate random position in universe based on distribution type
---@param rng table RNG instance
---@param distributionType string "sphere", "cube", or "grid"
---@param radius number Radius for sphere distribution (game units)
---@param gridSpacing number|nil Grid spacing for grid distribution
---@param gridJitter number|nil Jitter factor for grid (0-1)
---@param index number|nil Index for grid distribution
---@return table position Position object
function PositionCalculator.universePosition(rng, distributionType, radius, gridSpacing, gridJitter, index)
    if distributionType == "sphere" then
        -- Random position in sphere (uniform distribution)
        local theta = rng:getUniformRange(0, 2 * math.pi)
        local phi = math.acos(rng:getUniformRange(-1, 1))
        local r = radius * (rng:getUniform() ^ (1 / 3)) -- Cube root for uniform volume

        local x = r * math.sin(phi) * math.cos(theta)
        local y = r * math.sin(phi) * math.sin(theta)
        local z = r * math.cos(phi)

        return Position(x, y, z)
    elseif distributionType == "cube" then
        -- Random position in cube
        local x = rng:getUniformRange(-radius, radius)
        local y = rng:getUniformRange(-radius, radius)
        local z = rng:getUniformRange(-radius, radius)

        return Position(x, y, z)
    elseif distributionType == "grid" then
        -- Grid with optional jitter
        gridSpacing = gridSpacing or radius / 10
        gridJitter = gridJitter or 0.3
        index = index or 1

        -- Simple 3D grid indexing
        local gridSize = math.floor(2 * radius / gridSpacing)
        local x_idx = index % gridSize
        local y_idx = math.floor(index / gridSize) % gridSize
        local z_idx = math.floor(index / (gridSize * gridSize))

        local x = (x_idx - gridSize / 2) * gridSpacing
        local y = (y_idx - gridSize / 2) * gridSpacing
        local z = (z_idx - gridSize / 2) * gridSpacing

        -- Add jitter
        local jitterAmount = gridSpacing * gridJitter
        x = x + rng:getUniformRange(-jitterAmount, jitterAmount)
        y = y + rng:getUniformRange(-jitterAmount, jitterAmount)
        z = z + rng:getUniformRange(-jitterAmount, jitterAmount)

        return Position(x, y, z)
    end

    -- Default: origin
    return Position(0, 0, 0)
end

return PositionCalculator
