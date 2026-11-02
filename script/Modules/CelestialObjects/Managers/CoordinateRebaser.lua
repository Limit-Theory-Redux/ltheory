local Registry            = require("Core.ECS.Registry")
local CoreComponents      = require("Modules.Core.Components")
local PhysicsComponents   = require("Modules.Physics.Components")
local CelestialComponents = require("Modules.CelestialObjects.Components")

--- CoordinateRebaser — rebases entity hierarchies from absolute coordinates
--- to local coordinates near the origin. Critical for physics precision at
--- large distances (f64 loses sub-unit precision beyond ~1e15).
---@class CoordinateRebaser
local CoordinateRebaser = {}

--- Rebase the first star system and all its children to local coordinates
---@return Position systemOrigin The absolute origin that was subtracted
function CoordinateRebaser:rebaseStarSystem()
    local origin = Position(0, 0, 0)

    for starSystem in Registry:view(CelestialComponents.StarSystem) do
        local transform = starSystem:get(PhysicsComponents.Transform)
        origin = transform:getPos()

        self:_rebaseEntity(starSystem, origin)
        Log.Info("Rebased star system from (%.0f, %.0f, %.0f) to local origin",
            origin.x, origin.y, origin.z)
        break
    end

    return origin
end

--- Recursively subtract origin from all entity positions
---@param entity Entity
---@param origin Position
function CoordinateRebaser:_rebaseEntity(entity, origin)
    local transform = entity:get(PhysicsComponents.Transform)
    if transform then
        local pos = transform:getPos()
        transform:setPos(Position(pos.x - origin.x, pos.y - origin.y, pos.z - origin.z))
    end

    local childrenCmp = entity:get(CoreComponents.Children)
    if childrenCmp then
        for child in childrenCmp:iterChildren() do
            self:_rebaseEntity(child, origin)
        end
    end
end

return CoordinateRebaser
