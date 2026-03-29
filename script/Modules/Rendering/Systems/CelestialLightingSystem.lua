local PhysicsComponents = require("Modules.Physics.Components")
local CoreComponents    = require("Modules.Core.Components")
local CameraManager     = require("Modules.Cameras.Managers.CameraManager")
local RenderCoreSystem  = require("Modules.Rendering.Systems.RenderCoreSystem")

--- CelestialLightingSystem — finds the star entity and configures
--- directional lighting for the render pipeline.
---@class CelestialLightingSystem
local CelestialLightingSystem = {}

--- Star type to light color mapping
local starTypeColors = {
    RedGiant   = Vec3f(1.0, 0.4, 0.15),
    WhiteDwarf = Vec3f(0.8, 0.85, 1.0),
}
local defaultStarColor = Vec3f(1.0, 0.95, 0.8)

--- Update directional lighting from a star entity
---@param starEntity Entity
function CelestialLightingSystem:update(starEntity)
    if not starEntity then return end

    local starTransform = starEntity:get(PhysicsComponents.Transform)
    if not starTransform then return end
    local starPos = starTransform:getPos()

    -- Star light color based on type
    local typeCmp = starEntity:get(CoreComponents.Type)
    local starType = typeCmp and typeCmp:getSubtype() or "MainSequence"
    local color = starTypeColors[starType] or defaultStarColor

    -- Directional light from star position
    local eye = CameraManager:getEye()
    local toStar = starPos:relativeTo(eye)
    local dist = toStar:length()
    if dist < 1 then return end

    -- lightDir = FROM star toward scene (negate the to-star vector)
    local dir = Vec3f(-toStar.x / dist, -toStar.y / dist, -toStar.z / dist)

    RenderCoreSystem:setDirectionalLights({
        { dir = dir, color = color * 1.5 }
    })
end

return CelestialLightingSystem
