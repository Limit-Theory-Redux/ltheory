local Component = require("Core.ECS.Component")

---@class CameraDataComponent: Component
---@overload fun(self: CameraDataComponent): CameraDataComponent subclass internal
---@overload fun(): CameraDataComponent subclass external
local CameraDataComponent = Subclass("CameraDataComponent", Component, function(self)
    ---@cast self CameraDataComponent
    self:setComponentName("RenderingCameraData")

    self:init()
end)

function CameraDataComponent:init()
    -- self.x         = 0
    -- self.y         = 0
    -- self.sx        = 1
    -- self.sy        = 1
    -- self.pos       = Position()
    -- self.rot       = Quat.Identity()
    -- self.posT      = Position()
    -- self.rotT      = Quat.Identity()
    -- self.posOffset = Position()
    -- self.rotOffset = Quat.Identity()
    self.zNear             = Config.render.camera.zNear
    self.zFar              = Config.render.camera.zFar
    self.view              = Matrix.Identity()
    self.projection        = Matrix.Identity()
    self.viewInverse       = Matrix.Identity()
    self.projectionInverse = Matrix.Identity()
    self.controller        = nil
end

---@param view Matrix
function CameraDataComponent:setView(view)
    self.view = view
end

---@param projection Matrix
function CameraDataComponent:setProjection(projection)
    self.projection = projection
end

---@param viewInverse Matrix
function CameraDataComponent:setViewInverse(viewInverse)
    self.viewInverse = viewInverse
end

---@param projectionInverse Matrix
function CameraDataComponent:setProjectionInverse(projectionInverse)
    self.projectionInverse = projectionInverse
end

---@param controller CameraController
function CameraDataComponent:setController(controller)
    self.controller = controller
end

---@return Matrix
function CameraDataComponent:getView()
    return self.view
end

---@return Matrix
function CameraDataComponent:getProjection()
    return self.projection
end

---@return Matrix
function CameraDataComponent:getViewInverse()
    return self.viewInverse
end

---@return Matrix
function CameraDataComponent:getProjectionInverse()
    return self.projectionInverse
end

---@return CameraController|nil
function CameraDataComponent:getController()
    return self.controller
end

function CameraDataComponent:hasController()
    return self.controller ~= nil
end

return CameraDataComponent
