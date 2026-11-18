local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Physics = require("Modules.Physics.Components")
local Cameras = require("Modules.Cameras.Components")
local CameraShake = require("Modules.Cameras.Managers.CameraShake")

---@class CameraManager
---@field profiler QuickProfiler
---@field cameras table<string, Entity> Registered cameras by name
---@field activeCamera string|nil Name of the currently active camera
---@field activeCameraData CameraDataComponent|nil
---@field activeCameraTransform TransformComponent|nil
---@overload fun(self: CameraManager): CameraManager class internal
---@overload fun(): CameraManager class external
local CameraManager = Class("CameraManager", function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
end)

---@private
function CameraManager:registerVars()
    self.profiler = QuickProfiler("CameraManager", false, false)

    ---@type table<string, Entity>
    self.cameras = {}

    ---@type string|nil
    self.activeCamera = nil

    ---@type CameraDataComponent|nil
    self.activeCameraData = nil

    ---@type TransformComponent|nil
    self.activeCameraTransform = nil

    ---@type Vec3f Primary light direction, given to the renderer in beginDraw
    self.starDir = Vec3f(0, 1, 0)

    -- View effects on top of the active camera (speed feel): a field of view
    -- offset and a shake. They change what is drawn, not the camera's
    -- transform, so steering and aiming don't see them.
    ---@type number degrees added to Config.render.camera.fov
    self.fovOffset = 0
    ---@type CameraShake
    self.shake = CameraShake()
    ---@type Quat|nil the active camera's rotation with the shake, as last drawn
    self.viewRot = nil
end

---The field of view the scene is drawn with: the setting plus the offset.
---Whatever projects onto the screen by hand (labels, lens flares) uses this.
---@return number fov degrees
function CameraManager:getFov()
    return Math.Clamp(Config.render.camera.fov + self.fovOffset, 10, 150)
end

---@param degrees number added to the field of view setting (0: none)
function CameraManager:setFovOffset(degrees)
    self.fovOffset = degrees
end

---@return number degrees
function CameraManager:getFovOffset()
    return self.fovOffset
end

---The shake drawn on top of the active camera (see CameraShake).
---@return CameraShake
function CameraManager:getShake()
    return self.shake
end

---The rotation the scene is drawn with: the active camera's, plus the shake.
---Whatever projects onto the screen by hand uses this; steering and aiming
---use the camera's own (`getForward`, the transform).
---@return Quat
function CameraManager:getViewRot()
    if self.viewRot then return self.viewRot end
    if not self.activeCameraTransform then return Quat.Identity() end
    return self.activeCameraTransform:getRot()
end

---Set the star (primary light) direction given to the renderer in beginDraw
---@param dir Vec3f Star direction vector
function CameraManager:setStarDir(dir)
    self.starDir = dir
end

---Get the current star direction
---@return Vec3f dir Current star direction
function CameraManager:getStarDir()
    return self.starDir
end

---Register a camera entity with a unique name
---@param name string Unique identifier for this camera
---@param entity Entity Camera entity with CameraData and Transform components
---@return boolean success True if registration succeeded
function CameraManager:registerCamera(name, entity)
    if not Registry:hasEntity(entity) then
        Log.Error("CameraManager: Cannot register invalid entity for camera '" .. name .. "'")
        return false
    end

    if not entity:get(Cameras.CameraData) then
        Log.Error("CameraManager: Entity missing CameraData component for camera '" .. name .. "'")
        return false
    end

    if not entity:get(Physics.Transform) then
        Log.Error("CameraManager: Entity missing Transform component for camera '" .. name .. "'")
        return false
    end

    if self.cameras[name] then
        Log.Warn("CameraManager: Overwriting existing camera '" .. name .. "'")
    end

    self.cameras[name] = entity
    Log.Info("CameraManager: Registered camera '" .. name .. "'")

    return true
end

---Unregister a camera by name
---@param name string Camera name to unregister
---@return boolean success True if camera was found and removed
function CameraManager:unregisterCamera(name)
    if not self.cameras[name] then
        Log.Warn("CameraManager: Cannot unregister unknown camera '" .. name .. "'")
        return false
    end

    -- If we're removing the active camera, clear it
    if self.activeCamera == name then
        self.activeCamera = nil
        self.activeCameraData = nil
        self.activeCameraTransform = nil
    end

    self.cameras[name] = nil
    Log.Info("CameraManager: Unregistered camera '" .. name .. "'")
    return true
end

