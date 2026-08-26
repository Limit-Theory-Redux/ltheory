local Component = require("Core.ECS.Component")

---@class AsteroidBeltComponent: Component
---@overload fun(asteroidData: table, orbitRadius: number, width: number): AsteroidBeltComponent
local AsteroidBeltComponent = Subclass("AsteroidBeltComponent", Component, function(self, asteroidData, orbitRadius, width, lodMesh)
    self:setComponentName("AsteroidBelt")

    self._asteroidData = asteroidData or {}
    self._orbitRadius  = orbitRadius or 0
    self._width        = width or 0
    self._lodMesh      = lodMesh
end)

function AsteroidBeltComponent:getAsteroidData()  return self._asteroidData end
function AsteroidBeltComponent:getOrbitRadius()    return self._orbitRadius end
function AsteroidBeltComponent:getWidth()          return self._width end
function AsteroidBeltComponent:getLodMesh()        return self._lodMesh end

function AsteroidBeltComponent:setAsteroidData(v)  self._asteroidData = v end
function AsteroidBeltComponent:setOrbitRadius(v)   self._orbitRadius = v end
function AsteroidBeltComponent:setWidth(v)         self._width = v end
function AsteroidBeltComponent:setLodMesh(v)       self._lodMesh = v end

function AsteroidBeltComponent:getCount()
    return #self._asteroidData
end

--- Build angular buckets for fast spatial lookup (call once after generation)
function AsteroidBeltComponent:buildBuckets(numBuckets)
    numBuckets = numBuckets or 360
    self._buckets = {}
    self._numBuckets = numBuckets
    for i = 1, numBuckets do self._buckets[i] = {} end

    for i, a in ipairs(self._asteroidData) do
        local angle = math.atan2(a.pz, a.px) -- -pi to pi
        local bucket = math.floor((angle + math.pi) / (2 * math.pi) * numBuckets) + 1
        bucket = math.max(1, math.min(numBuckets, bucket))
        table.insert(self._buckets[bucket], i)
    end
end

--- Get asteroid indices in a range of angular buckets
function AsteroidBeltComponent:getAsteroidsInAngleRange(angleMin, angleMax)
    if not self._buckets then self:buildBuckets() end
    local result = {}
    local nb = self._numBuckets
    local b1 = math.floor((angleMin + math.pi) / (2 * math.pi) * nb) + 1
    local b2 = math.floor((angleMax + math.pi) / (2 * math.pi) * nb) + 1
    b1 = math.max(1, math.min(nb, b1))
    b2 = math.max(1, math.min(nb, b2))

    if b1 <= b2 then
        for b = b1, b2 do
            for _, idx in ipairs(self._buckets[b]) do
                table.insert(result, idx)
            end
        end
    else
        -- Wraps around (e.g., 350° to 10°)
        for b = b1, nb do
            for _, idx in ipairs(self._buckets[b]) do
                table.insert(result, idx)
            end
        end
        for b = 1, b2 do
            for _, idx in ipairs(self._buckets[b]) do
                table.insert(result, idx)
            end
        end
    end
    return result
end

return AsteroidBeltComponent
