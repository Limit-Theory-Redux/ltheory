---@class HullMountDiscovery
---@overload fun(): HullMountDiscovery
local HullMountDiscovery = Class("HullMountDiscovery", function() end)

local DEFAULT_NORMAL = Vec3f(0, 1, 0)
local DEFAULT_FACING = Vec3f(0, 0, 1)
local MountSide = Enums.Weapon.MountSide
local MountZone = Enums.Weapon.MountZone
local MountSurfaceBand = Enums.Weapon.MountSurfaceBand

local function copyVec3(value)
    return Vec3f(value.x, value.y, value.z)
end

local function normalForSpec(spec)
    if spec.normal then
        return spec.normal
    end

    local surfaceBand = spec.surfaceBand or MountSurfaceBand.Dorsal
    if surfaceBand == MountSurfaceBand.Ventral
        or surfaceBand == MountSurfaceBand.Bottom
        or surfaceBand == MountSurfaceBand.Underside
    then
        return Vec3f(0, -1, 0)
    end
    if surfaceBand == MountSurfaceBand.Side then
        if spec.side == MountSide.Port then
            return Vec3f(-1, 0, 0)
        elseif spec.side == MountSide.Starboard then
            return Vec3f(1, 0, 0)
        end
    end
    return DEFAULT_NORMAL
end

local function distanceSquared(a, b)
    local dx = a.x - b.x
    local dy = a.y - b.y
    local dz = a.z - b.z
    return dx * dx + dy * dy + dz * dz
end

local function mirrorAcrossX(position, center)
    return Vec3f(
        2 * center.x - position.x,
        position.y,
        position.z)
end

local function normalsAreMirrored(first, second, tolerance)
    return math.abs(first.x + second.x) <= tolerance
        and math.abs(first.y - second.y) <= tolerance
        and math.abs(first.z - second.z) <= tolerance
end

local function pointOnTriangle(point, triangle, tolerance)
    local a = triangle[1]
    local b = triangle[2]
    local c = triangle[3]
    local abx = b.x - a.x
    local aby = b.y - a.y
    local abz = b.z - a.z
    local acx = c.x - a.x
    local acy = c.y - a.y
    local acz = c.z - a.z
    local nx = aby * acz - abz * acy
    local ny = abz * acx - abx * acz
    local nz = abx * acy - aby * acx
    local normalLength = math.sqrt(nx * nx + ny * ny + nz * nz)
    if normalLength <= 0.000001 then
        return false
    end

    local apx = point.x - a.x
    local apy = point.y - a.y
    local apz = point.z - a.z
    local planeDistance = math.abs(apx * nx + apy * ny + apz * nz) / normalLength
    if planeDistance > tolerance then
        return false
    end

    local d00 = abx * abx + aby * aby + abz * abz
    local d01 = abx * acx + aby * acy + abz * acz
    local d11 = acx * acx + acy * acy + acz * acz
    local d20 = apx * abx + apy * aby + apz * abz
    local d21 = apx * acx + apy * acy + apz * acz
    local denominator = d00 * d11 - d01 * d01
    if math.abs(denominator) <= 0.000001 then
        return false
    end

    local v = (d11 * d20 - d01 * d21) / denominator
    local w = (d00 * d21 - d01 * d20) / denominator
    local u = 1 - v - w
    local barycentricTolerance = 0.0001
    return u >= -barycentricTolerance
        and v >= -barycentricTolerance
        and w >= -barycentricTolerance
end

local function findMirroredSurfaceCandidate(portCandidate, starboardCandidates, mirroredPosition, tolerance)
    for _, candidate in ipairs(starboardCandidates) do
        if normalsAreMirrored(portCandidate.surfaceNormal, candidate.surfaceNormal, tolerance)
            and pointOnTriangle(mirroredPosition, candidate.triangle, tolerance)
        then
            return candidate
        end
    end
    return nil
end

