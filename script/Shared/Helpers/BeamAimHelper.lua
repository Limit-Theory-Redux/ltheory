local BeamAimHelper = {}

local ROOT_EPSILON = 0.000001
local TWO_PI = math.pi * 2

local function getBasis(sourcePosition, targetPosition, previousBasis)
    local dx = targetPosition.x - sourcePosition.x
    local dy = targetPosition.y - sourcePosition.y
    local dz = targetPosition.z - sourcePosition.z
    local distanceSquared = dx * dx + dy * dy + dz * dz
    if distanceSquared <= ROOT_EPSILON then
        return nil
    end

    local distance = math.sqrt(distanceSquared)
    local invDistance = 1 / distance
    local lookX = dx * invDistance
    local lookY = dy * invDistance
    local lookZ = dz * invDistance
    local rightX
    local rightY
    local rightZ
    if previousBasis then
        local projection = previousBasis.rightX * lookX
            + previousBasis.rightY * lookY
            + previousBasis.rightZ * lookZ
        rightX = previousBasis.rightX - lookX * projection
        rightY = previousBasis.rightY - lookY * projection
        rightZ = previousBasis.rightZ - lookZ * projection
        local rightLength = math.sqrt(rightX * rightX + rightY * rightY + rightZ * rightZ)
        if rightLength <= ROOT_EPSILON then
            rightX = nil
        else
            rightX = rightX / rightLength
            rightY = rightY / rightLength
            rightZ = rightZ / rightLength
        end
    end
    if not rightX then
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
    end

    return {
        distance = distance,
        lookX = lookX,
        lookY = lookY,
        lookZ = lookZ,
        rightX = rightX,
        rightY = rightY,
        rightZ = rightZ,
        upX = rightY * lookZ - rightZ * lookY,
        upY = rightZ * lookX - rightX * lookZ,
        upZ = rightX * lookY - rightY * lookX,
    }
end

---@param sourcePosition Vec3f
---@param targetPosition Vec3f
---@param previousBasis table|nil
---@return table|nil
function BeamAimHelper.getBasis(sourcePosition, targetPosition, previousBasis)
    assert(sourcePosition and targetPosition)
    return getBasis(sourcePosition, targetPosition, previousBasis)
end

---@param sourcePosition Vec3f
---@param targetPosition Vec3f
---@param effect BeamDefinition|nil
---@param time number
---@param phase number
---@param aimAngles Vec3f|nil angular right/up offsets in radians
---@param previousBasis table|nil
---@return Vec3f
function BeamAimHelper.computeEndpoint(
    sourcePosition,
    targetPosition,
    effect,
    time,
    phase,
    aimAngles,
    previousBasis)
    assert(sourcePosition and targetPosition)
    local basis = getBasis(sourcePosition, targetPosition, previousBasis)
    if not basis then
        return Vec3f(targetPosition.x, targetPosition.y, targetPosition.z)
    end

    aimAngles = aimAngles or Vec3f()
    local angleRight = aimAngles.x or 0
    local angleUp = aimAngles.y or 0
    local sway = type(effect) == "table" and effect.sway or nil
    if sway then
        local amplitude = math.max(0, sway.amplitude or 0)
        if amplitude > 0 then
            local frequency = math.max(0, sway.frequency or 0)
            local secondaryFrequency = math.max(
                0,
                sway.secondaryFrequency or frequency * 0.73)
            local secondaryAmplitude = math.max(0, sway.secondaryAmplitude or 0.5)
            local swayPhase = phase or 0
            local swayTime = time or 0
            angleRight = angleRight + amplitude
                * math.sin(swayTime * frequency * TWO_PI + swayPhase)
            angleUp = angleUp + amplitude * secondaryAmplitude
                * math.sin(swayTime * secondaryFrequency * TWO_PI + swayPhase * 1.37)
        end
    end

    local rightOffset = math.tan(angleRight) * basis.distance
    local upOffset = math.tan(angleUp) * basis.distance
    return Vec3f(
        targetPosition.x + basis.rightX * rightOffset + basis.upX * upOffset,
        targetPosition.y + basis.rightY * rightOffset + basis.upY * upOffset,
        targetPosition.z + basis.rightZ * rightOffset + basis.upZ * upOffset)
end

return BeamAimHelper
