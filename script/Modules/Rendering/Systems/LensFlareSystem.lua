local PhysicsComponents = require("Modules.Physics.Components")
local CameraManager     = require("Modules.Cameras.Managers.CameraManager")

--- LensFlareSystem — renders screen-space lens flare for a bright light source.
--- Handles projection, geometric occlusion, distance falloff, and rendering.
--- General entry point: drawAt(worldPos, opts) works for ANY bright source
--- (explosion flash, sun, muzzle flash); draw(entity, ...) is a thin
--- entity-bound wrapper over it.
---@class LensFlareSystem
local LensFlareSystem = {}

--- Draw lens flare for a world-space position (general API).
---@param worldPos Position world position of the bright source
---@param opts? table { color = {r,g,b}, intensity = number (multiplier,
---  default 1), maxDist = number, occluders = table, ghosts = number (0-1),
---  halo = number (0-1), xStreak = number (0-1) }
function LensFlareSystem:drawAt(worldPos, opts)
    opts = opts or {}
    if not worldPos then return end

    -- Get camera
    local camEntity = CameraManager:getActiveCameraEntity()
    if not camEntity then return end
    local camTransform = camEntity:get(PhysicsComponents.Transform)
    if not camTransform then return end
    local camPos = camTransform:getPos()
    local camRot = camTransform:getRot()

    -- Check if light is in front of camera
    local rel = worldPos:relativeTo(camPos)
    local camFwd = camRot:getForward()
    local dot = rel.x * camFwd.x + rel.y * camFwd.y + rel.z * camFwd.z
    if dot <= 0 then return end

    -- Project to screen
    local camRight = camRot:getRight()
    local camUp = camRot:getUp()
    local fovRad = math.rad(Config.render.camera.fov)
    local tanHalfFov = math.tan(fovRad / 2)
    local aspect = Window:width() / Window:height()

    local sx = (rel.x * camRight.x + rel.y * camRight.y + rel.z * camRight.z) / (dot * tanHalfFov * aspect)
    local sy = (rel.x * camUp.x + rel.y * camUp.y + rel.z * camUp.z) / (dot * tanHalfFov)

    -- Normalized screen position [0, 1]
    local screenU = (sx + 1) * 0.5
    local screenV = (1 - sy) * 0.5

    -- Only draw if reasonably on screen
    if screenU < -0.5 or screenU > 1.5 or screenV < -0.5 or screenV > 1.5 then return end

    local maxDist = opts.maxDist or 5000000
    local dist = rel:length()
    if dist > maxDist then return end

    -- Geometric occlusion: check if any occluder disc covers the light's screen position
    local lightDir = rel:normalize()
    for _, entry in ipairs(opts.occluders or {}) do
        local entTransform = entry.entity:get(PhysicsComponents.Transform)
        if entTransform then
            local entPos = entTransform:getPos()
            local toCam = entPos:relativeTo(camPos)
            local entDist = toCam:length()

            -- Only check objects between camera and light
            if entDist > 10 and entDist < dist then
                local entDir = toCam:normalize()
                local dotProduct = lightDir.x * entDir.x + lightDir.y * entDir.y + lightDir.z * entDir.z
                if dotProduct > 0.99 then
                    local entScale = entTransform:getScale() or 0
                    local angularRadius = entScale / math.max(1, entDist)
                    local angularSep = math.acos(Math.Clamp(dotProduct, -1, 1))
                    if angularSep < angularRadius then
                        return -- light is behind this object
                    end
                end
            end
        end
    end

    -- Distance falloff: quadratic
    local distFactor = 1.0 - (dist / maxDist)
    distFactor = distFactor * distFactor

    -- Look-at factor: stronger when centered on screen
    local screenCenterDist = math.sqrt((screenU - 0.5) ^ 2 + (screenV - 0.5) ^ 2)
    local lookAtFactor = Math.Clamp(1.0 - screenCenterDist * 2.0, 0.05, 1.0)
    lookAtFactor = lookAtFactor * lookAtFactor

    local intensity = lookAtFactor * distFactor * 1.5

    -- Fade at screen edges
    local edgeFade = Math.Clamp(1.0 - math.max(
        math.max(0, math.abs(screenU - 0.5) - 0.45) * 10,
        math.max(0, math.abs(screenV - 0.5) - 0.45) * 10
    ), 0, 1)
    intensity = intensity * edgeFade

    if intensity < 0.01 then return end

    -- Draw lens flare
    local resX, resY = Window:width(), Window:height()
    local tint = opts.color or { 1.0, 0.9, 0.7 }
    RenderState.PushBlendMode(BlendMode.Additive)
    local shader = Cache.Shader('ui', 'filter/lensflare')
    shader:start()
    shader:setFloat2('lightPos', screenU, screenV)
    shader:setFloat3('lightColor', tint[1], tint[2], tint[3])
    shader:setFloat('intensity', intensity * (opts.intensity or 1.0))
    shader:setFloat2('screenSize', resX, resY)
    shader:setFloat('xStreak', opts.xStreak
        or Math.Clamp((intensity - 0.4) * 3.0, 0, 1))
    shader:setFloat('ghosts', opts.ghosts or 1.0)
    shader:setFloat('halo', opts.halo or 1.0)
    Draw.Rect(0, 0, resX, resY)
    shader:stop()
    RenderState.PopBlendMode()
end

--- Entity-bound wrapper: project the entity's transform position.
---@param lightEntity Entity The bright object (star)
---@param occluders table Array of { entity = Entity } entries that can occlude the light
---@param maxDist number|nil Maximum distance for flare visibility
---@param opts? table extra drawAt options (color, intensity, ghosts, halo)
function LensFlareSystem:draw(lightEntity, occluders, maxDist, opts)
    if not lightEntity then return end
    local lightTransform = lightEntity:get(PhysicsComponents.Transform)
    if not lightTransform then return end
    opts = opts or {}
    opts.occluders = occluders
    if maxDist then opts.maxDist = maxDist end
    self:drawAt(lightTransform:getPos(), opts)
end

return LensFlareSystem
