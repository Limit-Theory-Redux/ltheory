---@class RenderState
---@field cameraEye Position

---@class RenderState
---@overload fun(self): RenderState class internal
---@overload fun(): RenderState class external
local RenderState = Class(function(self) 
    self.cameraEye = Position()
end)

---@param pos Position
function RenderState:setCameraEye(pos)
    self.cameraEye = pos
end

---@return Position CameraEye
function RenderState:getCameraEye()
    return self.cameraEye
end

return RenderState()