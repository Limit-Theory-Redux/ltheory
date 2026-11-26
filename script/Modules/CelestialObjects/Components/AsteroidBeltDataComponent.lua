local Component = require("Core.ECS.Component")

---@class AsteroidBeltDataComponent: Component
---@overload fun(asteroidData: table, orbitRadius: number, width: number): AsteroidBeltDataComponent
local AsteroidBeltDataComponent = Subclass("AsteroidBeltDataComponent", Component, function(self, asteroidData, orbitRadius, width, lodMesh)
    self:setComponentName("AsteroidBeltData")

    self._asteroidData = asteroidData or {}
    self._orbitRadius  = orbitRadius or 0
    self._width        = width or 0
    self._lodMesh      = lodMesh
end)

function AsteroidBeltDataComponent:getAsteroidData()  return self._asteroidData end
function AsteroidBeltDataComponent:getOrbitRadius()    return self._orbitRadius end
function AsteroidBeltDataComponent:getWidth()          return self._width end
function AsteroidBeltDataComponent:getLodMesh()        return self._lodMesh end

function AsteroidBeltDataComponent:setAsteroidData(v)  self._asteroidData = v end
function AsteroidBeltDataComponent:setOrbitRadius(v)   self._orbitRadius = v end
function AsteroidBeltDataComponent:setWidth(v)         self._width = v end
function AsteroidBeltDataComponent:setLodMesh(v)       self._lodMesh = v end

function AsteroidBeltDataComponent:getCount()
    return #self._asteroidData
end

--- Build angular buckets for fast spatial lookup (call once after generation)
function AsteroidBeltDataComponent:buildBuckets(numBuckets)
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
function AsteroidBeltDataComponent:getAsteroidsInAngleRange(angleMin, angleMax)
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

return AsteroidBeltDataComponent
