local Registry = require("Core.ECS.Registry")
local QuickProfiler = require("Shared.Tools.QuickProfiler")
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")

---@class CameraSystem
---@field profiler QuickProfiler
---@field currentCameraData CameraDataComponent
---@field currentCameraTransform TransformComponent
---@overload fun(self: CameraSystem): CameraSystem class internal
---@overload fun(): CameraSystem class external
local CameraSystem = Class("CameraSystem", function(self)
    ---@diagnostic disable-next-line: invisible
    self:registerVars()
    ---@diagnostic disable-next-line: invisible
    self:registerEvents()
end)

---@private
function CameraSystem:registerVars()
    self.profiler = QuickProfiler("CameraSystem", false, false)
    ---@type CameraDataComponent|nil
    self.currentCameraData = nil
    ---@type TransformComponent|nil
    self.currentCameraTransform = nil
end

---@private
function CameraSystem:registerEvents()
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
    --TODO: Disabled Camera Render/PostRender Subscriptions. ShaderVar's must be set by Camera at start of Render
    --EventBus:subscribe(Event.Render, self, self.onRender)
    --EventBus:subscribe(Event.PostRender, self, self.onPostRender)
end

---@private
function CameraSystem:onPreRender()
end

---@private
function CameraSystem:onRender()
    -- if self.currentCamera and self.currentCameraData and self.currentCameraTransform then
    --     self:beginDraw(self.currentCameraData, self.currentCameraTransform)
    -- end
end

---@private
function CameraSystem:onPostRender()
    -- if self.currentCamera and self.currentCameraData then
    --     self:endDraw()
    -- end
end

---@param entity Entity
function CameraSystem:setCamera(entity)
    if not Registry:hasEntity(entity) then
        return
    end

    self.currentCamera = entity.id
    self.currentCameraData = entity:get(Rendering.CameraData)
    self.currentCameraTransform = entity:get(Physics.Transform)
end

---Get Current Camera 'Eye'/Position
---@return Position
function CameraSystem:getCurrentCameraEye()
    if not self.currentCameraTransform then
        Log.Error("Attempted to getCameraEye() with no current Camera set.")
    end
    return self.currentCameraTransform:getPos()
end

function CameraSystem:beginDraw()
    local camData = self.currentCameraData
    ShaderVar.PushMatrix('mView', camData:getView())
    ShaderVar.PushMatrix('mViewInv', camData:getViewInverse())
    ShaderVar.PushMatrix('mProj', camData:getProjection())
    ShaderVar.PushMatrix('mProjInv', camData:getProjectionInverse())
    ShaderVar.PushFloat3('eye', 0.0, 0.0, 0.0)
end

function CameraSystem:endDraw()
    ShaderVar.Pop('mView')
    ShaderVar.Pop('mViewInv')
    ShaderVar.Pop('mProj')
    ShaderVar.Pop('mProjInv')
    ShaderVar.Pop('eye')
end

-- Updates view matrix from current transform
function CameraSystem:updateViewMatrix()
    if not self.currentCameraData or not self.currentCameraTransform then return end
    local pos = self.currentCameraTransform:getPos()
    local rot = self.currentCameraTransform:getRot()
    local viewInv = Matrix.FromPosRot(pos:relativeTo(self:getCurrentCameraEye()), rot)
    self.currentCameraData:setViewInverse(viewInv)
    self.currentCameraData:setView(viewInv:inverse())
end

-- Updates projection matrix
function CameraSystem:updateProjectionMatrix(resX, resY)
    if not self.currentCameraData then return end
    local fov = Config.render.camera.fov
    local zNear = Config.render.camera.zNear
    local zFar = Config.render.camera.zFar
    local proj = Matrix.Perspective(fov, resX / resY, zNear, zFar)
    self.currentCameraData:setProjection(proj)
    self.currentCameraData:setProjectionInverse(proj:inverse())
end