local function inZone(position, center, longitudinalRadius, zone)
    local normalizedZ = (position.z - center.z) / longitudinalRadius

    if zone == MountZone.Fore then
        return normalizedZ >= 0.15
    elseif zone == MountZone.Aft then
        return normalizedZ <= -0.15
    elseif zone == MountZone.Mid then
        return math.abs(normalizedZ) <= 0.35
    end

    return true
end

local function longitudinalRadiusForMesh(mesh)
    local minimumZ = math.huge
    local maximumZ = -math.huge
    for index = 0, mesh:getVertexCount() - 1 do
        local vertex = mesh:getVertex(index)
        minimumZ = math.min(minimumZ, vertex.pz)
        maximumZ = math.max(maximumZ, vertex.pz)
    end
    return math.max((maximumZ - minimumZ) * 0.5, 0.0001)
end

local function transverseRadiusForMesh(mesh)
    local minimumX = math.huge
    local maximumX = -math.huge
    for index = 0, mesh:getVertexCount() - 1 do
        local vertex = mesh:getVertex(index)
        minimumX = math.min(minimumX, vertex.px)
        maximumX = math.max(maximumX, vertex.px)
    end
    return math.max((maximumX - minimumX) * 0.5, 0.0001)
end

local function onSide(position, center, transverseRadius, side)
    local normalizedX = (position.x - center.x) / transverseRadius

    if side == MountSide.Port then
        return normalizedX <= -0.08
    elseif side == MountSide.Starboard then
        return normalizedX >= 0.08
    end

    return true
end

local function onStrictSide(position, center, side)
    if side == MountSide.Port then
        return position.x < center.x
    elseif side == MountSide.Starboard then
        return position.x > center.x
    end

    return true
end

local function collectSurfaceCandidates(mesh, desiredNormal, minNormalDot)
    local desiredLength = math.sqrt(
        desiredNormal.x * desiredNormal.x
            + desiredNormal.y * desiredNormal.y
            + desiredNormal.z * desiredNormal.z)
    assert(desiredLength > 0.000001, "hull mount discovery requires a non-zero surface normal")

    local vertices = {}
    for index = 0, mesh:getVertexCount() - 1 do
        local vertex = mesh:getVertex(index)
        vertices[index] = {
            x = vertex.px,
            y = vertex.py,
            z = vertex.pz,
        }
    end

    local candidates = {}
    local indexCount = mesh:getIndexCount()
    mesh:lockIndexData(function(indices)
        for index = 0, indexCount - 3, 3 do
            local p0 = vertices[tonumber(indices[index])]
            local p1 = vertices[tonumber(indices[index + 1])]
            local p2 = vertices[tonumber(indices[index + 2])]
            if p0 and p1 and p2 then
                local e1x = p1.x - p0.x
                local e1y = p1.y - p0.y
                local e1z = p1.z - p0.z
                local e2x = p2.x - p0.x
                local e2y = p2.y - p0.y
                local e2z = p2.z - p0.z
                local nx = e1y * e2z - e1z * e2y
                local ny = e1z * e2x - e1x * e2z
                local nz = e1x * e2y - e1y * e2x
                local normalLength = math.sqrt(nx * nx + ny * ny + nz * nz)
                if normalLength > 0.000001 then
                    local dot = (nx * desiredNormal.x + ny * desiredNormal.y + nz * desiredNormal.z)
                        / (normalLength * desiredLength)
                    if dot >= minNormalDot then
                        table.insert(candidates, {
                            position = Vec3f(
                                (p0.x + p1.x + p2.x) / 3.0,
                                (p0.y + p1.y + p2.y) / 3.0,
                                (p0.z + p1.z + p2.z) / 3.0),
                            surfaceNormal = Vec3f(
                                nx / normalLength,
                                ny / normalLength,
                                nz / normalLength),
                            triangle = {
                                Vec3f(p0.x, p0.y, p0.z),
                                Vec3f(p1.x, p1.y, p1.z),
                                Vec3f(p2.x, p2.y, p2.z),
                            },
                        })
                    end
                end
            end
        end
    end)

    return candidates
