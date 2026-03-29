local PhysicsComponents   = require("Modules.Physics.Components")
local CameraManager       = require("Modules.Cameras.Managers.CameraManager")
local UniverseScaleConfig = require("Config.Gen.UniverseScaleConfig")
local DrawEx              = require("UI.DrawEx")

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

    local resX = Window:width()
    local resY = Window:height()

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

        -- Compute relative position from camera
        local rel = Vec3f(pos.x - camPos.x, pos.y - camPos.y, pos.z - camPos.z)
        local dist = rel:length()

        -- Get camera axes
        local camRot = camTransform:getRot()
        local camFwd = camRot:getForward()
        local camRight = camRot:getRight()
        local camUp = camRot:getUp()

        -- Skip if behind camera
        local dot = rel.x * camFwd.x + rel.y * camFwd.y + rel.z * camFwd.z
        if dot < 0 then goto next_label end

        -- Project onto screen axes
        local fovRad = math.rad(Config.render.camera.fov)
        local tanHalfFov = math.tan(fovRad / 2)
        local aspect = resX / resY

        local screenX = (rel.x * camRight.x + rel.y * camRight.y + rel.z * camRight.z) / (dot * tanHalfFov * aspect)
        local screenY = (rel.x * camUp.x + rel.y * camUp.y + rel.z * camUp.z) / (dot * tanHalfFov)

        -- Convert from [-1,1] to screen pixels
        local px = (screenX + 1) * 0.5 * resX
        local py = (1 - screenY) * 0.5 * resY

        -- Only draw if on screen
        if px >= -100 and px <= resX + 100 and py >= -50 and py <= resY + 50 then
            local distStr = " [" .. UniverseScaleConfig:formatDistance(dist) .. "]"
            local text = label .. distStr
            DrawEx.TextAdditive('Unageo-Medium', text, 12, px - 50, py - 8, 200, 16,
                1.0, 1.0, 0.5, 0.9, 0.5, 0.5)
        end

        ::next_label::
    end
end

return WorldLabelRenderSystem
