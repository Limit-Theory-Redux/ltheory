---@class CapitalHullGenerator
local CapitalHullGenerator = {}

local GENERATOR_ID = Enums.ShipGeneration.LayeredCapital
local GENERATOR_VERSION = 1
local MountZone = Enums.Weapon.MountZone
local MountSide = Enums.Weapon.MountSide
local MountSurfaceBand = Enums.Weapon.MountSurfaceBand
local MountId = Enums.Weapon.MountId
local MountPairId = Enums.Weapon.MountPairId

local DEFAULT_PROFILE = {
    { z = 0.00, beam = 0.24, height = 0.28 },
    { z = 0.12, beam = 0.56, height = 0.55 },
    { z = 0.25, beam = 0.82, height = 0.78 },
    { z = 0.40, beam = 0.96, height = 0.92 },
    { z = 0.55, beam = 1.00, height = 1.00 },
    { z = 0.70, beam = 0.96, height = 0.92 },
    { z = 0.82, beam = 0.82, height = 0.78 },
    { z = 0.94, beam = 0.56, height = 0.55 },
    { z = 1.00, beam = 0.22, height = 0.26 },
}

local DEFAULT_SOCKETS = {
    { pairId = "fore_top", zone = MountZone.Fore, surfaceBand = MountSurfaceBand.Top },
    { pairId = "fore_bottom", zone = MountZone.Fore, surfaceBand = MountSurfaceBand.Bottom },
    { pairId = "mid_top", zone = MountZone.Mid, surfaceBand = MountSurfaceBand.Top, z = 0.00 },
    { pairId = "mid_bottom", zone = MountZone.Mid, surfaceBand = MountSurfaceBand.Bottom, z = 0.00 },
    { pairId = "aft_top", zone = MountZone.Aft, surfaceBand = MountSurfaceBand.Top },
    { pairId = "aft_bottom", zone = MountZone.Aft, surfaceBand = MountSurfaceBand.Bottom },
}

local MOUNT_IDS_BY_PAIR = {
    [MountPairId.ForeOuter] = {
        [MountSide.Port] = MountId.ForeOuterPort,
        [MountSide.Starboard] = MountId.ForeOuterStarboard,
    },
    [MountPairId.ForeInner] = {
        [MountSide.Port] = MountId.ForeInnerPort,
        [MountSide.Starboard] = MountId.ForeInnerStarboard,
    },
    [MountPairId.ForePdOuter] = {
        [MountSide.Port] = MountId.ForePdOuterPort,
        [MountSide.Starboard] = MountId.ForePdOuterStarboard,
    },
    [MountPairId.MidPd] = {
        [MountSide.Port] = MountId.MidPdPort,
        [MountSide.Starboard] = MountId.MidPdStarboard,
    },
    [MountPairId.Mid] = {
        [MountSide.Port] = MountId.MidPort,
        [MountSide.Starboard] = MountId.MidStarboard,
    },
    [MountPairId.MidOuter] = {
        [MountSide.Port] = MountId.MidOuterPort,
        [MountSide.Starboard] = MountId.MidOuterStarboard,
    },
    [MountPairId.ForeHeavy] = {
        [MountSide.Port] = MountId.ForeHeavyPort,
        [MountSide.Starboard] = MountId.ForeHeavyStarboard,
    },
    [MountPairId.MidHeavy] = {
        [MountSide.Port] = MountId.MidHeavyPort,
        [MountSide.Starboard] = MountId.MidHeavyStarboard,
    },
    [MountPairId.FlakBattery] = {
        [MountSide.Port] = MountId.FlakBatteryPort,
        [MountSide.Starboard] = MountId.FlakBatteryStarboard,
    },
    [MountPairId.RailgunBattery] = {
        [MountSide.Port] = MountId.RailgunBatteryPort,
        [MountSide.Starboard] = MountId.RailgunBatteryStarboard,
    },
    [MountPairId.ForePdBottom] = {
        [MountSide.Port] = MountId.ForePdBottomPort,
        [MountSide.Starboard] = MountId.ForePdBottomStarboard,
    },
    [MountPairId.MidPdBottom] = {
        [MountSide.Port] = MountId.MidPdBottomPort,
        [MountSide.Starboard] = MountId.MidPdBottomStarboard,
    },
    [MountPairId.AftPdBottom] = {
        [MountSide.Port] = MountId.AftPdBottomPort,
        [MountSide.Starboard] = MountId.AftPdBottomStarboard,
    },
    [MountPairId.AftInner] = {
        [MountSide.Port] = MountId.AftInnerPort,
        [MountSide.Starboard] = MountId.AftInnerStarboard,
    },
    [MountPairId.AftPd] = {
        [MountSide.Port] = MountId.AftPdPort,
        [MountSide.Starboard] = MountId.AftPdStarboard,
    },
    [MountPairId.AftOuter] = {
        [MountSide.Port] = MountId.AftOuterPort,
        [MountSide.Starboard] = MountId.AftOuterStarboard,
    },
}