end

local function findStructuralSocket(structuralSockets, spec, strict)
    if type(structuralSockets) ~= "table" then
        return nil
    end
    local requestedSocketId = spec.socketId
    if strict and not requestedSocketId and spec.pairId then
        requestedSocketId = spec.pairId .. "_" .. tostring(spec.side)
    end
    for _, socket in ipairs(structuralSockets) do
        local sameIdentity
        if strict then
            sameIdentity = requestedSocketId ~= nil
                and socket.socketId == requestedSocketId
        else
            sameIdentity = (spec.socketId ~= nil
                    and socket.socketId == spec.socketId)
                or (spec.pairId ~= nil
                    and socket.socketId == spec.pairId)
                or (spec.pairId ~= nil
                    and socket.pairId == spec.pairId)
        end
        if sameIdentity and socket.side == spec.side then
            return socket
        end
    end
    return nil
end

local function validateStructuralSocketRecords(structuralSockets)
    assert(type(structuralSockets) == "table",
        "strict structural discovery requires structural socket records")
    local socketIds = {}
    local pairSides = {}
    for index, socket in ipairs(structuralSockets) do
        assert(type(socket) == "table",
            "strict structural socket record " .. tostring(index) .. " must be a table")
        assert(type(socket.socketId) == "string" and #socket.socketId > 0,
            "strict structural socket record " .. tostring(index)
                .. " requires a non-empty socketId")
        assert(not socketIds[socket.socketId],
            "strict structural socket records must not duplicate socketId " .. socket.socketId)
        socketIds[socket.socketId] = true
        assert(socket.side == MountSide.Port or socket.side == MountSide.Starboard,
            "strict structural socket record " .. tostring(index)
                .. " requires a port or starboard side")
        if socket.pairId ~= nil then
            local pairSide = tostring(socket.pairId) .. "|" .. socket.side
            assert(not pairSides[pairSide],
                "strict structural socket records must not duplicate pair side " .. pairSide)
            pairSides[pairSide] = true
        end
    end
end

