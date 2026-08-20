---@class WeaponSystem
---@overload fun(): WeaponSystem
local WeaponSystem = Class("WeaponSystem", function() end)

local ffi = require("ffi")
local WeaponRegistry = require("Shared.Registries.WeaponRegistry")
local WeaponResolver = require("Shared.Content.WeaponResolver")
local PointLightEffectEntity = require("Modules.Constructs.Entities.PointLightEffectEntity")
local PhysicsComponents = require("Modules.Physics.Components")
local RaycastHelper = require("Shared.Helpers.RaycastHelper")
local BeamAimHelper = require("Shared.Helpers.BeamAimHelper")
local WeaponMountSizing = require("Shared.Helpers.WeaponMountSizing")
local NULL_RIGID_BODY = ffi.cast("RigidBody*", nil)
local ROOT_EPSILON = 0.000001

local function clamp(value, minimum, maximum)
    return math.max(minimum, math.min(maximum, value))
end

---@param turret table
---@return table
function WeaponSystem:resolveWeapon(turret)
    assert(turret, "weapon resolution requires a turret")
    local weapon = WeaponResolver:resolve({
        weaponId = turret.weaponId,
        weaponRef = turret.weaponRef,
    })
    if turret.weaponRef and not weapon then
        error("unregistered procedural weapon: "
            .. tostring(turret.weaponRef.canonicalKey), 0)
    end
    assert(weapon, "unregistered weapon: " .. tostring(turret.weaponId))
    return weapon
end

local function wrapAngle(angle)
    while angle > math.pi do
        angle = angle - 2 * math.pi
    end
    while angle < -math.pi do
        angle = angle + 2 * math.pi
    end
    return angle
end

local function stepAngle(current, target, maximumDelta)
    local delta = wrapAngle(target - current)
    if math.abs(delta) <= maximumDelta then
        return target
    end
    return current + (delta > 0 and maximumDelta or -maximumDelta)
end

local function getRayHitPosition(hit, hitBody)
    if hit and hit.position then
        return hit.position
    end
    if hit and hit.posx ~= nil and hit.posy ~= nil and hit.posz ~= nil then
        return Vec3f(hit.posx, hit.posy, hit.posz)
    end
    if hitBody and hitBody.getPos then
        return hitBody:getPos()
    end
    return nil
end

local function triangleData(p0, p1, p2)
    local e1x = p1.x - p0.x
    local e1y = p1.y - p0.y
    local e1z = p1.z - p0.z
    local e2x = p2.x - p0.x
    local e2y = p2.y - p0.y
    local e2z = p2.z - p0.z
    local nx = e1y * e2z - e1z * e2y
    local ny = e1z * e2x - e1x * e2z
    local nz = e1x * e2y - e1y * e2x
    local doubleArea = math.sqrt(nx * nx + ny * ny + nz * nz)
    if doubleArea <= ROOT_EPSILON then
        return nil
    end

    return {
        p0 = Vec3f(p0.x, p0.y, p0.z),
        p1 = Vec3f(p1.x, p1.y, p1.z),
        p2 = Vec3f(p2.x, p2.y, p2.z),
        normal = Vec3f(nx / doubleArea, ny / doubleArea, nz / doubleArea),
        area = doubleArea * 0.5,
    }
end

---@param mesh Mesh
---@return table[] surface triangles
function WeaponSystem:buildTargetSurface(mesh)
    assert(mesh and mesh.getVertexCount and mesh.getIndexCount,
        "target surface sampling requires a generated mesh")

    local vertices = {}
    for index = 0, mesh:getVertexCount() - 1 do
        local vertex = mesh:getVertex(index)
        vertices[index] = Vec3f(vertex.px, vertex.py, vertex.pz)
    end

    local surface = {}
    local totalArea = 0
    local indexCount = mesh:getIndexCount()
    mesh:lockIndexData(function(indices)
        for index = 0, indexCount - 3, 3 do
            local p0 = vertices[tonumber(indices[index])]
            local p1 = vertices[tonumber(indices[index + 1])]
            local p2 = vertices[tonumber(indices[index + 2])]
            if p0 and p1 and p2 then
                local triangle = triangleData(p0, p1, p2)
                if triangle then
                    triangle.index = #surface + 1
                    table.insert(surface, triangle)
                    totalArea = totalArea + triangle.area
                end
            end
        end
    end)

    assert(#surface > 0 and totalArea > ROOT_EPSILON,
        "target surface sampling requires non-degenerate mesh triangles")
    surface.totalArea = totalArea
    return surface
end

