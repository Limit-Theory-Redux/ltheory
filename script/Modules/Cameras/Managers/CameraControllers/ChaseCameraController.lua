local CameraController = require("Modules.Cameras.Managers").CameraController
local PhysicsComponents = require("Modules.Physics.Components")

--- Chase camera — follows behind the ship, smoothly interpolating.
--- Direct port of Legacy CameraChase into ECS controller pattern.
---@class ChaseCameraController : CameraController
---@overload fun(entity: Entity, config?: table): ChaseCameraController
local ChaseCameraController = Subclass("ChaseCameraController", CameraController, function(self, entity, config)
    self:initController(entity)

    config = config or {}

    self.target    = nil
    self.posRel    = config.posRel or Vec3f(0, 2.5, -10)     -- Position behind/above ship (local space)
    self.lookAtRel = config.lookAtRel or Vec3f(0, 0, 1000)   -- Look-at point ahead of ship (local space)
    self.radius    = config.radius or 1.0                     -- Zoom multiplier
    self.radiusT   = self.radius
    self.minRadius = config.minRadius or 0.5
    self.maxRadius = config.maxRadius or 10.0
    self.zoomSpeed = config.zoomSpeed or 0.1

    -- Smoothed position and rotation (lerp targets)
    self.posT   = Position(0, 0, 0)
    self.rotT   = Quat.Identity()
    self.posSmooth = Position(0, 0, 0)
    self.rotSmooth = Quat.Identity()
    self.initialized = false
end)

---@param target Entity
function ChaseCameraController:setTarget(target)
    self.target = target
    self.initialized = false
end

function ChaseCameraController:modRadius(scrollDelta)
    -- Exponential zoom: each scroll tick multiplies radius by a fixed factor
    -- Feels proportional at all distances
    local factor = math.exp(-scrollDelta * self.zoomSpeed)
    self.radiusT = Math.Clamp(self.radiusT * factor, self.minRadius, self.maxRadius)
end

function ChaseCameraController:onInput(dt)
    if not self.enabled or not Window:isFocused() then return end

    -- Zoom with mouse wheel
    local scroll = Input:mouse():value(MouseControl.ScrollY)
    if math.abs(scroll) > 0.001 then
        self:modRadius(scroll)
    end
end

function ChaseCameraController:onPreRender(dt)
    if not self.enabled or not self.target then return end

    -- Smooth radius
    local f = 1.0 - math.exp(-8.0 * dt)
    self.radius = self.radius + (self.radiusT - self.radius) * f

    -- Get ship rigid body
    local rbCmp = self.target:get(PhysicsComponents.RigidBody)
    if not rbCmp then return end
    local rb = rbCmp:getRigidBody()
    if not rb then return end

    local shipPos = rb:getPos()
    local shipRot = rb:getRot()
    local shipFwd = shipRot:getForward()
    local shipRt  = shipRot:getRight()
    local shipUp  = shipRot:getUp()
    -- Scale relative position by ship radius and zoom. The camera
    -- distance is proportional to the target's actual radius (same
    -- convention as the benchmark's body zoom: dist = radius * factor),
    -- so a 50m fighter and a 500m cruiser both sit at the same multiple
    -- of their own size. A floor of 1 GU would treat every small ship as
    -- a 1 GU object and push the camera hundreds of ship-lengths away.
    local shipRadius = math.max(rb:getBoundingRadius(), 1e-4)
    local scale = 0.25 * self.radius * shipRadius
    local pr = self.posRel

    -- Transform posRel from ship-local to world space (matches legacy toWorld)
    self.posT = Position(
        shipPos.x + (shipRt.x * pr.x + shipUp.x * pr.y + shipFwd.x * pr.z) * scale,
        shipPos.y + (shipRt.y * pr.x + shipUp.y * pr.y + shipFwd.y * pr.z) * scale,
        shipPos.z + (shipRt.z * pr.x + shipUp.z * pr.y + shipFwd.z * pr.z) * scale
    )

    -- Look-at point in world space (far ahead of ship)
    local la = self.lookAtRel
    local lookWorld = Vec3f(
        shipPos.x + (shipRt.x * la.x + shipUp.x * la.y + shipFwd.x * la.z),
        shipPos.y + (shipRt.y * la.x + shipUp.y * la.y + shipFwd.y * la.z),
        shipPos.z + (shipRt.z * la.x + shipUp.z * la.y + shipFwd.z * la.z)
    )

    -- Compute target rotation: look from camera toward ship (always valid direction)
    local lookDir = Vec3f(
        shipPos.x - self.posT.x,
        shipPos.y - self.posT.y,
        shipPos.z - self.posT.z
    )
    local len = lookDir:length()
    if len < 1e-6 then
        lookDir = shipFwd
    else
        lookDir = lookDir:normalize()
    end
    self.rotT = Quat.FromLook(lookDir, shipUp)

    -- Lerp smoothing (matches legacy: 1 - exp(-10 * dt))
    if not self.initialized then
        self.posSmooth = self.posT
        self.rotSmooth = self.rotT
        self.initialized = true
    else
        local lerpF = 1.0 - math.exp(-10.0 * dt)
        self.posSmooth = Position(
            self.posSmooth.x + (self.posT.x - self.posSmooth.x) * lerpF,
            self.posSmooth.y + (self.posT.y - self.posSmooth.y) * lerpF,
            self.posSmooth.z + (self.posT.z - self.posSmooth.z) * lerpF
        )
        self.rotSmooth = self.rotSmooth:slerp(self.rotT, lerpF)
    end

    -- Set camera transform directly
    self.transform:setPos(self.posSmooth)
    self.transform:setRot(self.rotSmooth)
end

return ChaseCameraController
