---@class HullMountDiscovery
---@overload fun(): HullMountDiscovery
local HullMountDiscovery = Class("HullMountDiscovery", function() end)

local DEFAULT_NORMAL = Vec3f(0, 1, 0)
local DEFAULT_FACING = Vec3f(0, 0, 1)

local function copyVec3(value)
    return Vec3f(value.x, value.y, value.z)
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

local function inZone(position, center, radius, zone)
    local normalizedZ = (position.z - center.z) / radius

    if zone == "fore" then
        return normalizedZ >= 0.15
    elseif zone == "aft" then
        return normalizedZ <= -0.15
    elseif zone == "mid" then
        return math.abs(normalizedZ) <= 0.35
    end

    return true
end

local function onSide(position, center, radius, side)
    local normalizedX = (position.x - center.x) / radius

    if side == "port" then
        return normalizedX <= -0.08
    elseif side == "starboard" then
        return normalizedX >= 0.08
    end

    return true
end

local function onStrictSide(position, center, side)
    if side == "port" then
        return position.x < center.x
    elseif side == "starboard" then
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

---@param mesh Mesh Generated hull mesh in local coordinates
---@param seed integer Deterministic discovery seed
---@param mountSpecs table[] Ordered mount descriptors: {mountId, zone, side, normal?, facing?}
---@param options table|nil {maxAttempts, minSpacing, minNormalDot}
---@return table[] mounts
function HullMountDiscovery:discover(mesh, seed, mountSpecs, options)
    assert(mesh, "hull mount discovery requires a generated mesh")
    assert(type(seed) == "number", "hull mount discovery requires a numeric seed")
    assert(type(mountSpecs) == "table" and #mountSpecs > 0,
        "hull mount discovery requires ordered mount specifications")

    options = options or {}
    local maxAttempts = options.maxAttempts or 256
    local minNormalDot = options.minNormalDot or 0.35
    local center = mesh:getCenter()
    local radius = math.max(mesh:getRadius(), 0.0001)
    local minSpacing = options.minSpacing or math.max(radius * 0.08, 0.05)
    local minSpacingSquared = minSpacing * minSpacing
    local mirrorTolerance = options.mirrorTolerance or 0.0001
    local rng = RNG.Create(seed)
    local mounts = {}

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
            zoneMatch = zoneMatch,
            sideMatch = sideMatch,
        }
    end

    local function findPairIndex(index, spec)
        if not spec.pairId then
            return nil
        end
        for otherIndex, otherSpec in ipairs(mountSpecs) do
            if otherIndex ~= index
                and otherSpec.pairId == spec.pairId
                and otherSpec.side ~= spec.side
            then
                return otherIndex
            end
        end
        return nil
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
                local matchesZone = inZone(candidate.position, center, radius, spec.zone)
                local matchesSide = onSide(candidate.position, center, radius, spec.side)
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

                local portIndex = spec.side == "port" and index or pairIndex
                local starboardIndex = spec.side == "starboard" and index or pairIndex
                local portSpec = mountSpecs[portIndex]
                local starboardSpec = mountSpecs[starboardIndex]
                assert(portSpec.side == "port" and starboardSpec.side == "starboard",
                    "paired hull mounts must contain one port and one starboard side")

                local portCandidates = collectSurfaceCandidates(
                    mesh,
                    portSpec.normal or DEFAULT_NORMAL,
                    minNormalDot)
                local starboardCandidates = collectSurfaceCandidates(
                    mesh,
                    starboardSpec.normal or DEFAULT_NORMAL,
                    minNormalDot)
                assert(#portCandidates > 0 and #starboardCandidates > 0,
                    "paired hull discovery requires candidates on both sides")

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
                    local portZoneMatch = inZone(portCandidate.position, center, radius, portSpec.zone)
                    local portSideMatch = onSide(portCandidate.position, center, radius, portSpec.side)
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
                            }
                            local starboardZoneMatch = inZone(
                                starboardMountCandidate.position,
                                center,
                                radius,
                                starboardSpec.zone)
                            local starboardSideMatch = onSide(
                                starboardMountCandidate.position,
                                center,
                                radius,
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
                assert(foundPortZoneMatch and foundStarboardZoneMatch,
                    "mirrored hull mount pair " .. tostring(spec.pairId)
                        .. " does not satisfy its requested zone")
                assert(foundPortSideMatch and foundStarboardSideMatch,
                    "mirrored hull mount pair " .. tostring(spec.pairId)
                        .. " does not satisfy its requested side")

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
                    spec.normal or DEFAULT_NORMAL,
                    minNormalDot)
                assert(#candidates > 0,
                    string.format("hull mesh has no surface candidates for mount %s", spec.mountId))
                local found, foundZoneMatch, foundSideMatch = findUnpairedCandidate(spec, candidates)

                assert(found,
                    string.format(
                        "unable to discover hull mount %s (zone=%s side=%s seed=%d)",
                        spec.mountId,
                        tostring(spec.zone),
                        tostring(spec.side),
                        seed))
                assert(foundZoneMatch and foundSideMatch,
                    "unable to discover hull mount " .. spec.mountId
                        .. " with a matching zone/side candidate")

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
