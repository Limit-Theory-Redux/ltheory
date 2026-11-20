local PhysicsComponents   = require("Modules.Physics.Components")
local CameraManager       = require("Modules.Cameras.Managers.CameraManager")
local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")
local DrawEx              = require("UI.DrawEx")
local ScreenProjection    = require("Shared.Helpers.ScreenProjection")

-- A label this far outside the window is still drawn, so one that straddles the edge doesn't pop.
local LABEL_MARGIN_PX = 100

-- Reused each frame.
local labelProjector = {}

--- WorldLabelRenderSystem — projects 3D entity labels onto the screen.
---@class WorldLabelRenderSystem
local WorldLabelRenderSystem = {}

--- Draw labels for a list of labeled entities
---@param labeledEntities table Array of { entity, label, isMoon, parentPos }
---@param moonDistThreshold number|nil Max camera distance to parent for moon labels (default 100000)
function WorldLabelRenderSystem:draw(labeledEntities, moonDistThreshold)
    if not labeledEntities then return end
    moonDistThreshold = moonDistThreshold or 100000

    local camEntity = CameraManager:getActiveCameraEntity()
    if not camEntity then return end
    local camTransform = camEntity:get(PhysicsComponents.Transform)
    if not camTransform then return end
    local camPos = camTransform:getPos()

    local projector = ScreenProjection.active(labelProjector)

    for _, entry in ipairs(labeledEntities) do
        -- Hide moon labels unless camera is near the parent planet
        if entry.isMoon and entry.parentPos then
            local dx = camPos.x - entry.parentPos.x
            local dy = camPos.y - entry.parentPos.y
            local dz = camPos.z - entry.parentPos.z
            local distToParent = math.sqrt(dx * dx + dy * dy + dz * dz)
            if distToParent > moonDistThreshold then goto next_label end
        end

        local entity = entry.entity
        local label = entry.label

        local transform = entity:get(PhysicsComponents.Transform)
        if not transform then goto next_label end
        local pos = transform:getPos()

        local dx, dy, dz = pos.x - camPos.x, pos.y - camPos.y, pos.z - camPos.z
        local dist = math.sqrt(dx * dx + dy * dy + dz * dz)

        -- nil when the point is behind the camera
        local px, py = ScreenProjection.relativeToScreen(projector, dx, dy, dz)
        if not px then goto next_label end

        -- Only draw if on screen
        if ScreenProjection.inViewport(projector, px, py, LABEL_MARGIN_PX) then
            local distStr = " [" .. UniverseScaleConfig:formatDistance(dist) .. "]"
            local text = label .. distStr
            DrawEx.TextAdditive('Unageo-Medium', text, 12, px - 50, py - 8, 200, 16,
                1.0, 1.0, 0.5, 0.9, 0.5, 0.5)
        end

        ::next_label::
    end
end

--- Draw labels for nearby asteroids from belt components
---@param beltEntities table Array of entities with AsteroidBeltDataComponent
---@param maxDist number|nil Maximum distance to show labels (default 500000)
function WorldLabelRenderSystem:drawAsteroidMarkers(beltEntities, maxDist)
    if not beltEntities then return end
    maxDist = maxDist or 500000

    local camEntity = CameraManager:getActiveCameraEntity()
    if not camEntity then return end
    local camTransform = camEntity:get(PhysicsComponents.Transform)
    if not camTransform then return end
    local camPos = camTransform:getPos()
    local projector = ScreenProjection.active(labelProjector)

    local CelestialComponents = require("Modules.CelestialObjects.Components")

    for _, beltEntity in ipairs(beltEntities) do
        local beltCmp = beltEntity:get(CelestialComponents.AsteroidBeltData)
        if not beltCmp then goto next_belt end

        local asteroids = beltCmp:getAsteroidData()
        for idx, a in ipairs(asteroids) do
            local rx = a.px - camPos.x
            local ry = a.py - camPos.y
            local rz = a.pz - camPos.z
            local dist = math.sqrt(rx*rx + ry*ry + rz*rz)

            if dist > maxDist then goto next_asteroid end

            -- nil when the point is behind the camera
            local px, py = ScreenProjection.relativeToScreen(projector, rx, ry, rz)
            if not px then goto next_asteroid end

            if ScreenProjection.inViewport(projector, px, py, LABEL_MARGIN_PX) then
                local distStr = " [" .. UniverseScaleConfig:formatDistance(dist) .. "]"
                local text = "Asteroid " .. idx .. distStr
                DrawEx.TextAdditive('Unageo-Medium', text, 10, px - 40, py - 6, 200, 14,
                    0.7, 0.5, 0.3, 0.7, 0.5, 0.5)
            end

            ::next_asteroid::
        end
        ::next_belt::
    end
end

return WorldLabelRenderSystem
