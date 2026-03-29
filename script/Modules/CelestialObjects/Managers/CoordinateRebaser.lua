local CoreComponents    = require("Modules.Core.Components")
local PhysicsComponents = require("Modules.Physics.Components")

--- CoordinateRebaser — rebases entity hierarchies from absolute coordinates
--- to local coordinates near the origin. Critical for physics precision at
--- large distances (f64 loses sub-unit precision beyond ~1e15).
---@class CoordinateRebaser
local CoordinateRebaser = {}

--- Find the first star system and rebase all its children to local coordinates
---@param universeEntity Entity Root universe entity
---@return Position systemOrigin The absolute origin that was subtracted
function CoordinateRebaser:rebaseStarSystem(universeEntity)
    local origin = Position(0, 0, 0)

    local universeChildren = universeEntity:get(CoreComponents.Children)
    if not universeChildren then return origin end

    for child in universeChildren:iterChildren() do
        local name = tostring(child)
        if name:find("StarSystemEntity") then
            local transform = child:get(PhysicsComponents.Transform)
            origin = transform:getPos()

            self:_rebaseEntity(child, origin)
            Log.Info("Rebased star system from (%.0f, %.0f, %.0f) to local origin",
                origin.x, origin.y, origin.z)
            break
        end
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