---@param screenPos Vec2f
---@param length number
---@return Ray?
function CameraSystem:screenToRay(screenPos, length)
    length = length or 1e7
    if not self.currentCameraData or not self.currentCameraTransform then return nil end

    -- Convert screen to NDC
    local ndc     = Vec3f(
        2.0 * screenPos.x / Window:width() - 1.0,
        -(2.0 * screenPos.y / Window:height() - 1.0),
        -1.0
    )

    local viewInv = self.currentCameraData:getViewInverse()
    local projInv = self.currentCameraData:getProjectionInverse()

    local near4   = Vec4f(ndc.x, ndc.y, -1, 1)
    local far4    = Vec4f(ndc.x, ndc.y, 1, 1)

    -- Apply inverse projection
    near4         = projInv:mulVec(near4)
    far4          = projInv:mulVec(far4)

    -- Perspective divide
    near4:idivs(near4.w)
    far4:idivs(far4.w)

    -- Apply inverse view
    local nearPoint = viewInv:mulPoint(near4:toVec3f())
    local farPoint  = viewInv:mulPoint(far4:toVec3f())


    -- Direction
    local dir = farPoint - nearPoint
    if dir:length() < 1e-8 then
        -- degenerate ray, push slightly forward along camera forward
        dir = self.currentCameraTransform:getRot():getForward():imuls(1e-6)
    end
    dir = dir:normalize()

    return Ray(nearPoint.x, nearPoint.y, nearPoint.z, dir.x, dir.y, dir.z, 0, length)
end

-- Ray-sphere intersection (returns nil if no hit)
---@param rayOrigin Vec3f
---@param rayDir Vec3f
---@param sphereCenter Vec3f
---@param radius number
---@return number? distance or nil
function CameraSystem:raySphereIntersect(rayOrigin, rayDir, sphereCenter, radius)
    local oc = rayOrigin - sphereCenter
    local a = rayDir:dot(rayDir)
    local b = 2.0 * oc:dot(rayDir)
    local c = oc:dot(oc) - radius * radius
    local discriminant = b * b - 4 * a * c

    if discriminant < 0 then return nil end

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

-- Converts mouse position to a world-space ray
---@param length number optional, default 1e7
---@return Ray
function CameraSystem:mouseToRay(length)
    length = length or 1e7
    if not self.currentCameraData or not self.currentCameraTransform then return nil end

    local mp = Input:mouse():position()
    return self:screenToRay(Vec2f(mp.x, mp.y), length)
end