local function socketIdFor(descriptor, side, index)
    local pairId = descriptor.pairId or descriptor.socketId
    local knownPair = pairId and MOUNT_IDS_BY_PAIR[pairId]
    if knownPair and knownPair[side] then
        return knownPair[side]
    end
    return (descriptor.socketId or pairId or ("socket_" .. index))
        .. "_" .. side
end

local function bounded(value, minimum, maximum, fallback)
    value = value == nil and fallback or value
    return math.max(minimum, math.min(maximum, value))
end

local function copyTable(source)
    local result = {}
    for key, value in pairs(source or {}) do
        if type(value) == "table" then
            result[key] = copyTable(value)
        else
            result[key] = value
        end
    end
    return result
end

local function childRng(master)
    local child = RNG.Create(master:get64())
    assert(child, "capital hull generation could not create a child RNG")
    return child
end

local function sampleProfile(profile, normalizedZ)
    if #profile == 0 then
        profile = DEFAULT_PROFILE
    end
    if normalizedZ <= profile[1].z then
        return profile[1].beam, profile[1].height
    end
    for index = 2, #profile do
        local previous = profile[index - 1]
        local current = profile[index]
        if normalizedZ <= current.z then
            local span = math.max(current.z - previous.z, 0.000001)
            local factor = (normalizedZ - previous.z) / span
            return previous.beam + (current.beam - previous.beam) * factor,
                previous.height + (current.height - previous.height) * factor
        end
    end
    local last = profile[#profile]
    return last.beam, last.height
end

local function addVertex(mesh, x, y, z, u, v)
    local index = mesh:getVertexCount()
    mesh:addVertex(x, y, z, 1, 0, 0, u or 0, v or 0)
    return index
end

local function addTaperedBox(mesh, center, halfX, halfY, halfZ, aftScale, foreScale)
    local first = mesh:getVertexCount()
    aftScale = aftScale or 1
    foreScale = foreScale or 1

    local aftX = halfX * aftScale
    local aftY = halfY * aftScale
    local foreX = halfX * foreScale
    local foreY = halfY * foreScale
    local zAft = center.z - halfZ
    local zFore = center.z + halfZ

    addVertex(mesh, center.x - aftX, center.y + aftY, zAft)
    addVertex(mesh, center.x + aftX, center.y + aftY, zAft)
    addVertex(mesh, center.x - foreX, center.y + foreY, zFore)
    addVertex(mesh, center.x + foreX, center.y + foreY, zFore)
    addVertex(mesh, center.x - aftX, center.y - aftY, zAft)
    addVertex(mesh, center.x + aftX, center.y - aftY, zAft)
    addVertex(mesh, center.x - foreX, center.y - foreY, zFore)
    addVertex(mesh, center.x + foreX, center.y - foreY, zFore)

    -- top, bottom, aft, fore, port, starboard
    mesh:addTri(first + 0, first + 2, first + 3)
    mesh:addTri(first + 0, first + 3, first + 1)
    mesh:addTri(first + 4, first + 5, first + 7)
    mesh:addTri(first + 4, first + 7, first + 6)
    mesh:addTri(first + 0, first + 1, first + 5)
    mesh:addTri(first + 0, first + 5, first + 4)
    mesh:addTri(first + 2, first + 6, first + 7)
    mesh:addTri(first + 2, first + 7, first + 3)
    mesh:addTri(first + 0, first + 4, first + 6)
    mesh:addTri(first + 0, first + 6, first + 2)
    mesh:addTri(first + 1, first + 3, first + 7)
    mesh:addTri(first + 1, first + 7, first + 5)
end

local function addCoreHull(mesh, dimensions, frame, rng)
    local stations = math.floor(bounded(frame.stations, 5, 17, 9))
    local slices = math.floor(bounded(frame.slices, 8, 32, 12))
    local variation = bounded(frame.variation, 0, 0.30, 0.08)
    local profile = frame.profile or DEFAULT_PROFILE
    local length = dimensions.length
    local beam = dimensions.beam
    local height = dimensions.height
    local rings = {}

    for station = 1, stations do
        local normalizedZ = (station - 1) / (stations - 1)
        local beamFactor, heightFactor = sampleProfile(profile, normalizedZ)
        local jitter = (station == 1 or station == stations)
            and 1
            or 1 + rng:getUniformRange(-variation, variation)
        local ring = {}
        local z = (normalizedZ - 0.5) * length
        local ringBeam = math.max(beam * beamFactor * jitter, beam * 0.04)
        local ringHeight = math.max(height * heightFactor * jitter, height * 0.04)

        for slice = 0, slices - 1 do
            local angle = (2 * math.pi * slice) / slices
            local x = 0.5 * ringBeam * math.cos(angle)
            local y = 0.5 * ringHeight * math.sin(angle)
            ring[slice + 1] = addVertex(mesh, x, y, z,
                slice / slices,
                normalizedZ)
        end
        rings[station] = ring
    end

    for station = 1, stations - 1 do
        local current = rings[station]
        local nextRing = rings[station + 1]
        for slice = 1, slices do
            local nextSlice = slice == slices and 1 or slice + 1
            local a = current[slice]
            local b = current[nextSlice]
            local c = nextRing[nextSlice]
            local d = nextRing[slice]
            mesh:addTri(a, b, c)
            mesh:addTri(a, c, d)
        end
    end

    local aftCenter = addVertex(mesh, 0, 0, -0.5 * length)
    local foreCenter = addVertex(mesh, 0, 0, 0.5 * length)
    local aftRing = rings[1]
    local foreRing = rings[stations]
    for slice = 1, slices do
        local nextSlice = slice == slices and 1 or slice + 1
        mesh:addTri(aftCenter, aftRing[nextSlice], aftRing[slice])
        mesh:addTri(foreCenter, foreRing[slice], foreRing[nextSlice])
    end
end

local function normalForSurface(surfaceBand, sideSign)
    if surfaceBand == MountSurfaceBand.Bottom
        or surfaceBand == MountSurfaceBand.Ventral
        or surfaceBand == MountSurfaceBand.Underside
    then
        return Vec3f(0, -1, 0)
    elseif surfaceBand == MountSurfaceBand.Side then
        return Vec3f(sideSign, 0, 0)
    end
    return Vec3f(0, 1, 0)
end

local function addStructuralSockets(mesh, dimensions, mountDecks)
    local sockets = {}
    local layout = mountDecks.sockets or DEFAULT_SOCKETS
    local halfBeam = dimensions.beam * 0.5
    local halfHeight = dimensions.height * 0.5

    for index, descriptor in ipairs(layout) do
        local surfaceBand = descriptor.surfaceBand or MountSurfaceBand.Top
        local z = descriptor.z
        if z == nil then
            local zone = descriptor.zone
            z = zone == MountZone.Fore and dimensions.length * 0.30
                or zone == MountZone.Aft and -dimensions.length * 0.30
                or 0
        end
        local xFactor = descriptor.xFactor
            or (surfaceBand == MountSurfaceBand.Side and 0.74 or 0.78)
        local centerX = halfBeam * xFactor
        local halfZ = descriptor.padHalfDepth or dimensions.length * 0.035
        local halfX = descriptor.padHalfWidth or dimensions.beam * 0.07
        local halfY = descriptor.padHalfHeight or dimensions.height * 0.10
        local centerY
        if surfaceBand == MountSurfaceBand.Bottom
            or surfaceBand == MountSurfaceBand.Ventral
            or surfaceBand == MountSurfaceBand.Underside
        then
            centerY = -dimensions.height * (descriptor.yFactor or 0.30)
        elseif surfaceBand == MountSurfaceBand.Side then
            centerY = dimensions.height * (descriptor.yFactor or 0)
            halfX = descriptor.padHalfWidth or dimensions.beam * 0.10
            halfY = descriptor.padHalfHeight or dimensions.height * 0.16
        else
            centerY = dimensions.height * (descriptor.yFactor or 0.30)
        end

        for _, side in ipairs({ MountSide.Port, MountSide.Starboard }) do
            local sideSign = side == MountSide.Port and -1 or 1
            local x = sideSign * centerX
            addTaperedBox(
                mesh,
                { x = x, y = centerY, z = z },
                halfX,
                halfY,
                halfZ,
                descriptor.aftScale or 0.96,
                descriptor.foreScale or 0.96)

            local normal = normalForSurface(surfaceBand, sideSign)
            local surfaceX = x
            local surfaceY = centerY
            if surfaceBand == MountSurfaceBand.Side then
                surfaceX = x + sideSign * halfX * (descriptor.foreScale or 0.96)
            elseif surfaceBand == MountSurfaceBand.Bottom
                or surfaceBand == MountSurfaceBand.Ventral
                or surfaceBand == MountSurfaceBand.Underside
            then
                surfaceY = centerY - halfY * (descriptor.aftScale or 0.96)
            else
                surfaceY = centerY + halfY * (descriptor.foreScale or 0.96)
            end

            local footprint = descriptor.footprintRadius
                or math.min(surfaceBand == MountSurfaceBand.Side and halfY or halfX, halfZ) * 0.72
            table.insert(sockets, {
                socketId = socketIdFor(descriptor, side, index),
                pairId = descriptor.pairId or descriptor.socketId or ("socket_" .. index),
                zone = descriptor.zone,
                side = side,
                surfaceBand = surfaceBand,
                mountSizeClass = descriptor.mountSizeClass,
                mountRole = descriptor.mountRole,
                allowedSizeClasses = descriptor.allowedSizeClasses
                    and copyTable(descriptor.allowedSizeClasses)
                    or nil,
                localPosition = Position(surfaceX, surfaceY, z),
                surfaceNormal = normal,
                footprintRadius = footprint,
                forwardClearance = descriptor.forwardClearance
                    or dimensions.length * 0.08,
                clearanceHeight = descriptor.clearanceHeight
                    or math.max(dimensions.height * 0.035, 0.02),
            })
        end
    end

    return sockets
end

local function addLayeredStructure(mesh, dimensions, layers, rng)
    local length = dimensions.length
    local beam = dimensions.beam
    local height = dimensions.height
    local detailDensity = bounded(layers.detailDensity, 0, 1, 0.15)
    local variation = bounded(layers.variation, 0, 0.20, 0.08)

    addTaperedBox(
        mesh,
        { x = 0, y = height * 0.40, z = length * 0.06 },
        beam * 0.24,
        height * 0.18,
        length * 0.18,
        0.78,
        0.92)
    addTaperedBox(
        mesh,
        { x = 0, y = height * 0.66, z = length * 0.10 },
        beam * 0.16,
        height * 0.16,
        length * 0.09,
        0.72,
        0.88)
    addTaperedBox(
        mesh,
        { x = 0, y = -height * 0.43, z = -length * 0.04 },
        beam * 0.22,
        height * 0.16,
        length * 0.22,
        0.90,
        0.74)

    for _, sideSign in ipairs({ -1, 1 }) do
        addTaperedBox(
            mesh,
            {
                x = sideSign * beam * 0.40,
                y = -height * 0.13,
                z = -length * 0.36,
            },
            beam * 0.10,
            height * 0.15,
            length * 0.13,
            0.82,
            0.68)
        addTaperedBox(
            mesh,
            {
                x = sideSign * beam * 0.46,
                y = height * 0.05,
                z = length * 0.01,
            },
            beam * 0.07,
            height * 0.20,
            length * 0.27,
            0.94,
            0.88)
    end

    local detailCount = math.floor(detailDensity * 4 + rng:getUniform() * 2)
    for index = 1, detailCount do
        local z = rng:getUniformRange(-length * 0.28, length * 0.30)
        local scale = 1 + rng:getUniformRange(-variation, variation)
        for _, sideSign in ipairs({ -1, 1 }) do
            addTaperedBox(
                mesh,
                {
                    x = sideSign * beam * (0.47 + 0.02 * index),
                    y = height * 0.18,
                    z = z,
                },
                beam * 0.035 * scale,
                height * 0.07 * scale,
                length * 0.035 * scale,
                1,
                1)
        end
    end
end

---@param seed integer
---@param recipe table|nil
---@return table {mesh, generator, sockets}
function CapitalHullGenerator:generate(seed, recipe)
    assert(seed ~= nil, "capital hull generation requires a seed")
    recipe = recipe or {}
    local dimensionsRecipe = recipe.dimensions or {}
    local dimensions = {
        length = bounded(dimensionsRecipe.length, 2.0, 40.0, 6.0),
        beam = bounded(dimensionsRecipe.beam, 1.0, 20.0, 2.2),
        height = bounded(dimensionsRecipe.height, 0.7, 16.0, 1.4),
    }
    local frame = recipe.frame or {}
    local layers = recipe.layers or {}
    local mountDecks = recipe.mountDecks or {}
    local master = RNG.Create(seed)
    assert(master, "capital hull generation could not create its master RNG")
    local frameRng = childRng(master)
    local layerRng = childRng(master)
    local deckRng = childRng(master)
    local detailRng = childRng(master)

    local mesh = Mesh.Create()
    addCoreHull(mesh, dimensions, frame, frameRng)
    addLayeredStructure(mesh, dimensions, layers, layerRng)
    local sockets = addStructuralSockets(mesh, dimensions, mountDecks)
    if mountDecks.jitter then
        -- Sample once per bilateral pair so mirrored socket capabilities remain equal.
        local jitterByPair = {}
        for _, socket in ipairs(sockets) do
            local pairId = socket.pairId or socket.socketId
            local jitter = jitterByPair[pairId]
            if jitter == nil then
                jitter = deckRng:getUniformRange(-mountDecks.jitter, mountDecks.jitter)
                jitterByPair[pairId] = jitter
            end
            socket.forwardClearance = math.max(0.05, socket.forwardClearance + jitter)
        end
    end
    mesh:computeNormals()

    return {
        mesh = mesh,
        generator = {
            id = GENERATOR_ID,
            version = GENERATOR_VERSION,
            seed = seed,
            dimensions = copyTable(dimensions),
            socketCount = #sockets,
        },
        sockets = sockets,
    }
end

return CapitalHullGenerator