---Set the active camera by name
---@param name string Name of the camera to activate
---@return boolean success True if camera was found and activated
function CameraManager:setActiveCamera(name)
    local camera = self.cameras[name]

    if not camera then
        Log.Error("CameraManager: Cannot activate unknown camera '" .. name .. "'")
        return false
    end

    if not Registry:hasEntity(camera) then
        Log.Error("CameraManager: Camera entity '" .. name .. "' no longer exists")
        self.cameras[name] = nil
        return false
    end

    if self.activeCamera == name then
        -- Already active
        return true
    elseif self.activeCamera then
        -- Disable previous camera controller
        local prevCamera = self.cameras[self.activeCamera]
        if prevCamera and prevCamera:isValid() then
            local prevCamData = prevCamera:get(Cameras.CameraData)
            if prevCamData and prevCamData:hasController() then
                local prevController = prevCamData:getController()
                ---@cast prevController CameraController
                prevController:disable()
            end
        end
    end

    self.activeCamera = name
    self.activeCameraData = camera:get(Cameras.CameraData)
    self.activeCameraTransform = camera:get(Physics.Transform)

    -- activate controller
    if self.activeCameraData:hasController() then
        local controller = self.activeCameraData:getController()
        ---@cast controller CameraController
        controller:enable()
    end

    Log.Info("CameraManager: Activated camera '" .. name .. "'")
    return true
end

---Get the currently active camera name
---@return string|nil name Name of active camera, or nil if none active
function CameraManager:getActiveCameraName()
    return self.activeCamera
end

---Get the currently active camera entity
---@return Entity|nil entity Active camera entity, or nil if none active
function CameraManager:getActiveCameraEntity()
    if not self.activeCamera then
        return nil
    end
    return self.cameras[self.activeCamera]
end

---Get a camera entity by name
---@param name string Camera name
---@return Entity|nil entity Camera entity, or nil if not found
function CameraManager:getCamera(name)
    return self.cameras[name]
end

---Check if a camera is registered
---@param name string Camera name to check
---@return boolean exists True if camera exists
function CameraManager:hasCamera(name)
    return self.cameras[name] ~= nil
end

---Get all registered camera names
---@return string[] names Array of camera names
function CameraManager:getCameraNames()
    local names = {}
    for name, _ in pairs(self.cameras) do
        table.insert(names, name)
    end
    return names
end

---Get the active camera's position (eye position)
---@return Position position Camera position
function CameraManager:getEye()
    if not self.activeCameraTransform then
        Log.Error("CameraManager: No active camera set")
        return Position.Identity()
    end
    return self.activeCameraTransform:getPos()
end

---Get the active camera's view matrix (camera-relative - position zeroed, see `updateViewMatrix`)
---@return Matrix
function CameraManager:getViewMatrix()
    if not self.activeCameraData then
        Log.Error("CameraManager: No active camera set")
        return Matrix.Identity()
    end
    return self.activeCameraData:getView()
end

---Get the active camera's projection matrix
---@return Matrix
function CameraManager:getProjectionMatrix()
    if not self.activeCameraData then
        Log.Error("CameraManager: No active camera set")
        return Matrix.Identity()
    end
    return self.activeCameraData:getProjection()
end

---Get the active camera's forward direction
---@return Vec3f direction Forward direction vector
function CameraManager:getForward()
    if not self.activeCameraTransform then
        Log.Error("CameraManager: No active camera set")
        return Vec3f(0, 0, -1)
    end
    return self.activeCameraTransform:getRot():getForward()
end

---Get the active camera's right direction
---@return Vec3f direction Right direction vector
function CameraManager:getRight()
    if not self.activeCameraTransform then
        Log.Error("CameraManager: No active camera set")
        return Vec3f(1, 0, 0)
    end
    return self.activeCameraTransform:getRot():getRight()
end

---Get the active camera's up direction
---@return Vec3f direction Up direction vector
function CameraManager:getUp()
    if not self.activeCameraTransform then
        Log.Error("CameraManager: No active camera set")
        return Vec3f(0, 1, 0)
    end
    return self.activeCameraTransform:getRot():getUp()
end

---Begin drawing with the active camera: every render pass that begins after
---this renders with its view/projection (and the star direction). Rendering is
---camera-relative, so the eye is the origin.
function CameraManager:beginDraw()
    if not self.activeCameraData then
        Log.Error("CameraManager: Cannot beginDraw without active camera")
        return
    end

    local camData = self.activeCameraData
    Renderer:setCamera(camData:getView(), camData:getProjection(), self.starDir)
end

