local Physics = require("Modules.Physics.Components")

---@class CameraController
---@field entity Entity The camera entity this controller manages
---@field transform TransformComponent Cached transform component
---@field enabled boolean Whether this controller is currently active
---@overload fun(self: CameraController): CameraController class internal
---@overload fun(): CameraController class external
local CameraController = Class("CameraController", function(self) end)

function CameraController:initController(entity)
    if not entity then
        Log.Error("CameraController: Entity is required")
        return
    end

    self.entity = entity
    self.transform = entity:get(Physics.Transform)
    self.enabled = true

    if not self.transform then
        Log.Error("CameraController: Entity missing Transform component")
    end
end

---Enable this controller
function CameraController:enable()
    self.enabled = true
end

---Disable this controller
function CameraController:disable()
    self.enabled = false
end

---Check if controller is enabled
---@return boolean enabled
function CameraController:isEnabled()
    return self.enabled
end

---Handle input events
---@param event table Input event data
function CameraController:onInput(event)
    -- Override in subclasses
end

---Get the camera's current position
---@return Position position
function CameraController:getPosition()
    return self.transform:getPos()
end

---Get the camera's current rotation
---@return Quat rotation
function CameraController:getRotation()
    return self.transform:getRot()
end

---Set the camera's position
---@param pos Position New position
function CameraController:setPosition(pos)
    self.transform:setPos(pos)
end

---Set the camera's rotation
---@param rot Quat New rotation
function CameraController:setRotation(rot)
    self.transform:setRot(rot)
end

---Get forward direction vector
---@return Vec3f forward
function CameraController:getForward()
    return self.transform:getRot():getForward()
end

---Get right direction vector
---@return Vec3f right
function CameraController:getRight()
    return self.transform:getRot():getRight()
end

---Get up direction vector
---@return Vec3f up
function CameraController:getUp()
    return self.transform:getRot():getUp()
end

return CameraController
