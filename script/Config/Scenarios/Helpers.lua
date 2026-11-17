-- Small helpers the scenarios share.

local PhysicsComponents = require("Modules.Physics.Components")

local Helpers = {}

---The radius of something with a rigid body (its parts included), or its
---Transform scale.
---@param entity Entity
---@return number
function Helpers.radius(entity)
    local rbCmp = entity:get(PhysicsComponents.RigidBody)
    local rb = rbCmp and rbCmp:getRigidBody()
    if rb then
        local r = rb:getBoundingRadiusCompound()
        if r and r > 0 then return r end
    end
    return entity:get(PhysicsComponents.Transform):getScale()
end

---A spot `distance` beyond the player's ship, on the side away from the first
---planet (clear of the planet and its ring), or ahead of the ship without one.
---@param game table
---@param distance number
---@return Position
function Helpers.outward(game, distance)
    local shipPos = game.playerShip:get(PhysicsComponents.RigidBody):getRigidBody():getPos()
    local dx, dy, dz = 0, 0, 1
    local planet = game.planets and game.planets[1]
    if planet then
        local p = planet:get(PhysicsComponents.Transform):getPos()
        local x, y, z = shipPos.x - p.x, shipPos.y - p.y, shipPos.z - p.z
        local len = math.sqrt(x * x + y * y + z * z)
        if len > 1e-9 then dx, dy, dz = x / len, y / len, z / len end
    end
    return Position(shipPos.x + dx * distance, shipPos.y + dy * distance, shipPos.z + dz * distance)
end

return Helpers
