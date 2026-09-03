local ffi = require("ffi")

local NULL_RIGID_BODY = ffi.cast("RigidBody*", nil)
local ROOT_EPSILON = 0.000001
local DEFAULT_ADVANCE_EPSILON = 0.001
local DEFAULT_MAX_IGNORED_HITS = 8

---@class RaycastHelper
---@overload fun(): RaycastHelper
local RaycastHelper = Class("RaycastHelper", function() end)

local function isNullBody(body)
    return body == nil or body == NULL_RIGID_BODY
end

local function readHitPosition(result)
    if not result then
        return nil
    end

    local hasPosition, position = pcall(function()
        return result.position
    end)
    if hasPosition and position then
        return position
    end

    local hasCoordinates, px, py, pz = pcall(function()
        return result.posx, result.posy, result.posz
    end)
    if hasCoordinates and px ~= nil and py ~= nil and pz ~= nil then
        return Vec3f(px, py, pz)
    end
    return nil
end

local function readHitNormal(result)
    if not result then
        return nil
    end

    local hasNormal, normal = pcall(function()
        return result.normal
    end)
    if hasNormal and normal then
        return normal
    end

    local hasCoordinates, nx, ny, nz = pcall(function()
        return result.normx, result.normy, result.normz
    end)
    if hasCoordinates and nx ~= nil and ny ~= nil and nz ~= nil then
        return Vec3f(nx, ny, nz)
    end
    return nil
end

local function isIgnoredBody(body, options)
    if options.ignoreBody and body == options.ignoreBody then
        return true
    end
    if options.shouldIgnore and options.shouldIgnore(body) then
        return true
    end
    for _, ignoredBody in ipairs(options.ignoredBodies or {}) do
        if body == ignoredBody then
            return true
        end
    end
    return false
end

local function setRay(ray, origin, target)
    ray.tMin = 0
    ray.tMax = 1
    ray.px = origin.x
    ray.py = origin.y
    ray.pz = origin.z
    ray.dirx = target.x - origin.x
    ray.diry = target.y - origin.y
    ray.dirz = target.z - origin.z
end

local function makeHit(result, rayOrigin)
    local body = result and result.body
    if isNullBody(body) then
        return nil
    end

    return {
        body = body,
        position = readHitPosition(result),
        normal = readHitNormal(result),
        t = result.t,
        rayOrigin = rayOrigin,
    }
end

---@param world Physics
---@param origin Vec3f
---@param target Vec3f
---@param options table|nil
---@return table|nil
function RaycastHelper:castSegment(world, origin, target, options)
    assert(world and origin and target)
    options = options or {}

    local dx = target.x - origin.x
    local dy = target.y - origin.y
    local dz = target.z - origin.z
    local lengthSquared = dx * dx + dy * dy + dz * dz
    if lengthSquared < ROOT_EPSILON then
        return nil
    end

    local length = math.sqrt(lengthSquared)
    local invLength = 1 / length
    local nx = dx * invLength
    local ny = dy * invLength
    local nz = dz * invLength
    local originEpsilon = options.originEpsilon or 0
    local advanceEpsilon = options.advanceEpsilon or DEFAULT_ADVANCE_EPSILON
    local maxIgnoredHits = options.maxIgnoredHits or DEFAULT_MAX_IGNORED_HITS
    local rayOrigin = Vec3f(
        origin.x + nx * originEpsilon,
        origin.y + ny * originEpsilon,
        origin.z + nz * originEpsilon)
    local ray = Ray()
    local ignoredHits = 0

    while ignoredHits < maxIgnoredHits do
        local remainingX = target.x - rayOrigin.x
        local remainingY = target.y - rayOrigin.y
        local remainingZ = target.z - rayOrigin.z
        if remainingX * remainingX + remainingY * remainingY + remainingZ * remainingZ < ROOT_EPSILON then
            return nil
        end

        setRay(ray, rayOrigin, target)
        local hit = makeHit(world:rayCast(ray), rayOrigin)
        if not hit then
            return nil
        end

        if not isIgnoredBody(hit.body, options) then
            return hit
        end

        ignoredHits = ignoredHits + 1
        if not hit.position then
            return hit
        end

        local nextOrigin = Vec3f(
            hit.position.x + nx * advanceEpsilon,
            hit.position.y + ny * advanceEpsilon,
            hit.position.z + nz * advanceEpsilon)
        local progressX = nextOrigin.x - rayOrigin.x
        local progressY = nextOrigin.y - rayOrigin.y
        local progressZ = nextOrigin.z - rayOrigin.z
        if progressX * nx + progressY * ny + progressZ * nz <= 0 then
            return hit
        end
        rayOrigin = nextOrigin
    end

    return nil
end

return RaycastHelper()