local function validatePairSpecifications(mountSpecs, requireStructuralSockets)
    local membersByPair = {}
    for index, spec in ipairs(mountSpecs) do
        if requireStructuralSockets then
            assert((type(spec.socketId) == "string" and #spec.socketId > 0)
                or (type(spec.pairId) == "string" and #spec.pairId > 0),
                "strict structural hull mount " .. tostring(spec.mountId)
                    .. " requires a non-empty socketId or pairId")
            assert(spec.side == MountSide.Port or spec.side == MountSide.Starboard,
                "strict structural hull mount " .. tostring(spec.mountId)
                    .. " requires a port or starboard side")
        end
        if spec.pairId then
            assert(spec.side == MountSide.Port or spec.side == MountSide.Starboard,
                "paired hull mount " .. tostring(spec.pairId)
                    .. " requires a port or starboard side")
            local members = membersByPair[spec.pairId]
            if not members then
                members = {}
                membersByPair[spec.pairId] = members
            end
            table.insert(members, { index = index, spec = spec })
        end
    end

    local pairIndexByIndex = {}
    for pairId, members in pairs(membersByPair) do
        assert(#members == 2,
            "paired hull mount " .. tostring(pairId)
                .. " requires exactly one port and one starboard specification")
        local first = members[1]
        local second = members[2]
        local port = first.spec.side == MountSide.Port and first or second
        local starboard = first.spec.side == MountSide.Starboard and first or second
        assert(port.spec.side == MountSide.Port and starboard.spec.side == MountSide.Starboard,
            "paired hull mount " .. tostring(pairId)
                .. " requires one port and one starboard specification")
        assert(port.spec.zone == starboard.spec.zone,
            "paired hull mount " .. tostring(pairId)
                .. " must use the same requested zone on both sides")
        if requireStructuralSockets then
            assert(port.spec.zone ~= nil,
                "strict paired hull mount " .. tostring(pairId)
                    .. " requires a requested zone")
            assert(port.spec.surfaceBand ~= nil and starboard.spec.surfaceBand ~= nil,
                "strict paired hull mount " .. tostring(pairId)
                    .. " requires a surface band on both sides")
            assert(port.spec.surfaceBand == starboard.spec.surfaceBand,
                "strict paired hull mount " .. tostring(pairId)
                    .. " must use the same requested surface band on both sides")
        else
            assert(port.spec.surfaceBand == nil
                or starboard.spec.surfaceBand == nil
                or port.spec.surfaceBand == starboard.spec.surfaceBand,
                "paired hull mount " .. tostring(pairId)
                    .. " must use the same requested surface band on both sides")
        end
        pairIndexByIndex[port.index] = starboard.index
        pairIndexByIndex[starboard.index] = port.index
    end
    return pairIndexByIndex
end

local function assertStructuralSocketContract(socket, spec, label, strict)
    local identity = spec.socketId
    if not identity and spec.pairId then
        identity = spec.pairId .. "_" .. tostring(spec.side)
    end
    if not strict then
        assert(socket.side == spec.side,
            "structural " .. label .. " socket must match requested side for " .. tostring(identity))
        return
    end
    if spec.pairId then
        assert(socket.pairId == spec.pairId,
            "structural " .. label .. " socket must share pairId " .. tostring(spec.pairId))
    else
        assert(socket.socketId == spec.socketId,
            "structural " .. label .. " socket must match socketId " .. tostring(spec.socketId))
    end
    assert(socket.side == spec.side,
        "structural " .. label .. " socket must match requested side for " .. tostring(identity))
    assert(socket.socketId == identity,
        "strict structural " .. label .. " socket must match socketId " .. tostring(identity))
    assert(spec.zone ~= nil and socket.zone ~= nil and socket.zone == spec.zone,
        "strict structural " .. label .. " socket must match requested zone for " .. tostring(identity))
    assert(spec.surfaceBand ~= nil and socket.surfaceBand ~= nil
        and socket.surfaceBand == spec.surfaceBand,
        "strict structural " .. label .. " socket must match requested surface band for " .. tostring(identity))
end

local function assertStructuralSocketGeometry(
    socket,
    label,
    minimumFootprintRadius,
    minimumForwardClearance,
    strict)
    if strict then
        assert(type(socket.footprintRadius) == "number"
                and socket.footprintRadius >= minimumFootprintRadius,
            "strict structural " .. label .. " socket requires sufficient footprint")
        assert(type(socket.forwardClearance) == "number"
                and socket.forwardClearance >= minimumForwardClearance,
            "strict structural " .. label .. " socket requires sufficient clearance")
    else
        assert(socket.footprintRadius == nil
                or socket.footprintRadius >= minimumFootprintRadius,
            "structural " .. label .. " socket has insufficient footprint")
        assert(socket.forwardClearance == nil
                or socket.forwardClearance >= minimumForwardClearance,
            "structural " .. label .. " socket has insufficient clearance")
    end
end

local function findSocketCandidate(socket, candidates, tolerance)
    if not socket or not socket.localPosition then
        return nil
    end
    for _, candidate in ipairs(candidates) do
        if pointOnTriangle(socket.localPosition, candidate.triangle, tolerance) then
            return {
                position = copyVec3(socket.localPosition),
                surfaceNormal = copyVec3(candidate.surfaceNormal),
                triangle = candidate.triangle,
                footprintRadius = socket.footprintRadius,
                forwardClearance = socket.forwardClearance,
                socketId = socket.socketId,
            }
        end
    end
    return nil
end

local function deriveMountSpecs(structuralSockets)
    assert(type(structuralSockets) == "table" and #structuralSockets > 0,
        "mount discovery requires structural sockets when no mount specs are supplied")
    local mountSpecs = {}
    for index, socket in ipairs(structuralSockets) do
        assert(type(socket.socketId) == "string" and #socket.socketId > 0,
            "structural socket " .. tostring(index) .. " requires a socketId")
        mountSpecs[index] = {
            mountId = socket.socketId,
            socketId = socket.socketId,
            pairId = socket.pairId,
            zone = socket.zone,
            side = socket.side,
            surfaceBand = socket.surfaceBand,
            normal = socket.surfaceNormal,
            mountSizeClass = socket.mountSizeClass,
            allowedSizeClasses = socket.allowedSizeClasses,
            mountRole = socket.mountRole,
            arc = socket.arc,
            facing = socket.facing,
            localRotation = socket.localRotation,
        }
    end
    return mountSpecs
end

---@param mesh Mesh Generated hull mesh in local coordinates
---@param seed integer Deterministic discovery seed
---@param mountSpecs table[] Ordered mount descriptors: {mountId, zone, side, normal?, facing?}
---@param options table|nil {maxAttempts, minSpacing, minNormalDot, structuralSockets, requireStructuralSockets, enforceZoneSide}
---@return table[] mounts
function HullMountDiscovery:discover(mesh, seed, mountSpecs, options)
    assert(mesh, "hull mount discovery requires a generated mesh")
    assert(seed ~= nil, "hull mount discovery requires a seed")

    options = options or {}
    if type(mountSpecs) ~= "table" or #mountSpecs == 0 then
        mountSpecs = deriveMountSpecs(options.structuralSockets)
    end
    assert(#mountSpecs > 0,
        "hull mount discovery requires ordered mount specifications")
    local maxAttempts = options.maxAttempts or 256
    local minNormalDot = options.minNormalDot or 0.35
    local center = mesh:getCenter()
    local transverseRadius = transverseRadiusForMesh(mesh)
    local longitudinalRadius = longitudinalRadiusForMesh(mesh)
    local minimumFootprintRadius = options.minimumFootprintRadius or 0.025
    local minSpacing = options.minSpacing or math.max(
        transverseRadius * 0.08,
        minimumFootprintRadius * 2,
        0.05)
    local minSpacingSquared = minSpacing * minSpacing
    local mirrorTolerance = options.mirrorTolerance or 0.0001
    local socketTolerance = options.socketTolerance or 0.001
    local minimumForwardClearance = options.minimumForwardClearance or 0.05
    local enforceZoneSide = options.enforceZoneSide ~= false
    local requireStructuralSockets = options.requireStructuralSockets == true
    local rng = RNG.Create(seed)
    local mounts = {}
    if requireStructuralSockets then
        validateStructuralSocketRecords(options.structuralSockets)
    end
    local pairIndexByIndex = validatePairSpecifications(mountSpecs, requireStructuralSockets)

    local function isSeparated(position)
        for _, previous in pairs(mounts) do
            if previous and distanceSquared(position, previous.localPosition) < minSpacingSquared then
                return false
            end
        end
        return true
    end

    local function createMount(spec, candidate, zoneMatch, sideMatch)
        local facing = spec.facing or DEFAULT_FACING
        return {
            mountId = spec.mountId,
            localPosition = Position(
                candidate.position.x,
                candidate.position.y,
                candidate.position.z),
            surfaceNormal = copyVec3(candidate.surfaceNormal),
            facing = copyVec3(facing),
            localRotation = spec.localRotation
                or Quat.FromLookUp(facing, candidate.surfaceNormal),
            zone = spec.zone,
            side = spec.side,
            pairId = spec.pairId,
            socketId = candidate.socketId or spec.socketId or spec.pairId,
            mountSizeClass = spec.mountSizeClass,
            allowedSizeClasses = spec.allowedSizeClasses,
            mountRole = spec.mountRole,
            surfaceBand = spec.surfaceBand,
            arc = spec.arc,
            footprintRadius = candidate.footprintRadius,
            forwardClearance = candidate.forwardClearance,
            zoneMatch = zoneMatch,
            sideMatch = sideMatch,
        }
    end

    local function findPairIndex(index, spec)
        if not spec.pairId then
            return nil
        end
        return pairIndexByIndex[index]
    end

    local function findUnpairedCandidate(spec, candidates)
        local start = rng:getInt(1, #candidates)
        local bestScore = -math.huge
        local found = nil
        local foundZoneMatch = false
        local foundSideMatch = false
        for offset = 0, math.min(maxAttempts, #candidates) - 1 do
            local candidate = candidates[((start + offset - 1) % #candidates) + 1]
            if isSeparated(candidate.position) then
                local matchesZone = inZone(candidate.position, center, longitudinalRadius, spec.zone)
                local matchesSide = onSide(candidate.position, center, transverseRadius, spec.side)
                local score = (matchesSide and 4 or 0) + (matchesZone and 2 or 0)
                if score > bestScore then
                    bestScore = score
                    found = candidate
                    foundZoneMatch = matchesZone
                    foundSideMatch = matchesSide
                end
            end
        end
        return found, foundZoneMatch, foundSideMatch
    end

    local processed = {}
    for index, spec in ipairs(mountSpecs) do
        if not processed[index] then
            assert(type(spec.mountId) == "string" and #spec.mountId > 0,
                "hull mount specification " .. tostring(index) .. " requires a mountId")

            local pairIndex = findPairIndex(index, spec)
            if pairIndex then
                local pairSpec = mountSpecs[pairIndex]
                assert(pairSpec.pairId == spec.pairId,
                    "paired hull mount specifications must share a pairId")

                local portIndex = spec.side == MountSide.Port and index or pairIndex
                local starboardIndex = spec.side == MountSide.Starboard and index or pairIndex
                local portSpec = mountSpecs[portIndex]
                local starboardSpec = mountSpecs[starboardIndex]
                assert(portSpec.side == MountSide.Port
                    and starboardSpec.side == MountSide.Starboard,
                    "paired hull mounts must contain one port and one starboard side")

                local portCandidates = collectSurfaceCandidates(
                    mesh,
                    normalForSpec(portSpec),
                    minNormalDot)
                local starboardCandidates = collectSurfaceCandidates(
                    mesh,
                    normalForSpec(starboardSpec),
                    minNormalDot)
                assert(#portCandidates > 0 and #starboardCandidates > 0,
                    "paired hull discovery requires candidates on both sides")

                local portSocket = findStructuralSocket(
                    options.structuralSockets,
                    portSpec,
                    requireStructuralSockets)
                local starboardSocket = findStructuralSocket(
                    options.structuralSockets,
                    starboardSpec,
                    requireStructuralSockets)
                if requireStructuralSockets then
                    assert(portSocket and starboardSocket,
                        "strict structural hull mount pair " .. tostring(spec.pairId)
                            .. " requires both port and starboard sockets")
                elseif portSocket or starboardSocket then
                    assert(portSocket and starboardSocket,
                        "structural hull mount pair " .. tostring(spec.pairId)
                            .. " requires both port and starboard sockets")
                end
                if portSocket and starboardSocket then
                    assertStructuralSocketContract(
                        portSocket,
                        portSpec,
                        "port",
                        requireStructuralSockets)
                    assertStructuralSocketContract(
                        starboardSocket,
                        starboardSpec,
                        "starboard",
                        requireStructuralSockets)
                    assert(portSocket.pairId == starboardSocket.pairId,
                        "structural hull mount pair sockets must share a pairId")
                    assert(portSocket.surfaceBand == nil
                        or starboardSocket.surfaceBand == nil
                        or portSocket.surfaceBand == starboardSocket.surfaceBand,
                        "structural hull mount pair sockets must share a surface band")
                    assertStructuralSocketGeometry(
                        portSocket,
                        "port",
                        minimumFootprintRadius,
                        minimumForwardClearance,
                        requireStructuralSockets)
                    assertStructuralSocketGeometry(
                        starboardSocket,
                        "starboard",
                        minimumFootprintRadius,
                        minimumForwardClearance,
                        requireStructuralSockets)
                    if requireStructuralSockets then
                        assert(math.abs(portSocket.footprintRadius - starboardSocket.footprintRadius)
                                <= mirrorTolerance,
                            "strict structural pair sockets must share footprint metadata")
                        assert(math.abs(portSocket.forwardClearance - starboardSocket.forwardClearance)
                                <= mirrorTolerance,
                            "strict structural pair sockets must share clearance metadata")
                    end
                    local structuralPortCandidate = findSocketCandidate(
                        portSocket,
                        portCandidates,
                        socketTolerance)
                    local structuralStarboardCandidate = findSocketCandidate(
                        starboardSocket,
                        starboardCandidates,
                        socketTolerance)
                    assert(structuralPortCandidate and structuralStarboardCandidate,
                        "structural hull mount sockets are not attached to matching mesh triangles: "
                            .. tostring(spec.pairId))
                    local mirroredSocketPosition = mirrorAcrossX(
                        structuralPortCandidate.position,
                        center)
                    assert(distanceSquared(
                            mirroredSocketPosition,
                            structuralStarboardCandidate.position)
                            <= mirrorTolerance * mirrorTolerance,
                        "structural hull mount sockets are not mirrored: " .. tostring(spec.pairId))
                    portCandidates = { structuralPortCandidate }
                    starboardCandidates = { structuralStarboardCandidate }
                end

                local start = rng:getInt(1, #portCandidates)
                local foundPort = nil
                local foundStarboard = nil
                local foundPortZoneMatch = false
                local foundPortSideMatch = false
                local foundStarboardZoneMatch = false
                local foundStarboardSideMatch = false
                local bestScore = -math.huge
                for offset = 0, #portCandidates - 1 do
                    local portCandidate = portCandidates[((start + offset - 1) % #portCandidates) + 1]
                    local portZoneMatch = inZone(
                        portCandidate.position,
                        center,
                        longitudinalRadius,
                        portSpec.zone)
                    local portSideMatch = onSide(portCandidate.position, center, transverseRadius, portSpec.side)
                    if isSeparated(portCandidate.position)
                        and onStrictSide(portCandidate.position, center, portSpec.side)
                    then
                        local mirroredPosition = mirrorAcrossX(portCandidate.position, center)
                        local starboardCandidate = findMirroredSurfaceCandidate(
                            portCandidate,
                            starboardCandidates,
                            mirroredPosition,
                            mirrorTolerance)
                        if starboardCandidate then
                            local starboardMountCandidate = {
                                position = copyVec3(mirroredPosition),
                                surfaceNormal = copyVec3(starboardCandidate.surfaceNormal),
                                footprintRadius = starboardCandidate.footprintRadius,
                                forwardClearance = starboardCandidate.forwardClearance,
                                socketId = starboardCandidate.socketId,
                            }
                            local starboardZoneMatch = inZone(
                                starboardMountCandidate.position,
                                center,
                                longitudinalRadius,
                                starboardSpec.zone)
                            local starboardSideMatch = onSide(
                                starboardMountCandidate.position,
                                center,
                                transverseRadius,
                                starboardSpec.side)
                            if onStrictSide(
                                    starboardMountCandidate.position,
                                    center,
                                    starboardSpec.side)
                                and isSeparated(starboardMountCandidate.position)
                                and distanceSquared(
                                    portCandidate.position,
                                    starboardMountCandidate.position)
                                    >= minSpacingSquared
                            then
                                local score = (portSideMatch and 4 or 0)
                                    + (portZoneMatch and 2 or 0)
                                    + (starboardSideMatch and 4 or 0)
                                    + (starboardZoneMatch and 2 or 0)
                                if score > bestScore then
                                    bestScore = score
                                    foundPort = portCandidate
                                    foundStarboard = starboardMountCandidate
                                    foundPortZoneMatch = portZoneMatch
                                    foundPortSideMatch = portSideMatch
                                    foundStarboardZoneMatch = starboardZoneMatch
                                    foundStarboardSideMatch = starboardSideMatch
                                end
                            end
                        end
                    end
                end

                assert(foundPort and foundStarboard,
                    string.format(
                        "unable to discover mirrored hull mount pair %s (seed=%d tolerance=%.6f candidates=%d/%d)",
                        tostring(spec.pairId),
                        seed,
                        mirrorTolerance,
                        #portCandidates,
                        #starboardCandidates))
                if enforceZoneSide then
                    assert(foundPortZoneMatch and foundStarboardZoneMatch,
                        string.format(
                            "mirrored hull mount pair %s does not satisfy its requested zone "
                                .. "(port=%s starboard=%s portZ=%.6f starboardZ=%.6f centerZ=%.6f transverseRadius=%.6f)",
                            tostring(spec.pairId),
                            tostring(foundPortZoneMatch),
                            tostring(foundStarboardZoneMatch),
                            foundPort.position.z,
                            foundStarboard.position.z,
                            center.z,
                            transverseRadius))
                    assert(foundPortSideMatch and foundStarboardSideMatch,
                        "mirrored hull mount pair " .. tostring(spec.pairId)
                            .. " does not satisfy its requested side")
                end

                mounts[portIndex] = createMount(
                    portSpec,
                    foundPort,
                    foundPortZoneMatch,
                    foundPortSideMatch)
                mounts[starboardIndex] = createMount(
                    starboardSpec,
                    foundStarboard,
                    foundStarboardZoneMatch,
                    foundStarboardSideMatch)
                processed[portIndex] = true
                processed[starboardIndex] = true
            else
                local candidates = collectSurfaceCandidates(
                    mesh,
                    normalForSpec(spec),
                    minNormalDot)
                assert(#candidates > 0,
                    string.format("hull mesh has no surface candidates for mount %s", spec.mountId))

                local structuralSocket = findStructuralSocket(
                    options.structuralSockets,
                    spec,
                    requireStructuralSockets)
                if requireStructuralSockets then
                    assert(structuralSocket,
                        "strict structural hull mount " .. tostring(spec.mountId)
                            .. " requires its declared structural socket")
                end
                local found
                local foundZoneMatch
                local foundSideMatch
                if structuralSocket then
                    assertStructuralSocketContract(
                        structuralSocket,
                        spec,
                        "unpaired",
                        requireStructuralSockets)
                    assertStructuralSocketGeometry(
                        structuralSocket,
                        "unpaired",
                        minimumFootprintRadius,
                        minimumForwardClearance,
                        requireStructuralSockets)
                    found = findSocketCandidate(structuralSocket, candidates, socketTolerance)
                    assert(found,
                        "structural hull mount socket is not attached to a matching mesh triangle: "
                            .. tostring(spec.mountId))
                    foundZoneMatch = inZone(
                        found.position,
                        center,
                        longitudinalRadius,
                        spec.zone)
                    foundSideMatch = onSide(found.position, center, transverseRadius, spec.side)
                else
                    found, foundZoneMatch, foundSideMatch = findUnpairedCandidate(spec, candidates)
                end

                assert(found,
                    string.format(
                        "unable to discover hull mount %s (zone=%s side=%s seed=%d)",
                        spec.mountId,
                        tostring(spec.zone),
                        tostring(spec.side),
                        seed))
                if enforceZoneSide then
                    assert(foundZoneMatch and foundSideMatch,
                        "unable to discover hull mount " .. spec.mountId
                            .. " with a matching zone/side candidate")
                end

                mounts[index] = createMount(spec, found, foundZoneMatch, foundSideMatch)
                processed[index] = true
            end
        end
    end

    local orderedMounts = {}
    for index = 1, #mountSpecs do
        assert(mounts[index], "hull discovery did not produce mount " .. tostring(index))
        orderedMounts[index] = mounts[index]
    end
    return orderedMounts
end

return HullMountDiscovery()