---Update the view matrix from the active camera's transform
function CameraManager:updateViewMatrix()
    if not self.activeCameraData or not self.activeCameraTransform then
        return
    end

    local pos = self.activeCameraTransform:getPos()
    local rot = self.activeCameraTransform:getRot()

    -- The shake turns the view about the camera's own axes, the way the free
    -- camera turns itself: yaw about its up, pitch about its right, roll about
    -- its forward.
    local yaw, pitch, roll = self.shake:sample(Engine:getTime())
    if yaw ~= 0 or pitch ~= 0 or roll ~= 0 then
        local up, right, forward = rot:getUp(), rot:getRight(), rot:getForward()
        rot = Quat.FromAxisAngle(up, yaw):mul(rot)
        rot = Quat.FromAxisAngle(right, pitch):mul(rot)
        rot = Quat.FromAxisAngle(forward, roll):mul(rot)
    end
    self.viewRot = rot

    -- View inverse: world transform of camera
    local viewInv = Matrix.FromPosRot(Vec3f(pos.x, pos.y, pos.z), rot)

    -- View: inverse transform, with position relative to itself
    local view = Matrix.FromPosRot(
        pos:relativeTo(pos),
        rot:inverse()
    )

    self.activeCameraData:setViewInverse(viewInv)
    self.activeCameraData:setView(view)
end

---Update the projection matrix for the active camera
---@param resX number Screen width in pixels
---@param resY number Screen height in pixels
---@param fov? number Field of view in degrees (optional, default `getFov()`)
---@param zNear? number Near clip plane (optional, uses config default)
---@param zFar? number Far clip plane (optional, uses config default)
function CameraManager:updateProjectionMatrix(resX, resY, fov, zNear, zFar)
    if not self.activeCameraData then
        return
    end

    -- A minimized window reports a zero size; keep the last valid projection
    -- instead of building one from a NaN aspect ratio.
    if not resX or not resY or resX <= 0 or resY <= 0 then
        return
    end

    fov = fov or self:getFov()
    zNear = zNear or Config.render.camera.zNear
    zFar = zFar or Config.render.camera.zFar

    local proj = Matrix.Perspective(fov, resX / resY, zNear, zFar)
    self.activeCameraData:setProjection(proj)
    self.activeCameraData:setProjectionInverse(proj:inverse())
end

---Convert screen position to world-space ray
---@param screenPos Vec2f Screen position in pixels
---@param length? number Ray length (default: 1e7)
---@return Ray|nil ray World-space ray, or nil if no active camera
function CameraManager:screenToRay(screenPos, length)
    length = length or 1e7

    if not self.activeCameraData or not self.activeCameraTransform then
        return nil
    end

    -- Convert screen to NDC
    local ndc = Vec3f(
        2.0 * screenPos.x / Window:width() - 1.0,
        -(2.0 * screenPos.y / Window:height() - 1.0),
        -1.0
    )

    local viewInv = self.activeCameraData:getViewInverse()
    local projInv = self.activeCameraData:getProjectionInverse()

    -- Transform NDC points through inverse projection
    local near4 = Vec4f(ndc.x, ndc.y, -1.0, 1.0)
    local far4 = Vec4f(ndc.x, ndc.y, 1.0, 1.0)

    near4 = projInv:mulVec(near4)
    near4:idivs(near4.w)
    far4 = projInv:mulVec(far4)
    far4:idivs(far4.w)

    -- Transform through inverse view to get world positions
    local nearPoint = viewInv:mulPoint(near4:toVec3f())
    local farPoint = viewInv:mulPoint(far4:toVec3f())

    -- Calculate ray direction
    local dir = farPoint - nearPoint
    -- `not (>=)` also catches NaN from a degenerate projection
    if not (dir:length() >= 1e-6) then
        dir = self.activeCameraTransform:getRot():getForward()
    else
        dir = dir:normalize()
    end

    return Ray(
        nearPoint.x, nearPoint.y, nearPoint.z,
        dir.x, dir.y, dir.z,
        0.0,
        length
    )
end

---Convert mouse position to world-space ray
---@param length? number Ray length (default: 1e7)
---@return Ray|nil ray World-space ray, or nil if no active camera
function CameraManager:mouseToRay(length)
    length = length or 1e7
    local mp = Input:mouse():position()
    return self:screenToRay(Vec2f(mp.x, mp.y), length)
end

---Ray-sphere intersection test
---@param rayOrigin Vec3f Ray origin
---@param rayDir Vec3f Ray direction (normalized)
---@param sphereCenter Vec3f Sphere center position
---@param radius number Sphere radius
---@return number|nil distance Distance to intersection, or nil if no hit
function CameraManager:raySphereIntersect(rayOrigin, rayDir, sphereCenter, radius)
    local oc = rayOrigin - sphereCenter
    local a = rayDir:dot(rayDir)
    local b = 2.0 * oc:dot(rayDir)
    local c = oc:dot(oc) - radius * radius
    local discriminant = b * b - 4 * a * c

    if discriminant < 0 then
        return nil
    end

    local sqrtDisc = math.sqrt(discriminant)
    local t0 = (-b - sqrtDisc) / (2 * a)
    local t1 = (-b + sqrtDisc) / (2 * a)

    -- Return the closest positive intersection
    if t0 > 0 then
        return t0
    elseif t1 > 0 then
        return t1
    else
        return nil
    end
end

return CameraManager()
