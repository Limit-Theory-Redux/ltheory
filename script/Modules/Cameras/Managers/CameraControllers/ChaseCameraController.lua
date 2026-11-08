local CameraController = require("Modules.Cameras.Managers").CameraController
local PhysicsComponents = require("Modules.Physics.Components")
local FlightProfileComponent = require("Modules.Constructs.Components").FlightProfile

--- Chase camera — follows behind the ship, smoothly interpolating its offset from the ship.
--- The offset stretches with the ship's speed, up to `1 + maxLag` times the rest distance.
--- Design: doc/script/cameras.md
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
    -- Speed trail: the offset grows by up to `maxLag` times the rest distance, on a log curve of
    -- the speed between the reference and top speeds (both multiples of the profile's max speed).
    self.maxLag              = config.maxLag or 1.5
    self.trailReferenceRatio = config.trailReferenceRatio or 0.1
    self.trailTopSpeedRatio  = config.trailTopSpeedRatio
    assert(self.trailTopSpeedRatio and self.trailTopSpeedRatio > self.trailReferenceRatio,
        "ChaseCameraController: trailTopSpeedRatio must be above trailReferenceRatio")

    -- Smoothed offset from the ship and rotation (lerp targets)
    self.posT   = Position(0, 0, 0)
    self.rotT   = Quat.Identity()
    self.offsetSmooth = Vec3f(0, 0, 0)
    self.rotSmooth = Quat.Identity()
    self.initialized = false
end)

--- The trail's share of the rest distance at `speed`: 0 at rest, `maxLag` at `topSpeed` and above,
--- logarithmic in between so that speeds decades apart stay distinguishable.
---@param speed number Game units per second, >= 0
---@param referenceSpeed number > 0
---@param topSpeed number > referenceSpeed
---@param maxLag number
---@return number
local function trailShare(speed, referenceSpeed, topSpeed, maxLag)
    local share = math.log(1 + speed / referenceSpeed) / math.log(1 + topSpeed / referenceSpeed)
    return maxLag * math.min(share, 1)
end

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
    local maxSpeed = FlightProfileComponent.ProfileOf(self.target).maxSpeed
    local trail = trailShare(rb:getSpeed(), maxSpeed * self.trailReferenceRatio,
        maxSpeed * self.trailTopSpeedRatio, self.maxLag)
    local scale = 0.25 * self.radius * shipRadius * (1 + trail)
    local pr = self.posRel

    -- Transform posRel from ship-local to world space (matches legacy toWorld)
    local offsetT = Vec3f(
        (shipRt.x * pr.x + shipUp.x * pr.y + shipFwd.x * pr.z) * scale,
        (shipRt.y * pr.x + shipUp.y * pr.y + shipFwd.y * pr.z) * scale,
        (shipRt.z * pr.x + shipUp.z * pr.y + shipFwd.z * pr.z) * scale
    )
    self.posT = Position(shipPos.x + offsetT.x, shipPos.y + offsetT.y, shipPos.z + offsetT.z)

    -- Look-at point in world space (far ahead of ship)
    local la = self.lookAtRel
    local lookWorld = Vec3f(
        shipPos.x + (shipRt.x * la.x + shipUp.x * la.y + shipFwd.x * la.z),
        shipPos.y + (shipRt.y * la.x + shipUp.y * la.y + shipFwd.y * la.z),
        shipPos.z + (shipRt.z * la.x + shipUp.z * la.y + shipFwd.z * la.z)
    )

    -- Compute target rotation: look from camera toward ship (always valid direction)
    local lookDir = Vec3f(-offsetT.x, -offsetT.y, -offsetT.z)
    local len = lookDir:length()
    if len < 1e-6 then
        lookDir = shipFwd
    else
        lookDir = lookDir:normalize()
    end
    self.rotT = Quat.FromLook(lookDir, shipUp)

    -- Lerp smoothing (matches legacy: 1 - exp(-10 * dt)). The offset from the ship is smoothed,
    -- not the world position: a smoothed world position trails a moving ship by speed / 10.
    if not self.initialized then
        self.offsetSmooth = offsetT
        self.rotSmooth = self.rotT
        self.initialized = true
    else
        local lerpF = 1.0 - math.exp(-10.0 * dt)
        local o = self.offsetSmooth
        self.offsetSmooth = Vec3f(
            o.x + (offsetT.x - o.x) * lerpF,
            o.y + (offsetT.y - o.y) * lerpF,
            o.z + (offsetT.z - o.z) * lerpF
        )
        self.rotSmooth = self.rotSmooth:slerp(self.rotT, lerpF)
    end

    -- Set camera transform directly
    local o = self.offsetSmooth
    self.transform:setPos(Position(shipPos.x + o.x, shipPos.y + o.y, shipPos.z + o.z))
    self.transform:setRot(self.rotSmooth)
end

return ChaseCameraController