---@param surface table[] target surface triangles
---@param seed integer deterministic mount seed
---@param time number smooth sample time
---@param options table|nil {motionAmplitude, motionFrequency}
---@return table {position, normal, triangleIndex, barycentric}
function WeaponSystem:sampleTargetPoint(surface, seed, time, options)
    assert(type(surface) == "table" and #surface > 0, "target surface needs triangles")
    options = options or {}
    time = time or 0
    local candidates = surface
    local sampleRng = RNG.Create(seed or 0)
    assert(sampleRng, "target surface sampling could not create an RNG")
    local viewDirection = options.viewDirection
    if viewDirection then
        local minFacingDot = options.minFacingDot or 0
        local facingCandidates = {}
        for _, triangle in ipairs(surface) do
            local dot = triangle.normal.x * viewDirection.x
                + triangle.normal.y * viewDirection.y
                + triangle.normal.z * viewDirection.z
            if dot >= minFacingDot then
                table.insert(facingCandidates, triangle)
            end
        end
        if #facingCandidates > 0 then
            candidates = facingCandidates
        end
    end

    local totalArea = 0
    for _, triangle in ipairs(candidates) do
        totalArea = totalArea + (triangle.area or 1)
    end
    assert(totalArea > 0, "target surface triangles must have positive area")
    local pick = sampleRng:getUniform() * totalArea
    local selected = candidates[#candidates]
    local selectedIndex = #candidates
    local accumulated = 0
    for index, triangle in ipairs(candidates) do
        accumulated = accumulated + (triangle.area or 1)
        if pick <= accumulated then
            selected = triangle
            selectedIndex = index
            break
        end
    end
    local first = sampleRng:getUniform()
    local second = sampleRng:getUniform()
    local root = math.sqrt(first)
    local w0 = 1 - root
    local w1 = root * (1 - second)
    local w2 = root * second
    local amplitude = clamp(options.motionAmplitude or 0.05, 0, 0.25)
    local frequency = options.motionFrequency or 0.5
    local phase = sampleRng:getUniform() * math.pi * 2
    w1 = clamp(w1 + amplitude * math.sin(time * frequency * math.pi * 2 + phase), 0.02, 0.94)
    w2 = clamp(w2 + amplitude * math.cos(time * frequency * math.pi * 1.82 + phase * 1.37), 0.02, 0.94)
    if w1 + w2 > 0.96 then
        local scale = 0.96 / (w1 + w2)
        w1 = w1 * scale
        w2 = w2 * scale
    end
    w0 = 1 - w1 - w2
    local position = Vec3f(
        selected.p0.x * w0 + selected.p1.x * w1 + selected.p2.x * w2,
        selected.p0.y * w0 + selected.p1.y * w1 + selected.p2.y * w2,
        selected.p0.z * w0 + selected.p1.z * w1 + selected.p2.z * w2)
    return {
        position = position,
        normal = selected.normal,
        triangleIndex = selected.index or selectedIndex,
        barycentric = { w0 = w0, w1 = w1, w2 = w2 },
    }
end

---@param sample table target point sample in local target coordinates
---@param targetBody RigidBody
---@return Vec3f world target point
function WeaponSystem:targetPointToWorld(sample, targetBody)
    assert(sample and sample.position and targetBody and targetBody.getPos)
    local scale = targetBody.getScale and targetBody:getScale() or 1
    local localPosition = Vec3f(
        sample.position.x * scale,
        sample.position.y * scale,
        sample.position.z * scale)
    local rotation = targetBody.getRot and targetBody:getRot() or nil
    local rotated = rotation and rotation:mulV(localPosition) or localPosition
    local center = targetBody:getPos()
    return Vec3f(
        center.x + rotated.x,
        center.y + rotated.y,
        center.z + rotated.z)
end

---@param world Physics
---@param sourceBody RigidBody
---@param targetBody RigidBody
---@param sourcePosition Vec3f
---@param targetPosition Vec3f
---@param obstacles table[]|nil
---@return boolean, string, Vec3f|nil
function WeaponSystem:hasLineOfSight(world, sourceBody, targetBody, sourcePosition, targetPosition, obstacles)
    assert(world and sourceBody and targetBody and sourcePosition and targetPosition)

    local dx = targetPosition.x - sourcePosition.x
    local dy = targetPosition.y - sourcePosition.y
    local dz = targetPosition.z - sourcePosition.z
    local distanceSquared = dx * dx + dy * dy + dz * dz
    if distanceSquared < ROOT_EPSILON then
        return true
    end

    local distance = math.sqrt(distanceSquared)
    local invDistance = 1 / distance
    local nx = dx * invDistance
    local ny = dy * invDistance
    local nz = dz * invDistance

    local hit = RaycastHelper:castSegment(
        world,
        sourcePosition,
        targetPosition,
        {
            ignoreBody = sourceBody,
        })
    local hitBody = hit and hit.body
    if hitBody == nil or hitBody == NULL_RIGID_BODY then
        return false, "none"
    end

    if hitBody == sourceBody then
        return false, "source", hit.position
    end

    local hitPosition = hit.position
    if hitBody == targetBody then
        return true, "target", hitPosition
    end

    for _, obstacle in ipairs(obstacles or {}) do
        local obstacleBody = obstacle.body or obstacle
        if hitBody == obstacleBody then
            local hitPosition = getRayHitPosition(hit, hitBody)
            if obstacle.kind == "hull" then
                local center = obstacle.position
                    or (obstacleBody.getPos and obstacleBody:getPos())
                local radius = obstacle.radius
                    or (obstacleBody.getBoundingRadius and obstacleBody:getBoundingRadius())
                if center and radius then
                    local sourceOffsetX = sourcePosition.x - center.x
                    local sourceOffsetY = sourcePosition.y - center.y
                    local sourceOffsetZ = sourcePosition.z - center.z
                    local sourceRadiusSquared = sourceOffsetX * sourceOffsetX
                        + sourceOffsetY * sourceOffsetY
                        + sourceOffsetZ * sourceOffsetZ
                    local outwardDot = sourceOffsetX * nx
                        + sourceOffsetY * ny
                        + sourceOffsetZ * nz
                    local targetOffsetX = targetPosition.x - center.x
                    local targetOffsetY = targetPosition.y - center.y
                    local targetOffsetZ = targetPosition.z - center.z
                    local targetRadiusSquared = targetOffsetX * targetOffsetX
                        + targetOffsetY * targetOffsetY
                        + targetOffsetZ * targetOffsetZ
                    local radiusSquared = radius * radius

                    local hullExitNormal = hit.normal
                    local hullExitDot = hullExitNormal
                        and (hullExitNormal.x * nx
                            + hullExitNormal.y * ny
                            + hullExitNormal.z * nz)
                    local isOutwardHullHit = hullExitDot == nil
                        or hullExitDot > ROOT_EPSILON
                    if sourceRadiusSquared < radiusSquared
                        and targetRadiusSquared > radiusSquared
                        and outwardDot > ROOT_EPSILON
                        and isOutwardHullHit
                        and hit.position
                    then
                        local discriminant = outwardDot * outwardDot
                            + radiusSquared - sourceRadiusSquared
                        local exitDistance = -outwardDot + math.sqrt(math.max(0, discriminant))
                        if exitDistance < distance - ROOT_EPSILON then
                            local continuationHit = RaycastHelper:castSegment(
                                world,
                                hit.position,
                                targetPosition,
                                {
                                    ignoredBodies = { sourceBody, obstacleBody },
                                })
                            if continuationHit then
                                if continuationHit.body == targetBody then
                                    return true, "target", continuationHit.position
                                end
                                for _, continuationObstacle in ipairs(obstacles or {}) do
                                    local continuationObstacleBody = continuationObstacle.body
                                        or continuationObstacle
                                    if continuationHit.body == continuationObstacleBody then
                                        return false,
                                            continuationObstacle.reason or continuationObstacle.kind or "other",
                                            getRayHitPosition(continuationHit, continuationHit.body)
                                    end
                                end
                                return false, "other",
                                    getRayHitPosition(continuationHit, continuationHit.body)
                            end
                        end
                    end
                end
                return false, "hull", hitPosition
            end
            return false, obstacle.reason or "other", hitPosition
        end
    end

    local hitPosition = getRayHitPosition(hit, hitBody)
    return false, "other", hitPosition
end

---@param origin Vec3f
---@param originVelocity Vec3f
---@param targetPosition Vec3f
---@param targetVelocity Vec3f
---@param projectileSpeed number
---@param maxPredictionTime number|nil
---@return table|nil
function WeaponSystem:solveIntercept(
    origin,
    originVelocity,
    targetPosition,
    targetVelocity,
    projectileSpeed,
    maxPredictionTime)
    assert(origin and originVelocity and targetPosition and targetVelocity)
    assert(projectileSpeed and projectileSpeed >= 0)
    assert(maxPredictionTime == nil or maxPredictionTime >= 0)

    local rx = targetPosition.x - origin.x
    local ry = targetPosition.y - origin.y
    local rz = targetPosition.z - origin.z

    local vx = targetVelocity.x - originVelocity.x
    local vy = targetVelocity.y - originVelocity.y
    local vz = targetVelocity.z - originVelocity.z

    local a = vx * vx + vy * vy + vz * vz - projectileSpeed * projectileSpeed
    local b = 2 * (rx * vx + ry * vy + rz * vz)
    local c = rx * rx + ry * ry + rz * rz

    local time

    if math.abs(a) < ROOT_EPSILON then
        if math.abs(b) < ROOT_EPSILON then
            if c < ROOT_EPSILON then
                time = 0
            else
                return nil
            end
        else
            local candidate = -c / b
            if candidate >= 0 then
                time = candidate
            else
                return nil
            end
        end
    else
        local discriminant = b * b - 4 * a * c
        if discriminant < 0 then
            return nil
        end

        local root = math.sqrt(math.max(0, discriminant))
        local first = (-b - root) / (2 * a)
        local second = (-b + root) / (2 * a)

        if first >= 0 then
            time = first
        end
        if second >= 0 and (not time or second < time) then
            time = second
        end
        if not time then
            return nil
        end
    end

    if maxPredictionTime ~= nil and time > maxPredictionTime then
        return nil
    end

    return {
        time = time,
        position = Vec3f(
            targetPosition.x + targetVelocity.x * time,
            targetPosition.y + targetVelocity.y * time,
            targetPosition.z + targetVelocity.z * time
        )
    }
end

---@param origin Vec3f
---@param time number
---@param motion table|nil
---@return table
function WeaponSystem:sampleTargetMotion(origin, time, motion)
    assert(origin and time and motion)
    local position = Vec3f(origin.x, origin.y, origin.z)
    local velocity = Vec3f()
    if motion.enabled == false then
        return { position = position, velocity = velocity }
    end

    if motion.mode == "orbit" then
        local center = motion.center or { x = 0, y = 0, z = 0 }
        local centerX = center.x or 0
        local centerY = center.y or 0
        local centerZ = center.z or 0
        local radiusX = origin.x - centerX
        local radiusY = origin.y - centerY
        local radiusZ = origin.z - centerZ
        local radius = motion.radius or math.sqrt(
            radiusX * radiusX + radiusY * radiusY + radiusZ * radiusZ)
        assert(radius > ROOT_EPSILON, "orbit target motion requires a positive radius")

        local plane = motion.plane or "xz"
        local axisA = { x = 1, y = 0, z = 0 }
        local axisB
        if plane == "xz" or plane == "horizontal" then
            axisB = { x = 0, y = 0, z = 1 }
        elseif plane == "xy" or plane == "vertical" then
            axisB = { x = 0, y = 1, z = 0 }
        elseif plane == "yz" then
            axisA = { x = 0, y = 1, z = 0 }
            axisB = { x = 0, y = 0, z = 1 }
        elseif plane == "tilted" then
            local tilt = motion.tilt or math.rad(35)
            axisB = { x = 0, y = math.cos(tilt), z = math.sin(tilt) }
        else
            error("unsupported target orbit plane: " .. tostring(plane))
        end

        local direction = motion.direction or 1
        assert(direction == 1 or direction == -1,
            "target orbit direction must be 1 or -1")
        local angularSpeed = motion.angularSpeed
        if angularSpeed == nil then
            angularSpeed = (motion.frequency or 0) * 2 * math.pi
        end
        local phase = (motion.phase or 0) + time * angularSpeed * direction
        local cosine = math.cos(phase)
        local sine = math.sin(phase)
        local orbitX = radius * (axisA.x * cosine + axisB.x * sine)
        local orbitY = radius * (axisA.y * cosine + axisB.y * sine)
        local orbitZ = radius * (axisA.z * cosine + axisB.z * sine)
        position.x = centerX + orbitX
        position.y = centerY + orbitY
        position.z = centerZ + orbitZ
        if plane == "xz" or plane == "horizontal" then
            position.y = origin.y
        elseif plane == "xy" or plane == "vertical" then
            position.z = origin.z
        end

        local angularVelocity = angularSpeed * direction
        velocity = Vec3f(
            radius * angularVelocity * (-axisA.x * sine + axisB.x * cosine),
            radius * angularVelocity * (-axisA.y * sine + axisB.y * cosine),
            radius * angularVelocity * (-axisA.z * sine + axisB.z * cosine))
        if plane == "xz" or plane == "horizontal" then
            velocity.y = 0
        elseif plane == "xy" or plane == "vertical" then
            velocity.z = 0
        elseif plane == "yz" then
            velocity.x = 0
        end
        return {
            position = position,
            velocity = velocity,
            phase = phase,
            radius = radius,
            plane = plane,
        }
    end

    local amplitude = math.abs(motion.amplitude or 0)
    local frequency = motion.frequency or 0
    local phase = (motion.phase or 0) + time * 2 * math.pi * frequency
    local offset = amplitude * math.sin(phase)
    local speed = amplitude * 2 * math.pi * frequency * math.cos(phase)
    local axis = motion.axis or "y"
    if axis == "x" then
        position.x = position.x + offset
        velocity.x = speed
    elseif axis == "y" then
        position.y = position.y + offset
        velocity.y = speed
    elseif axis == "z" then
        position.z = position.z + offset
        velocity.z = speed
    else
        error("unsupported target motion axis: " .. tostring(axis))
    end

    return { position = position, velocity = velocity, phase = phase }
end

---@param sourcePosition Vec3f
---@param targetPosition Vec3f
---@param weapon table
---@param seed number
---@return table
function WeaponSystem:applyAccuracy(sourcePosition, targetPosition, weapon, seed)
    assert(sourcePosition and targetPosition and weapon)
    local dx = targetPosition.x - sourcePosition.x
    local dy = targetPosition.y - sourcePosition.y
    local dz = targetPosition.z - sourcePosition.z
    local distanceSquared = dx * dx + dy * dy + dz * dz
    if distanceSquared < ROOT_EPSILON then
        return {
            position = Vec3f(targetPosition.x, targetPosition.y, targetPosition.z),
            offset = Vec3f(),
            angles = Vec3f(),
        }
    end

    local distance = math.sqrt(distanceSquared)
    local invDistance = 1 / distance
    local lookX = dx * invDistance
    local lookY = dy * invDistance
    local lookZ = dz * invDistance
    local rightX
    local rightY
    local rightZ
    if math.abs(lookY) < 0.9 then
        rightX = -lookZ
        rightY = 0
        rightZ = lookX
    else
        rightX = 0
        rightY = lookZ
        rightZ = -lookY
    end
    local rightLength = math.sqrt(rightX * rightX + rightY * rightY + rightZ * rightZ)
    rightX = rightX / rightLength
    rightY = rightY / rightLength
    rightZ = rightZ / rightLength
    local upX = rightY * lookZ - rightZ * lookY
    local upY = rightZ * lookX - rightX * lookZ
    local upZ = rightX * lookY - rightY * lookX

    local accuracy = weapon.accuracy or {}
    local spread = math.max(0, accuracy.spread or 0)
    local trackingJitter = math.max(0, accuracy.trackingJitter or 0)
    local accuracyRng = RNG.Create(seed or 0)
    assert(accuracyRng, "weapon accuracy could not create a deterministic RNG")
    local angleRight = (accuracyRng:getUniform() * 2 - 1) * spread
        + (accuracyRng:getUniform() * 2 - 1) * trackingJitter
    local angleUp = (accuracyRng:getUniform() * 2 - 1) * spread
        + (accuracyRng:getUniform() * 2 - 1) * trackingJitter
    local rightOffset = math.tan(angleRight) * distance
    local upOffset = math.tan(angleUp) * distance
    local offset = Vec3f(
        rightX * rightOffset + upX * upOffset,
        rightY * rightOffset + upY * upOffset,
        rightZ * rightOffset + upZ * upOffset)

    return {
        position = Vec3f(
            targetPosition.x + offset.x,
            targetPosition.y + offset.y,
            targetPosition.z + offset.z),
        offset = offset,
        angles = Vec3f(angleRight, angleUp, 0),
    }
end

local function getCapacitorBanks(capacitor)
    if not capacitor then
        return nil
    end
    if capacitor.getBanks then
        return capacitor:getBanks()
    end
    return capacitor.banks or capacitor
end

local function ensureBankId(bank, declarationIndex)
    if bank.id == nil then
        bank.id = "bank_" .. tostring(declarationIndex)
    else
        bank.id = tostring(bank.id)
    end
    return bank.id
end

local function getReservationLedger(capacitor)
    capacitor.weaponReservationScopes = capacitor.weaponReservationScopes or {}
    return capacitor.weaponReservationScopes
end

local function normalizeReservationOwner(ownerKey)
    if ownerKey == nil then
        return nil
    end
    local normalized = tostring(ownerKey)
    if #normalized == 0 then
        return nil
    end
    return normalized
end

local function reservationStorageKey(requestId, ownerKey)
    local normalizedOwner = normalizeReservationOwner(ownerKey)
    if requestId == nil or normalizedOwner == nil then
        return nil
    end
    local normalizedRequest = tostring(requestId)
    return table.concat({
        tostring(#normalizedOwner), normalizedOwner,
        tostring(#normalizedRequest), normalizedRequest,
    }, ":")
end

local function setReservation(capacitor, storageKey, reservation)
    getReservationLedger(capacitor)[storageKey] = reservation
    if capacitor.setReservation then
        capacitor:setReservation(reservation.requestId, reservation)
    end
end

local function removeReservation(capacitor, storageKey)
    local ledger = getReservationLedger(capacitor)
    local reservation = ledger[storageKey]
    ledger[storageKey] = nil
    if reservation and capacitor.removeReservation then
        local visible
        if capacitor.getReservation then
            visible = capacitor:getReservation(reservation.requestId)
        elseif capacitor.reservations then
            visible = capacitor.reservations[reservation.requestId]
        end
        if visible == reservation
            or (visible and visible.storageKey == storageKey)
        then
            capacitor:removeReservation(reservation.requestId)
        end
    end
    return reservation
end

local function nextReservationId(capacitor, prefix)
    if capacitor.nextReservationId then
        return capacitor:nextReservationId(prefix)
    end
    capacitor.reservationSerial = (capacitor.reservationSerial or 0) + 1
    return (prefix or "capacitor") .. ":" .. tostring(capacitor.reservationSerial)
end

local function getBankById(capacitor, banks, id)
    if capacitor.getBank then
        return capacitor:getBank(id)
    end
    for _, bank in ipairs(banks or {}) do
        if tostring(bank.id) == tostring(id) then
            return bank
        end
    end
    return nil
end

local function sortedReservationIds(ledger)
    local ids = {}
    for id in pairs(ledger or {}) do
        table.insert(ids, id)
    end
    table.sort(ids, function(left, right)
        return tostring(left) < tostring(right)
    end)
    return ids
end

local function reservedAmount(ledger, bankId)
    local amount = 0
    for _, requestId in ipairs(sortedReservationIds(ledger)) do
        local reservation = ledger[requestId]
        for _, allocation in ipairs(reservation.allocations or {}) do
            if tostring(allocation.bankId) == tostring(bankId) then
                amount = amount + allocation.amount
            end
        end
    end
    return amount
end

local function bankSupportsWeapon(bank, weapon)
    local supported = bank.supportedSizeClasses
    if not supported or #supported == 0 or weapon.mountSizeClass == nil then
        return true
    end
    local sizeClass = WeaponMountSizing:normalize(weapon.mountSizeClass)
    for _, supportedSizeClass in ipairs(supported) do
        if WeaponMountSizing:normalize(supportedSizeClass) == sizeClass then
            return true
        end
    end
    return false
end

local function bankPreference(bank, weapon, declarationIndex)
    local sizeClass = weapon.mountSizeClass and WeaponMountSizing:normalize(weapon.mountSizeClass)
    local supported = bank.supportedSizeClasses
    local exact = 1
    if sizeClass and supported and #supported == 1
        and WeaponMountSizing:normalize(supported[1]) == sizeClass
    then
        exact = 0
    end
    local group = 1
    if weapon.capacitorGroup ~= nil and bank.groupId == weapon.capacitorGroup then
        group = 0
    end
    local flexibility = supported and #supported or math.huge
    return {
        bank = bank,
        exact = exact,
        group = group,
        flexibility = flexibility,
        declarationIndex = declarationIndex,
    }
end

local function validateCapacityPolicy(mode, weapon, capacitor)
    local policy = weapon.capacityPolicy and weapon.capacityPolicy[mode]
    if capacitor then
        local configuredPolicy
        if capacitor.getPolicy then
            configuredPolicy = capacitor:getPolicy(mode)
        elseif capacitor.policies then
            configuredPolicy = capacitor.policies[mode]
        end
        policy = configuredPolicy or policy
    end
    local policies = Enums.Weapon.CapacityPolicy
    if mode == Enums.Weapon.FireMode.Volley then
        assert(not policy or policy.id == policies.Burst,
            "volley weapon must use burst capacity policy")
    elseif mode == Enums.Weapon.FireMode.Sequence then
        assert(not policy or policy.id == policies.Sustain,
            "sequence weapon must use sustain capacity policy")
    end
end

local function releaseReservation(capacitor, requestId, ownerKey)
    if not capacitor or not requestId then
        return false
    end
    local storageKey = reservationStorageKey(requestId, ownerKey)
    if not storageKey then
        return false
    end
    return removeReservation(capacitor, storageKey) ~= nil
end

---@param capacitor table
---@param dt number
function WeaponSystem:rechargeCapacitors(capacitor, dt)
    assert(capacitor and dt >= 0)
    local banks = getCapacitorBanks(capacitor)
    for _, bank in ipairs(banks or {}) do
        bank.charge = clamp(
            (bank.charge or 0) + (bank.chargeRate or 0) * dt,
            0,
            bank.maxCharge or 0)
    end
    self:refreshCapacitorReservations(capacitor)
end

---@param capacitor table
---@param cost number
---@return boolean
function WeaponSystem:dischargeCapacitors(capacitor, cost)
    assert(capacitor and cost >= 0)
    if cost == 0 then
        return true
    end
    local banks = getCapacitorBanks(capacitor) or {}
    local ledger = getReservationLedger(capacitor)
    local available = 0
    for declarationIndex, bank in ipairs(banks) do
        ensureBankId(bank, declarationIndex)
        available = available + math.max(0,
            (bank.charge or 0) - reservedAmount(ledger, bank.id))
    end
    if available + ROOT_EPSILON < cost then
        return false
    end
    local remaining = cost
    for declarationIndex, bank in ipairs(banks) do
        ensureBankId(bank, declarationIndex)
        local free = math.max(0,
            (bank.charge or 0) - reservedAmount(ledger, bank.id))
        local used = math.min(free, remaining)
        bank.charge = bank.charge - used
        remaining = remaining - used
        if remaining <= ROOT_EPSILON then
            break
        end
    end
    return true
end

---@param capacitor table
function WeaponSystem:refreshCapacitorReservations(capacitor)
    if not capacitor then
        return
    end
    local ledger = getReservationLedger(capacitor)
    local requestIds = sortedReservationIds(ledger)
    for _, storageKey in ipairs(requestIds) do
        local reservation = ledger[storageKey]
        if reservation and reservation.mode and reservation.weaponByMount then
            self:reserveFirePlan(
                reservation.mode,
                {
                    shots = reservation.shots or {},
                    nextIndex = reservation.nextIndex,
                },
                reservation.weaponByMount,
                capacitor,
                reservation.requestId,
                reservation.ownerKey)
        end
    end
end

---@param mode integer
---@param plan table
---@param weaponByMount table<string, table>
---@param capacitor table|nil
---@param requestId string|nil
---@param ownerKey string|nil
---@return table
function WeaponSystem:reserveFirePlan(mode, plan, weaponByMount, capacitor, requestId, ownerKey)
    assert(mode and plan and weaponByMount)
    local demands = {}
    local selectedShots = {}
    local selectedWeapons = {}
    local required = 0
    for order, mountId in ipairs(plan.shots or {}) do
        if mode == Enums.Weapon.FireMode.Sequence and order > 1 then
            break
        end
        table.insert(selectedShots, mountId)
        local weapon = weaponByMount[mountId]
        assert(weapon, "missing weapon for capacitor plan mount: " .. tostring(mountId))
        selectedWeapons[mountId] = weapon
        validateCapacityPolicy(mode, weapon, capacitor)
        local amount = math.max(0, weapon.capacitorCost or 0)
        if amount > ROOT_EPSILON then
            table.insert(demands, {
                mountId = mountId,
                amount = amount,
                order = order,
                sizeClass = weapon.mountSizeClass,
                weapon = weapon,
                candidates = {},
                remaining = amount,
            })
            required = required + amount
        end
    end

    if not capacitor then
        return {
            ready = true,
            required = required,
            allocated = required,
            reservationId = nil,
            demands = demands,
            shots = selectedShots,
        }
    end
    requestId = tostring(requestId or nextReservationId(capacitor, "weapon"))
    local normalizedOwnerKey = normalizeReservationOwner(ownerKey)
    if #demands == 0 then
        local storageKey = reservationStorageKey(requestId, normalizedOwnerKey)
        if storageKey then
            removeReservation(capacitor, storageKey)
        end
        return {
            ready = true,
            required = required,
            allocated = required,
            reservationId = nil,
            demands = demands,
            shots = selectedShots,
        }
    end
    assert(normalizedOwnerKey,
        "capacitor reservations require a non-empty owner key")
    local storageKey = reservationStorageKey(requestId, normalizedOwnerKey)

    local banks = getCapacitorBanks(capacitor) or {}
    local ledger = getReservationLedger(capacitor)
    removeReservation(capacitor, storageKey)

    for _, demand in ipairs(demands) do
        for declarationIndex, bank in ipairs(banks) do
            ensureBankId(bank, declarationIndex)
            if bankSupportsWeapon(bank, demand.weapon) then
                table.insert(demand.candidates,
                    bankPreference(bank, demand.weapon, declarationIndex))
            end
        end
        table.sort(demand.candidates, function(left, right)
            if left.exact ~= right.exact then
                return left.exact < right.exact
            end
            if left.group ~= right.group then
                return left.group < right.group
            end
            if left.flexibility ~= right.flexibility then
                return left.flexibility < right.flexibility
            end
            if tostring(left.bank.id) ~= tostring(right.bank.id) then
                return tostring(left.bank.id) < tostring(right.bank.id)
            end
            return left.declarationIndex < right.declarationIndex
        end)
    end

    table.sort(demands, function(left, right)
        if #left.candidates ~= #right.candidates then
            return #left.candidates < #right.candidates
        end
        local leftSize = left.sizeClass and (WeaponMountSizing:order(left.sizeClass) or 0) or 0
        local rightSize = right.sizeClass and (WeaponMountSizing:order(right.sizeClass) or 0) or 0
        if leftSize ~= rightSize then
            return leftSize > rightSize
        end
        return left.order < right.order
    end)

    local allocations = {}
    local allocatedByBank = {}
    local allocated = 0
    for _, demand in ipairs(demands) do
        local remaining = demand.amount
        for _, candidate in ipairs(demand.candidates) do
            local bank = candidate.bank
            local bankId = tostring(bank.id)
            local free = math.max(0, (bank.charge or 0)
                - reservedAmount(ledger, bankId)
                - (allocatedByBank[bankId] or 0))
            if free > ROOT_EPSILON then
                local amount = math.min(free, remaining)
                table.insert(allocations, {
                    mountId = demand.mountId,
                    bankId = bankId,
                    amount = amount,
                })
                allocatedByBank[bankId] = (allocatedByBank[bankId] or 0) + amount
                allocated = allocated + amount
                remaining = remaining - amount
                if remaining <= ROOT_EPSILON then
                    break
                end
            end
        end
        demand.remaining = remaining
    end

    local ready = true
    for _, demand in ipairs(demands) do
        if demand.remaining > ROOT_EPSILON then
            ready = false
            break
        end
    end
    local reservation = {
        requestId = requestId,
        ownerKey = normalizedOwnerKey,
        storageKey = storageKey,
        mode = mode,
        nextIndex = plan.nextIndex,
        shots = selectedShots,
        weaponByMount = selectedWeapons,
        demands = demands,
        allocations = allocations,
        required = required,
        allocated = allocated,
        ready = ready,
    }
    setReservation(capacitor, storageKey, reservation)
    return reservation
end

---@param capacitor table
---@param requestId string
---@param ownerKey string
---@return boolean
function WeaponSystem:commitFireReservation(capacitor, requestId, ownerKey)
    if not capacitor or not requestId then
        return true
    end
    local storageKey = reservationStorageKey(requestId, ownerKey)
    if not storageKey then
        return false
    end
    local ledger = getReservationLedger(capacitor)
    local reservation = ledger[storageKey]
    if not reservation
        or reservation.ownerKey ~= normalizeReservationOwner(ownerKey)
        or not reservation.ready
    then
        return false
    end
    local banks = getCapacitorBanks(capacitor) or {}
    local bankById = {}
    for _, bank in ipairs(banks) do
        bankById[tostring(bank.id)] = bank
    end
    local requiredByBank = {}
    for _, allocation in ipairs(reservation.allocations or {}) do
        local bankId = tostring(allocation.bankId)
        local bank = bankById[bankId]
        if not bank then
            return false
        end
        requiredByBank[bankId] = (requiredByBank[bankId] or 0) + allocation.amount
    end
    for bankId, amount in pairs(requiredByBank) do
        local bank = bankById[bankId]
        if (bank.charge or 0) + ROOT_EPSILON < amount then
            return false
        end
    end
    for _, allocation in ipairs(reservation.allocations or {}) do
        local bank = bankById[tostring(allocation.bankId)]
        bank.charge = math.max(0, bank.charge - allocation.amount)
    end
    removeReservation(capacitor, storageKey)
    return true
end

---@param capacitor table
---@param requestId string
---@param ownerKey string
---@return boolean
function WeaponSystem:releaseFireReservation(capacitor, requestId, ownerKey)
    return releaseReservation(capacitor, requestId, ownerKey)
end

function WeaponSystem:getStateCapacitorRequestId(state)
    if not state.weaponCapacitorRequestId then
        local ownerKey = state.capacitorOwnerKey
            or state.weaponCapacitorOwnerKey
            or state.seed
            or state.name
            or "weapon-state"
        state.weaponCapacitorRequestId = "weapon:" .. tostring(ownerKey)
    end
    return state.weaponCapacitorRequestId
end

function WeaponSystem:releaseStateCapacitorReservation(state)
    if state and state.weaponCapacitor and state.weaponCapacitorRequestId then
        self:releaseFireReservation(
            state.weaponCapacitor,
            state.weaponCapacitorRequestId,
            state.capacitorOwnerKey
                or state.weaponCapacitorOwnerKey
                or state.weaponCapacitorRequestId)
    end
end

function WeaponSystem:fundStateFirePlan(state, mode, plan, weaponByMount)
    if not plan or #(plan.shots or {}) == 0 then
        self:releaseStateCapacitorReservation(state)
        return false
    end
    if not state.weaponCapacitor then
        return true
    end
    local requestId = self:getStateCapacitorRequestId(state)
    local reservation = self:reserveFirePlan(
        mode,
        plan,
        weaponByMount,
        state.weaponCapacitor,
        requestId,
        state.capacitorOwnerKey
            or state.weaponCapacitorOwnerKey
            or requestId)
    if not reservation.ready then
        return false
    end
    return self:commitFireReservation(
        state.weaponCapacitor,
        requestId,
        state.capacitorOwnerKey
            or state.weaponCapacitorOwnerKey
            or requestId)
end

---@param mode integer
---@param plan table
---@param weaponByMount table<string, table>
---@param capacitor table
---@return table
function WeaponSystem:gateFirePlan(mode, plan, weaponByMount, capacitor)
    assert(mode and plan and weaponByMount)
    if not capacitor then
        return plan
    end
    local requestId = nextReservationId(capacitor, "gate")
    local reservation = self:reserveFirePlan(
        mode, plan, weaponByMount, capacitor, requestId, "gateFirePlan")
    if not reservation.ready then
        self:releaseFireReservation(capacitor, requestId, "gateFirePlan")
        local denied = {}
        for _, mountId in ipairs(plan.shots or {}) do
            table.insert(denied, mountId)
        end
        return {
            shots = {},
            nextIndex = plan.startIndex or plan.nextIndex,
            denied = denied,
        }
    end
    assert(self:commitFireReservation(capacitor, requestId, "gateFirePlan"),
        "ready capacitor reservation failed to commit")
    return {
        shots = reservation.shots or plan.shots,
        nextIndex = plan.nextIndex,
        denied = {},
    }
end

---@param mode string
---@param mountIds string[]
---@param readyByMount table<string, boolean>
---@param sequenceIndex integer|nil
---@return table
function WeaponSystem:planFire(mode, mountIds, readyByMount, sequenceIndex)
    local fireModes = Enums.Weapon.FireMode
    assert(mode == fireModes.Volley or mode == fireModes.Sequence, "unsupported weapon fire mode")
    assert(mountIds and readyByMount)

    local plan = {
        shots = {},
        nextIndex = sequenceIndex or 1,
        startIndex = sequenceIndex or 1,
    }

    if #mountIds == 0 then
        return plan
    end

    if mode == Enums.Weapon.FireMode.Volley then
        for _, mountId in ipairs(mountIds) do
            if readyByMount[mountId] then
                table.insert(plan.shots, mountId)
            end
        end
        return plan
    end

    local index = sequenceIndex or 1
    index = ((index - 1) % #mountIds) + 1
    for offset = 0, #mountIds - 1 do
        local candidateIndex = ((index + offset - 1) % #mountIds) + 1
        local mountId = mountIds[candidateIndex]
        if readyByMount[mountId] then
            table.insert(plan.shots, mountId)
            plan.nextIndex = (candidateIndex % #mountIds) + 1
            break
        end
    end

    return plan
end

---@param sizeClass string|nil
---@param aiConfig table|nil
---@return string
function WeaponSystem:selectFireMode(sizeClass, aiConfig)
    aiConfig = aiConfig or {}
    local modeBySizeClass = aiConfig.modeBySizeClass or {}
    local mode = modeBySizeClass[sizeClass or "small"]
        or aiConfig.defaultMode
        or Enums.Weapon.FireMode.Sequence
    assert(mode == Enums.Weapon.FireMode.Volley or mode == Enums.Weapon.FireMode.Sequence,
        "AI weapon mode must be a known enum value")
    return mode
end

---@param origin Vec3f
---@param candidates table[]
---@param range number
---@return table|nil
function WeaponSystem:selectNearestTarget(origin, candidates, range)
    assert(origin and candidates and range >= 0)

    local rangeSquared = range * range
    local best = nil
    local bestDistanceSquared = math.huge

    for _, candidate in ipairs(candidates) do
        if candidate.enabled ~= false and candidate.position then
            local dx = candidate.position.x - origin.x
            local dy = candidate.position.y - origin.y
            local dz = candidate.position.z - origin.z
            local distanceSquared = dx * dx + dy * dy + dz * dz

            local closer = distanceSquared < bestDistanceSquared
            local tied = math.abs(distanceSquared - bestDistanceSquared) < ROOT_EPSILON
            local lowerId = best and candidate.id < best.id

            if distanceSquared <= rangeSquared and (not best or closer or (tied and lowerId)) then
                best = candidate
                bestDistanceSquared = distanceSquared
            end
        end
    end

    return best
end



---Select the highest-priority target in range for a weapon role.
---Score = sizeClassPriority / distance; closer and role-appropriate
---contacts win. Hysteresis keeps the current target unless a candidate
---scores decisively better.
---@param origin Vec3f
---@param candidates table[]
---@param range number
---@param combatRole string|nil
---@param currentEntity Entity|nil
---@return table|nil best, number bestScore
function WeaponSystem:selectPriorityTarget(
    origin, candidates, range, combatRole, currentEntity)
    assert(origin and candidates and range >= 0)

    local priorities = Config.weapons.targetPriorityByRole
    local weights = (combatRole and priorities[combatRole])
        or priorities[Enums.Weapon.CombatRole.Line]
    local rangeSquared = range * range
    local switchHysteresis = 1.3

    local best, bestScore, bestIsCurrent = nil, 0.0, false
    for _, candidate in ipairs(candidates) do
        if candidate.enabled ~= false and candidate.position then
            local dx = candidate.position.x - origin.x
            local dy = candidate.position.y - origin.y
            local dz = candidate.position.z - origin.z
            local distanceSquared = dx * dx + dy * dy + dz * dz
            if distanceSquared <= rangeSquared then
                local distance = math.sqrt(distanceSquared)
                local weight = weights[candidate.sizeClass] or 1.0
                -- Weight dominates; distance applies a soft falloff so a
                -- high-priority large hull beats a close low-priority one.
                local distanceFactor = range / math.max(distance, 0.0001)
                local score = weight * (0.5 + 0.5 * math.min(distanceFactor, 2.0))
                local isCurrent = currentEntity ~= nil
                    and candidate.entity == currentEntity
                if not best
                    or score > bestScore * (bestIsCurrent and switchHysteresis or 1.0)
                    or (isCurrent and not bestIsCurrent
                        and score >= bestScore / switchHysteresis)
                then
                    best = candidate
                    bestScore = score
                    bestIsCurrent = isCurrent
                end
            end
        end
    end

    return best, bestScore
end

---Choose the best target for one weapon mount from live candidates.
---Scoring: role-based size-class weight divided by distance. An explicit
---focusTarget (set by external systems) overrides scoring when valid and
---in range.
---@param mount table Mount record with body/position context
---@param candidates table[] Live candidate records
---@param origin Vec3f
---@param range number Weapon range in game units
---@param combatRole string|nil Weapon combat role for priority weights
---@param focusTargetEntity Entity|nil External focus override
---@return table|nil best Chosen candidate record
function WeaponSystem:chooseBestTarget(
    mount, candidates, origin, range, combatRole, focusTargetEntity)
    assert(mount and candidates and origin and range >= 0)

    local priorities = Config.weapons.targetPriorityByRole
    local weights = (combatRole and priorities[combatRole])
        or priorities[Enums.Weapon.CombatRole.Line]

    -- Focus override: external systems can command this mount onto a
    -- specific contact; validity and range are still enforced.
    if focusTargetEntity then
        for _, candidate in ipairs(candidates) do
            if candidate.entity == focusTargetEntity
                and candidate.enabled ~= false
                and candidate.position
            then
                local dx = candidate.position.x - origin.x
                local dy = candidate.position.y - origin.y
                local dz = candidate.position.z - origin.z
                if dx*dx + dy*dy + dz*dz <= range * range then
                    return candidate
                end
            end
        end
        -- Focus invalid/out of range: fall through to scoring.
    end

    -- X4-style threat scoring:
    --   threat   = base danger the contact represents (size class)
    --   killEase = damaged contacts are cheaper to finish off
    --   distance = soft falloff, tie-breaker only (not dominant)
    --   role     = mount-role vs contact-size matching multiplier
    local best, bestScore = nil, -math.huge
    for _, candidate in ipairs(candidates) do
        if candidate.enabled ~= false and candidate.position then
            local dx = candidate.position.x - origin.x
            local dy = candidate.position.y - origin.y
            local dz = candidate.position.z - origin.z
            local distanceSquared = dx*dx + dy*dy + dz*dz
            if distanceSquared <= range * range then
                local distance = math.max(math.sqrt(distanceSquared), 0.0001)

                -- Threat: how dangerous is this contact class?
                local threat = weights[candidate.sizeClass] or 1.0

                -- Kill ease: weakened contacts are attractive finishes.
                local killEase = 1.0
                if candidate.healthFraction ~= nil then
                    local fraction = clamp(candidate.healthFraction, 0.05, 1.0)
                    killEase = 0.75 + 0.85 * (1.0 - fraction)
                end

                -- Role match: PD guns value small prey, capital guns value
                -- big hulls; mismatched pairings are penalized.
                local roleMatch = 1.0
                if combatRole == Enums.Weapon.CombatRole.PointDefense then
                    roleMatch = candidate.sizeClass == Enums.Target.SizeClass.Small and 2.0
                        or candidate.sizeClass == Enums.Target.SizeClass.Medium and 1.0
                        or 0.35
                elseif combatRole == Enums.Weapon.CombatRole.CapitalHeavy then
                    roleMatch = candidate.sizeClass == Enums.Target.SizeClass.Capital and 2.2
                        or candidate.sizeClass == Enums.Target.SizeClass.Large and 1.6
                        or 0.5
                elseif combatRole == Enums.Weapon.CombatRole.Line
                    or combatRole == Enums.Weapon.CombatRole.Heavy
                then
                    roleMatch = candidate.sizeClass == Enums.Target.SizeClass.Capital and 1.8
                        or candidate.sizeClass == Enums.Target.SizeClass.Large and 1.6
                        or candidate.sizeClass == Enums.Target.SizeClass.Medium and 1.2
                        or 0.8
                end

                -- Distance: soft falloff so it breaks ties but doesn't
                -- dominate the threat decision.
                local distanceFactor = range / (distance + range * 0.25)

                local score = threat * killEase * roleMatch
                    * (0.6 + 0.4 * math.min(distanceFactor, 2.0))
                if score > bestScore then
                    bestScore = score
                    best = candidate
                end
            end
        end
    end
    return best
end

---@param turret TurretComponent
---@param origin Vec3f
---@param target Vec3f
---@param dt number
---@param weapon table
---@param baseRotation Quat|nil
---@param trackingConfig table|nil
---@return table
function WeaponSystem:aimTurret(turret, origin, target, dt, weapon, baseRotation, trackingConfig)
    assert(turret and origin and target and dt >= 0 and weapon)

    local direction = Vec3f(
        target.x - origin.x,
        target.y - origin.y,
        target.z - origin.z
    )
    if baseRotation then
        direction = baseRotation:inverse():mulV(direction)
    end
    local rawDesiredYaw, rawDesiredPitch = self:directionToAngles(direction)
    local desiredYaw = clamp(rawDesiredYaw, turret.yawMin, turret.yawMax)
    local desiredPitch = clamp(rawDesiredPitch, turret.pitchMin, turret.pitchMax)

    turret.desiredYaw = desiredYaw
    turret.desiredPitch = desiredPitch
    local tracking = trackingConfig or weapon.tracking or {}
    turret.yaw = stepAngle(turret.yaw, desiredYaw, (tracking.traverseRate or turret.traverseRate) * dt)
    turret.pitch = stepAngle(turret.pitch, desiredPitch, (tracking.traverseRate or turret.traverseRate) * dt)

    local yawError = math.abs(wrapAngle(rawDesiredYaw - turret.yaw))
    local pitchError = math.abs(rawDesiredPitch - turret.pitch)
    local aimError = math.max(yawError, pitchError)
    local withinLimits = math.abs(rawDesiredYaw - desiredYaw) <= ROOT_EPSILON
        and math.abs(rawDesiredPitch - desiredPitch) <= ROOT_EPSILON
    local ready = withinLimits and aimError <= ((tracking.aimTolerance) or 0.01)

    turret.ready = ready
    turret.aimError = aimError
    turret.withinAimLimits = withinLimits

    return {
        yaw = turret.yaw,
        pitch = turret.pitch,
        desiredYaw = desiredYaw,
        desiredPitch = desiredPitch,
        rawDesiredYaw = rawDesiredYaw,
        rawDesiredPitch = rawDesiredPitch,
        aimError = aimError,
        withinLimits = withinLimits,
        ready = ready,
    }
end

---@param turret table
---@param weapon table
---@return integer remaining shots after the current shot
function WeaponSystem:advanceBurst(turret, weapon)
    assert(turret and weapon, "burst advancement requires a turret and weapon")
    local burst = weapon.effect and weapon.effect.burst
    if not burst then
        turret.burstRemaining = 0
        turret.burstGap = 0
        return 0
    end

    local count = math.max(1, math.floor(burst.count or 1))
    if (turret.burstRemaining or 0) <= 0 then
        turret.burstRemaining = count - 1
    else
        turret.burstRemaining = math.max(0, turret.burstRemaining - 1)
    end
    turret.burstGap = turret.burstRemaining > 0
        and math.max(0, burst.gap or weapon.interShotGap or 0)
        or 0
    return turret.burstRemaining
end

---@param state table
function WeaponSystem:updateFromTracking(state)
    local control = state.control
    local targetBody = state.weaponTargetBody or state.targetBody
    local trackingByMount = state.trackingByMount or {}
    local readyByMount = {}
    local weaponByMount = {}
    local sightCount = 0
    local sightReasons = { none = 0, source = 0, target = 0, other = 0, hull = 0 }
    local sightByMount = {}
    local sightReasonByMount = {}
    local readyCount = 0
    state.sightHitPosition = nil

    if not targetBody then
        self:releaseStateCapacitorReservation(state)
        state.readyByMount = {}
        state.sightCount = 0
        state.sightReasons = sightReasons
        state.sightByMount = {}
        state.sightReasonByMount = {}
        state.readyCount = 0
        state.sightHitPosition = nil
        state.targetPointByMount = {}
        state.lastTargetPoint = nil
        control.interShotGap = 0
        for _, mount in ipairs(state.turrets or {}) do
            mount.component.burstRemaining = 0
            mount.component.burstGap = 0
            mount.component.fireSolution = nil
            mount.component.aimPosition = nil
            mount.component.inRange = false
            mount.component.hasLineOfSight = false
            mount.component.ready = false
        end
        return
    end

    for _, mount in ipairs(state.turrets or {}) do
        local turret = mount.component
        local tracking = trackingByMount[mount.mountId] or {}
        local weapon = tracking.weapon or self:resolveWeapon(turret)
        assert(weapon, "unregistered weapon: " .. tostring(turret.weaponId))
        weaponByMount[mount.mountId] = weapon
        turret.burstGap = math.max(0, (turret.burstGap or 0) - (state.deltaTime or 0))
        turret.cooldown = math.max(0, (turret.cooldown or 0) - (state.deltaTime or 0))

        local solution = tracking.solution
        local aimPosition = tracking.aimPosition or targetBody:getPos()
        local targetPoint = tracking.targetPoint or targetBody:getPos()
        local sourcePosition = tracking.sourcePosition or mount.body:getPos()
        local aim = tracking.aim
        local distance = math.sqrt(
            (sourcePosition.x - targetPoint.x) ^ 2
            + (sourcePosition.y - targetPoint.y) ^ 2
            + (sourcePosition.z - targetPoint.z) ^ 2)
        local inRange = distance <= weapon.range
        local hasSight = false
        local sightReason = solution and "none" or "no_solution"
        local sightHitPosition
        if solution then
            -- LOS validates against THIS mount's engaged contact; role
            -- groups engaging different contacts don't block each other.
            local mountTargetBody = tracking.targetBody or targetBody
            hasSight, sightReason, sightHitPosition = self:hasLineOfSight(
                state.world,
                mount.body,
                mountTargetBody,
                sourcePosition,
                targetPoint,
                state.losObstacles)
        end
        if sightHitPosition and not state.sightHitPosition then
            state.sightHitPosition = sightHitPosition
        end
        if hasSight then
            sightCount = sightCount + 1
        end
        sightByMount[mount.mountId] = hasSight
        sightReasonByMount[mount.mountId] = sightReason
        if sightReasons[sightReason] ~= nil then
            sightReasons[sightReason] = sightReasons[sightReason] + 1
        end

        turret.fireSolution = solution
        turret.aimPosition = aimPosition
        turret.inRange = inRange
        turret.hasLineOfSight = hasSight
        turret.ready = aim ~= nil and aim.ready == true
        local burstReady = (turret.burstRemaining or 0) > 0
            and (turret.burstGap or 0) <= 0
        readyByMount[mount.mountId] = solution ~= nil
            and inRange
            and hasSight
            and turret.ready
            and (turret.cooldown <= 0 or burstReady)
            and ((turret.burstRemaining or 0) == 0 or (turret.burstGap or 0) <= 0)
        if readyByMount[mount.mountId] then
            readyCount = readyCount + 1
        end
    end

    state.readyByMount = readyByMount
    state.sightCount = sightCount
    state.sightReasons = sightReasons
    state.sightByMount = sightByMount
    state.sightReasonByMount = sightReasonByMount
    state.readyCount = readyCount

    local hasBurstPending = false
    for _, mount in ipairs(state.turrets or {}) do
        if (mount.component.burstRemaining or 0) > 0 then
            hasBurstPending = true
            break
        end
    end
    if not control.triggerHeld and not hasBurstPending then
        self:releaseStateCapacitorReservation(state)
        return
    end

    -- Point-defense mounts bypass the shared battery cadence: each fires on
    -- its own cooldown as long as it holds a solution, so PD coverage is a
    -- continuous stream instead of one round-robin slot among all mounts.
    local pdShots = {}
    for _, mount in ipairs(state.turrets or {}) do
        local weapon = weaponByMount[mount.mountId]
        if weapon and weapon.combatRole == Enums.Weapon.CombatRole.PointDefense
            and readyByMount[mount.mountId]
            and (mount.component.cooldown or 0) <= 0
            and mount.component.fireSolution
        then
            table.insert(pdShots, mount.mountId)
        end
    end
    if #pdShots > 0 then
        local pdPlan = { shots = pdShots, nextIndex = control.sequenceIndex }
        if self:fundStateFirePlan(state, Enums.Weapon.FireMode.Volley, pdPlan, weaponByMount) then
            for _, mountId in ipairs(pdPlan.shots) do
                local mount = state.turretsById[mountId]
                local weapon = weaponByMount[mountId]
                if mount and mount.component.fireSolution then
                    control.shotSerial = control.shotSerial + 1
                    self:spawnFiringLight(state, mount, weapon, control.shotSerial)
                    mount.component.cooldown = weapon.cooldown
                    state:spawnProjectile(mount, mount.component.fireSolution, weapon, control.shotSerial)
                end
            end
        end
    end

    if control.interShotGap > 0 then
        return
    end

    local order = control.mode == Enums.Weapon.FireMode.Sequence and control.sequence or state.mountIds
    local plan = self:planFire(control.mode, order, readyByMount, control.sequenceIndex)
    if not self:fundStateFirePlan(state, control.mode, plan, weaponByMount) then
        return
    end
    if #plan.shots == 0 then
        return
    end

    for _, mountId in ipairs(plan.shots) do
        local mount = state.turretsById[mountId]
        local weapon = weaponByMount[mountId]
        if mount and mount.component.fireSolution then
            control.shotSerial = control.shotSerial + 1
            self:spawnFiringLight(state, mount, weapon, control.shotSerial)
            mount.component.cooldown = weapon.cooldown
            if weapon.effect.kind == Enums.Weapon.Effect.Beam then
                state:spawnBeam(mount, mount.component.fireSolution, weapon, control.shotSerial)
            elseif weapon.effect.kind == Enums.Weapon.Effect.Projectile then
                state:spawnProjectile(mount, mount.component.fireSolution, weapon, control.shotSerial)
            else
                error("unsupported weapon effect kind: " .. tostring(weapon.effect.kind))
            end
            self:advanceBurst(mount.component, weapon)
        end
    end

    control.sequenceIndex = plan.nextIndex
    control.interShotGap = weaponByMount[plan.shots[1]].interShotGap
    state.lastShotOrder = plan.shots
end

---@param state table
---@param mount table
---@param weapon WeaponDefinition
---@param shotSerial integer
---@return Entity|nil
function WeaponSystem:spawnFiringLight(state, mount, weapon, shotSerial)
    local presentation = WeaponRegistry:getPresentation(weapon)
    local firingLight = presentation and presentation.firingLight
    if not firingLight or not mount or not mount.body then
        return nil
    end

    local origin = mount.body:getPos()
    local direction
    local aimPosition = mount.component and mount.component.aimPosition
    if aimPosition then
        local dx = aimPosition.x - origin.x
        local dy = aimPosition.y - origin.y
        local dz = aimPosition.z - origin.z
        local length = math.sqrt(dx * dx + dy * dy + dz * dz)
        if length > ROOT_EPSILON then
            direction = Vec3f(dx / length, dy / length, dz / length)
        end
    end
    if not direction and mount.entity then
        local rigidBodyComponent = mount.entity:get(PhysicsComponents.RigidBody)
        if rigidBodyComponent then
            direction = rigidBodyComponent:getForward()
        end
    end

    local muzzleOffset = math.max(0, firingLight.muzzleOffset or 0)
    local position = Position(origin.x, origin.y, origin.z)
    if direction then
        position = Position(
            origin.x + direction.x * muzzleOffset,
            origin.y + direction.y * muzzleOffset,
            origin.z + direction.z * muzzleOffset)
    end

    local entity = PointLightEffectEntity(shotSerial, {
        kind = "weapon_firing",
        position = position,
        pointLight = {
            color = firingLight.color or presentation.lightColor,
            radius = firingLight.radius,
            intensity = firingLight.intensity,
        },
        duration = firingLight.duration,
        fadeOutDuration = firingLight.fadeOutDuration,
    })
    state.lightEffects = state.lightEffects or {}
    table.insert(state.lightEffects, entity)
    return entity
end

---@param state table
---@param dt number
function WeaponSystem:update(state, dt)
    local control = state.control
    local targetBody = state.weaponTargetBody or state.targetBody
    if not control then
        return
    end

    control.interShotGap = math.max(0, (control.interShotGap or 0) - dt)
    if state.weaponCapacitor then
        self:rechargeCapacitors(state.weaponCapacitor, dt)
    end

    state.deltaTime = dt
    if state.weaponTrackingComponent then
        self:updateFromTracking(state)
        return
    end

    if not targetBody then
        self:releaseStateCapacitorReservation(state)
        state.readyByMount = {}
        state.sightCount = 0
        state.sightReasons = { none = 0, source = 0, target = 0, other = 0, hull = 0 }
        state.sightByMount = {}
        state.sightReasonByMount = {}
        state.readyCount = 0
        state.sightHitPosition = nil
        state.targetPointByMount = {}
        state.lastTargetPoint = nil
        control.interShotGap = 0
        for _, mount in ipairs(state.turrets or {}) do
            mount.component.burstRemaining = 0
            mount.component.burstGap = 0
            mount.component.fireSolution = nil
            mount.component.aimPosition = nil
            mount.component.inRange = false
            mount.component.hasLineOfSight = false
            mount.component.ready = false
        end
        return
    end

    local targetPosition = targetBody:getPos()
    local targetVelocity = state.targetVelocity or targetBody:getVelocity()
    state.targetPointTime = (state.targetPointTime or 0) + dt
    local targetPointByMount = {}
    local readyByMount = {}
    local weaponByMount = {}
    local sightCount = 0
    local sightReasons = { none = 0, source = 0, target = 0, other = 0, hull = 0 }
    local sightByMount = {}
    local sightReasonByMount = {}
    local readyCount = 0
    state.sightHitPosition = nil
    state.aimStep = (state.aimStep or 0) + 1
    state.beamSwayBasisByMount = state.beamSwayBasisByMount or {}
    local targetPointRng = RNG.Create(state.targetPointSeed or 0)
    local beamSwayRng = RNG.Create(state.beamSwaySeed or 0)
    assert(targetPointRng and beamSwayRng,
        "weapon system could not create deterministic target and sway RNG streams")

    for mountIndex, mount in ipairs(state.turrets) do
        local turret = mount.component
        local body = mount.body
        local weapon = self:resolveWeapon(turret)
        weaponByMount[mount.mountId] = weapon
        turret.cooldown = math.max(0, turret.cooldown - dt)

        local sourcePosition = body:getPos()
        local sourceVelocity = body:getVelocity()
        local effect = weapon.effect
        assert(effect, "ship weapon has no effect definition: " .. tostring(turret.weaponId))
        local targetPoint = targetPosition
        local targetPointLocal = nil
        if state.targetSurface then
            local toSourceX = sourcePosition.x - targetPosition.x
            local toSourceY = sourcePosition.y - targetPosition.y
            local toSourceZ = sourcePosition.z - targetPosition.z
            local toSourceLength = math.sqrt(
                toSourceX * toSourceX + toSourceY * toSourceY + toSourceZ * toSourceZ)
            local pointOptions = state.targetPointOptions or {}
            local sampledOptions = {
                motionAmplitude = pointOptions.motionAmplitude,
                motionFrequency = pointOptions.motionFrequency,
                minFacingDot = pointOptions.minFacingDot or 0.15,
            }
            if toSourceLength > ROOT_EPSILON then
                sampledOptions.viewDirection = Vec3f(
                    toSourceX / toSourceLength,
                    toSourceY / toSourceLength,
                    toSourceZ / toSourceLength)
            end
            local targetPointSample = self:sampleTargetPoint(
                state.targetSurface,
                targetPointRng:get64(),
                state.targetPointTime,
                sampledOptions)
            targetPoint = self:targetPointToWorld(targetPointSample, targetBody)
            targetPointLocal = Vec3f(
                targetPointSample.position.x,
                targetPointSample.position.y,
                targetPointSample.position.z)
            targetPointByMount[mount.mountId] = {
                position = targetPoint,
                triangleIndex = targetPointSample.triangleIndex,
                normal = targetPointSample.normal,
            }
        end
        local parentRotation = mount.parentBody and mount.parentBody:getRot() or Quat.Identity()
        local mountRotation = parentRotation * turret:getLocalRotation()
        local beamSwayPhase = beamSwayRng:getUniform() * math.pi * 2
        local beamSwayBasis
        if effect.kind == Enums.Weapon.Effect.Beam then
            beamSwayBasis = BeamAimHelper.getBasis(
                sourcePosition,
                targetPoint,
                state.beamSwayBasisByMount[mount.mountId])
            state.beamSwayBasisByMount[mount.mountId] = beamSwayBasis
        end
        local solution
        if effect.kind == Enums.Weapon.Effect.Beam then
            solution = {
                time = 0,
                position = targetPoint,
                targetPoint = targetPoint,
                targetPointLocal = targetPointLocal,
                swayPhase = beamSwayPhase,
                swayTime = state.targetPointTime,
            }
        else
            solution = self:solveIntercept(
                sourcePosition,
                sourceVelocity,
                targetPoint,
                targetVelocity,
                effect.speed,
                weapon.intercept and weapon.intercept.maxPredictionTime)
        end

        local aimPosition = solution and solution.position or targetPosition
        if solution then
            local accuracySeed = effect.kind == Enums.Weapon.Effect.Beam
                and mountIndex
                or state.aimStep * 17 + mountIndex
            local accurateAim = self:applyAccuracy(
                sourcePosition,
                solution.position,
                weapon,
                accuracySeed)
            if effect.kind == Enums.Weapon.Effect.Beam then
                accurateAim.position = BeamAimHelper.computeEndpoint(
                    sourcePosition,
                    targetPoint,
                    effect,
                    state.targetPointTime,
                    beamSwayPhase,
                    accurateAim.angles,
                    beamSwayBasis)
            end
            solution = {
                time = solution.time,
                position = accurateAim.position,
                aimOffset = accurateAim.offset,
                aimAngles = accurateAim.angles,
                targetPoint = targetPoint,
                targetPointLocal = targetPointLocal,
                swayPhase = beamSwayPhase,
                swayTime = state.targetPointTime,
                swayBasis = beamSwayBasis,
            }
            aimPosition = solution.position
        end
        local aim = self:aimTurret(
            turret,
            sourcePosition,
            aimPosition,
            dt,
            weapon,
            mountRotation)
        local turretRotation = Quat.FromEuler(aim.yaw, aim.pitch, 0)
        body:setRot(mountRotation * turretRotation)

        local distance = math.sqrt(
            (sourcePosition.x - aimPosition.x) ^ 2
            + (sourcePosition.y - aimPosition.y) ^ 2
            + (sourcePosition.z - aimPosition.z) ^ 2)
        local inRange = distance <= weapon.range
        local hasSight = false
        local sightReason = solution and "none" or "no_solution"
        local sightHitPosition = nil
        if solution ~= nil then
            hasSight, sightReason, sightHitPosition = self:hasLineOfSight(
                state.world,
                body,
                targetBody,
                sourcePosition,
                targetPoint,
                state.losObstacles)
        end
        if not sightReason then
            sightReason = solution and "none" or "no_solution"
        end
        if sightHitPosition and not state.sightHitPosition then
            state.sightHitPosition = sightHitPosition
        end

        if hasSight then
            sightCount = sightCount + 1
        end
        sightByMount[mount.mountId] = hasSight
        sightReasonByMount[mount.mountId] = sightReason
        if sightReasons[sightReason] ~= nil then
            sightReasons[sightReason] = sightReasons[sightReason] + 1
        end

        turret.fireSolution = solution
        turret.aimPosition = aimPosition
        turret.inRange = inRange
        turret.hasLineOfSight = hasSight
        readyByMount[mount.mountId] = solution ~= nil
            and inRange
            and hasSight
            and aim.ready
            and turret.cooldown <= 0
        if readyByMount[mount.mountId] then
            readyCount = readyCount + 1
        end
    end

    state.readyByMount = readyByMount
    state.sightCount = sightCount
    state.sightReasons = sightReasons
    state.sightByMount = sightByMount
    state.sightReasonByMount = sightReasonByMount
    state.targetPointByMount = targetPointByMount
    local firstMountId = state.mountIds and state.mountIds[1]
    state.lastTargetPoint = firstMountId and targetPointByMount[firstMountId] or nil
    state.readyCount = readyCount

    if not control.triggerHeld then
        self:releaseStateCapacitorReservation(state)
        return
    end
    if control.interShotGap > 0 then
        return
    end

    local order = control.mode == Enums.Weapon.FireMode.Sequence and control.sequence or state.mountIds
    local plan = self:planFire(control.mode, order, readyByMount, control.sequenceIndex)
    if not self:fundStateFirePlan(state, control.mode, plan, weaponByMount) then
        return
    end
    if #plan.shots == 0 then
        return
    end

    for _, mountId in ipairs(plan.shots) do
        local mount = state.turretsById[mountId]
        local weapon = weaponByMount[mountId]
        if mount and mount.component.fireSolution then
            control.shotSerial = control.shotSerial + 1
            self:spawnFiringLight(state, mount, weapon, control.shotSerial)
            mount.component.cooldown = weapon.cooldown
            if weapon.effect.kind == Enums.Weapon.Effect.Beam then
                state:spawnBeam(mount, mount.component.fireSolution, weapon, control.shotSerial)
            elseif weapon.effect.kind == Enums.Weapon.Effect.Projectile then
                state:spawnProjectile(mount, mount.component.fireSolution, weapon, control.shotSerial)
            else
                error("unsupported weapon effect kind: " .. tostring(weapon.effect.kind))
            end
        end
    end

    control.sequenceIndex = plan.nextIndex
    control.interShotGap = weaponByMount[plan.shots[1]].interShotGap
    state.lastShotOrder = plan.shots
end

---@param direction Vec3f
---@return number yaw, number pitch
function WeaponSystem:directionToAngles(direction)
    assert(direction)

    local horizontal = math.sqrt(direction.x * direction.x + direction.z * direction.z)
    if horizontal < ROOT_EPSILON and math.abs(direction.y) < ROOT_EPSILON then
        return 0, 0
    end

    return math.atan2(-direction.x, -direction.z), math.atan2(direction.y, horizontal)
end

return WeaponSystem()