-- function CameraSystem:lerpFrom(pos, rot)
--     self.posOffset = pos + self.posT:inverse()
--     self.rotOffset = rot * self.rotT:inverse()
-- end
--
-- function CameraSystem:cancelLerp()
--     self.posOffset = Position.Identity()
--     self.rotOffset = Quat.Identity()
-- end
--
-- function CameraSystem:lerp(dt)
--     local f = 1.0 - exp(-10.0 * dt)
--     self.posOffset:ilerp(Position.Identity(), f)
--     self.rotOffset:iLerp(Quat.Identity(), f)
-- end
--
-- -- Fundamental Transformations -------------------------------------------------
-- -- NOTE : These are all for *positions* not *directions*
-- -- NOTE : 'window' means the OpenGL window
-- -- NOTE : 'screen' means the camera widget, which may be offset and resized within the window
--
-- function CameraSystem:windowToScreen(wnd)
--     local ss = Vec2f()
--     ss.x = wnd.x - self.x
--     ss.y = wnd.y - self.y
--     return ss
-- end
--
-- function CameraSystem:screenToNDC(ss)
--     local ndc = Vec3f()
--     ndc.x = 2.0 * ss.x / self.sx - 1.0
--     ndc.y = -(2.0 * ss.y / self.sy - 1.0)
--     ndc.z = -1.0
--     return ndc
-- end
--
-- -- BUG : ndc.z = 1 gives NaNs when zNear == 0.1 and zFar == 1e7. Expect 0.1
-- -- BUG : ndc.z = 1 gives 9,586,980 when zNear == 10 and zFar == 1e7. Expect 10,000,000
-- function CameraSystem:ndcToView(ndc)
--     local vs4 = self.mProjInv:mulVec(Vec4f(ndc.x, ndc.y, ndc.z, 1.0))
--     local vs  = vs4:divs(vs4.w):toVec3f()
--     return vs
-- end
--
-- function CameraSystem:viewToWorld(vs)
--     local ws = self.mViewInv:mulPoint(vs) + self.pos
--     return ws
-- end
--
-- function CameraSystem:worldToView(ws)
--     local vs = self.mView:mulPoint(ws:relativeTo(self.pos))
--     return vs
-- end
--
-- function CameraSystem:viewToNDC(vs)
--     local ndc4 = self.mProj:mulVec(Vec4f(vs.x, vs.y, vs.z, 1.0))
--     local ndc  = ndc4:divs(ndc4.w):toVec3f()
--     return ndc, Math.Sign(ndc4.w)
-- end
--
-- function CameraSystem:ndcToScreen(ndc)
--     local ss = Vec2f()
--     ss.x = self.sx * (ndc.x + 1.0) / 2.0
--     ss.y = self.sy * (-ndc.y + 1.0) / 2.0
--     return ss
-- end
--
-- function CameraSystem:screenToWindow(ss)
--     local wnd = Vec2f()
--     wnd.x = ss.x + self.x
--     wnd.y = ss.y + self.y
--     return wnd
-- end
--
-- --------------------------------------------------------------------------------
--
-- -- Helper Transformations ------------------------------------------------------
-- -- NOTE : These are all for *positions* not *directions*
--
-- -- OPTIMIZE : Creating a table is maybe not so great
-- function CameraSystem:entityToScreenRect(entity)
--     local box = entity:getBoundingBoxLocal()
--     local points = {
--         Vec3f(box.lowerx, box.lowery, box.lowerz),
--         Vec3f(box.upperx, box.lowery, box.lowerz),
--         Vec3f(box.lowerx, box.uppery, box.lowerz),
--         Vec3f(box.upperx, box.uppery, box.lowerz),
--         Vec3f(box.lowerx, box.lowery, box.upperz),
--         Vec3f(box.upperx, box.lowery, box.upperz),
--         Vec3f(box.lowerx, box.uppery, box.upperz),
--         Vec3f(box.upperx, box.uppery, box.upperz),
--     }
--
--     local xMin, yMin, xMax, yMax = math.huge, math.huge, -math.huge, -math.huge
--     for i = 1, #points do
--         local ws   = entity:toWorld(points[i])
--         local vs   = self:worldToView(ws)
--         local ndc  = self:viewToNDC(vs)
--         local ss   = self:ndcToScreen(ndc)
--
--         xMin, yMin = min(xMin, ss.x), min(yMin, ss.y)
--         xMax, yMax = max(xMax, ss.x), max(yMax, ss.y)
--     end
--
--     return xMin, yMin, xMax - xMin, yMax - yMin
-- end
--
-- function CameraSystem:ndcToRay(ndc, length)
--     ndc.z = 0.9
--     local vs = self:ndcToView(ndc)
--     local ws = self:viewToWorld(vs)
--
--     -- NOTE : Calculate dir in View Space to avoid catastrophic cancellation
--     ndc.z = 0.99
--     local vs_p1 = self:ndcToView(ndc)
--     local vs_dir = vs_p1 - vs
--
--     -- NOTE: We now need to test inputs to normalize() to prevent near-zero inputs
--     local dir = self.mViewInv:mulDir(vs_dir)
--     if dir:length() >= 0.00000001 then
--         dir = dir:normalize()
--     end
--
--     return Ray(ws.x, ws.y, ws.z, dir.x, dir.y, dir.z, 0, length)
-- end
--
-- function CameraSystem:mouseToRay(length)
--     local mp  = Input:mouse():position()
--     local ss  = self:windowToScreen(mp)
--     local ndc = self:screenToNDC(ss)
--     local ray = self:ndcToRay(ndc, length)
--     return ray
-- end
--
-- -- NOTE : NDC.z is +/- 1, indicating in front or behind the near plane.
-- function CameraSystem:worldToNDC(ws)
--     local vs     = self:worldToView(ws)
--     local ndc, w = self:viewToNDC(vs)
--     ndc.z        = w
--     return ndc
-- end
--
-- --------------------------------------------------------------------------------
--
-- function CameraSystem:refreshMatrices()
--     self.pos = self.posOffset + self.posT
--     self.rot = self.rotOffset * self.rotT
--
--     -- View matrix has the "position" at (0,0,0), as all world matrices are offset by self.pos.
--     self.mViewInv = Matrix.FromPosRot(Vec3f.Identity(), self.rot)
--     self.mView = self.mViewInv:inverse()
--
--     self.mProj = Matrix.Perspective(
--         GameState.render.fov,
--         self.sx / self.sy,
--         self.zNear,
--         self.zFar)
--     self.mProjInv = self.mProj:inverse()
-- end
--

---Deprecated function. Only ever set never used.
---@deprecated
function CameraSystem:setViewport(x, y, sx, sy) end

-- function CameraSystem:warp() end

return CameraSystem()
