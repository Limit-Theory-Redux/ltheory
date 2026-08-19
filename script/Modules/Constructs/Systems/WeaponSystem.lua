---@class WeaponSystem
---@overload fun(): WeaponSystem
local WeaponSystem = Class("WeaponSystem", function() end)

local ffi = require("ffi")
local WeaponRegistry = require("Shared.Registries.WeaponRegistry")
local PointLightEffectEntity = require("Modules.Constructs.Entities.PointLightEffectEntity")
local PhysicsComponents = require("Modules.Physics.Components")
local RaycastHelper = require("Shared.Helpers.RaycastHelper")
local BeamAimHelper = require("Shared.Helpers.BeamAimHelper")
local NULL_RIGID_BODY = ffi.cast("RigidBody*", nil)
local ROOT_EPSILON = 0.000001

local function clamp(value, minimum, maximum)
    return math.max(minimum, math.min(maximum, value))
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

local function getCapacitorBanks(capacitor, groupId)
    if not capacitor then
        return nil
    end
    if capacitor.getBanks then
        return capacitor:getBanks(groupId)
    end
    local banks = capacitor.banks or capacitor
    if groupId == nil then
        return banks
    end

    local groupedBanks = false
    local selected = {}
    for _, bank in ipairs(banks or {}) do
        if bank.groupId ~= nil then
            groupedBanks = true
            if bank.groupId == groupId then
                table.insert(selected, bank)
            end
        end
    end
    if not groupedBanks then
        return banks
    end
    return selected
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
end

---@param banks table[]
---@param cost number
---@return boolean
function WeaponSystem:dischargeCapacitors(banks, cost)
    assert(banks and cost >= 0)
    if cost == 0 then
        return true
    end

    local available = 0
    for _, bank in ipairs(banks) do
        available = available + math.max(0, bank.charge or 0)
    end
    if available + ROOT_EPSILON < cost then
        return false
    end

    local remaining = cost
    for _, bank in ipairs(banks) do
        local used = math.min(math.max(0, bank.charge or 0), remaining)
        bank.charge = bank.charge - used
        remaining = remaining - used
        if remaining <= ROOT_EPSILON then
            break
        end
    end
    return true
end

---@param mode integer
---@param plan table
---@param weaponByMount table<string, table>
---@param capacitor table
---@return table
function WeaponSystem:gateFirePlan(mode, plan, weaponByMount, capacitor)
    assert(mode and plan and weaponByMount)
    local allBanks = getCapacitorBanks(capacitor)
    if not allBanks or #allBanks == 0 then
        return plan
    end

    local policies = Enums.Weapon.CapacityPolicy
    local gated = { shots = {}, nextIndex = plan.nextIndex, denied = {} }
    local burstGroupReady = {}
    if mode == Enums.Weapon.FireMode.Volley then
        local requiredByGroup = {}
        for _, mountId in ipairs(plan.shots) do
            local weapon = weaponByMount[mountId]
            assert(weapon, "missing weapon for capacitor plan mount: " .. tostring(mountId))
            if weapon.capacitorGroup ~= nil then
                requiredByGroup[weapon.capacitorGroup] =
                    (requiredByGroup[weapon.capacitorGroup] or 0)
                    + math.max(0, weapon.capacitorCost or 0)
            end
        end
        for groupId, requiredCharge in pairs(requiredByGroup) do
            local groupBanks = getCapacitorBanks(capacitor, groupId)
            assert(groupBanks and #groupBanks > 0,
                "missing capacitor bank for weapon group: " .. tostring(groupId))
            local available = 0
            for _, bank in ipairs(groupBanks) do
                available = available + math.max(0, bank.charge or 0)
            end
            burstGroupReady[groupId] = available + ROOT_EPSILON >= requiredCharge
        end
    end
    for _, mountId in ipairs(plan.shots) do
        local weapon = weaponByMount[mountId]
        assert(weapon, "missing weapon for capacitor plan mount: " .. tostring(mountId))
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
        if mode == Enums.Weapon.FireMode.Volley then
            assert(not policy or policy.id == policies.Burst,
                "volley weapon must use burst capacity policy")
        elseif mode == Enums.Weapon.FireMode.Sequence then
            assert(not policy or policy.id == policies.Sustain,
                "sequence weapon must use sustain capacity policy")
        end

        local banks = getCapacitorBanks(capacitor, weapon.capacitorGroup)
        if not banks or #banks == 0 then
            assert((weapon.capacitorCost or 0) <= 0,
                "missing capacitor bank for weapon group: " .. tostring(weapon.capacitorGroup))
        end
        local groupReady = weapon.capacitorGroup == nil
            or mode ~= Enums.Weapon.FireMode.Volley
            or burstGroupReady[weapon.capacitorGroup]
        local sustainLimit = mode == Enums.Weapon.FireMode.Sequence and 1 or math.huge
        if #gated.shots < sustainLimit
            and groupReady
            and banks
            and self:dischargeCapacitors(banks, math.max(0, weapon.capacitorCost or 0)) then
            table.insert(gated.shots, mountId)
        else
            table.insert(gated.denied, mountId)
        end
    end
    return gated
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
        nextIndex = sequenceIndex or 1
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
        local weapon = tracking.weapon or WeaponRegistry:get(turret.weaponId)
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
            hasSight, sightReason, sightHitPosition = self:hasLineOfSight(
                state.world,
                mount.body,
                targetBody,
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
    if (not control.triggerHeld and not hasBurstPending) or control.interShotGap > 0 then
        return
    end

    local order = control.mode == Enums.Weapon.FireMode.Sequence and control.sequence or state.mountIds
    local plan = self:planFire(control.mode, order, readyByMount, control.sequenceIndex)
    plan = self:gateFirePlan(control.mode, plan, weaponByMount, state.weaponCapacitor)
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
        local weapon = WeaponRegistry:get(turret.weaponId)
        assert(weapon, "unregistered weapon: " .. tostring(turret.weaponId))
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

    if not control.triggerHeld or control.interShotGap > 0 then
        return
    end

    local order = control.mode == Enums.Weapon.FireMode.Sequence and control.sequence or state.mountIds
    local plan = self:planFire(control.mode, order, readyByMount, control.sequenceIndex)
    plan = self:gateFirePlan(control.mode, plan, weaponByMount, state.weaponCapacitor)
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
