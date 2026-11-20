-- Checks the shared world-to-screen projection against a real camera and its matrices: a point on
-- the view axis lands at the viewport's centre (also far from the origin and in an offset viewport),
-- right is +x and up is -y, a point at half the field of view lands on the edge, a point behind the
-- eye gives nothing, and a rotated camera agrees with the projection by camera axes. Logs each
-- failed check and ends with a failure exit code.

local Application       = require("States.Application")
local PhysicsComponents = require("Modules.Physics.Components")
local CameraEntity      = require("Modules.Cameras.Entities").Camera
local CameraManager     = require("Modules.Cameras.Managers.CameraManager")
local ScreenProjection  = require("Shared.Helpers.ScreenProjection")

---@class UIScreenProjectionTest: Application
local ScreenProjectionTest = Subclass("UIScreenProjectionTest", Application)

local CAMERA_NAME = "ScreenProjectionTest"
-- Pixels; the matrices are single precision.
local PIXEL_TOLERANCE = 0.05
local AHEAD = 100
local SIDE = 10
-- An eye far from the origin: the f64 subtraction must keep the small offsets exact.
local FAR_EYE = Position(1.0e8, 2.0e8, -3.0e8)
local YAW, PITCH = 0.7, 0.3
local VIEWPORT = { x = 40, y = 30, w = 640, h = 360 }

function ScreenProjectionTest:check(ok, what, detail)
    if ok then
        self.passed = self.passed + 1
    else
        self.failed = self.failed + 1
        Log.Error("UIScreenProjectionTest: %s failed%s", what, detail and (": " .. detail) or "")
    end
end

local function near(a, b)
    return a ~= nil and math.abs(a - b) <= PIXEL_TOLERANCE
end

---Places the camera at `eye` with `rot` and refreshes its matrices.
function ScreenProjectionTest:placeCamera(eye, rot)
    local transform = self.camera:get(PhysicsComponents.Transform)
    transform:setPos(eye)
    transform:setRot(rot)
    CameraManager:updateViewMatrix()
    CameraManager:updateProjectionMatrix(Window:width(), Window:height())
    self.eye, self.rot = eye, rot
end

---The point `ahead` along the view axis, `right` to its right and `up` above it.
function ScreenProjectionTest:pointAt(ahead, right, up)
    local fwd, rgt, upv = self.rot:getForward(), self.rot:getRight(), self.rot:getUp()
    return Position(
        self.eye.x + fwd.x * ahead + rgt.x * right + upv.x * up,
        self.eye.y + fwd.y * ahead + rgt.y * right + upv.y * up,
        self.eye.z + fwd.z * ahead + rgt.z * right + upv.z * up)
end

---The pixel by camera axes: the projection the labels used before the shared one.
function ScreenProjectionTest:axisProjection(pos)
    local rel = Vec3f(pos.x - self.eye.x, pos.y - self.eye.y, pos.z - self.eye.z)
    local fwd, rgt, upv = self.rot:getForward(), self.rot:getRight(), self.rot:getUp()
    local depth = rel.x * fwd.x + rel.y * fwd.y + rel.z * fwd.z
    local tanHalfFov = math.tan(math.rad(Config.render.camera.fov) / 2)
    local w, h = Window:width(), Window:height()
    local sx = (rel.x * rgt.x + rel.y * rgt.y + rel.z * rgt.z) / (depth * tanHalfFov * (w / h))
    local sy = (rel.x * upv.x + rel.y * upv.y + rel.z * upv.z) / (depth * tanHalfFov)
    return (sx + 1) * 0.5 * w, (1 - sy) * 0.5 * h
end

function ScreenProjectionTest:checkAxes(eye, rot, what)
    self:placeCamera(eye, rot)
    local projector = ScreenProjection.active()
    local cx, cy = Window:width() / 2, Window:height() / 2

    local px, py = ScreenProjection.toScreen(projector, self:pointAt(AHEAD, 0, 0))
    self:check(near(px, cx) and near(py, cy), what .. ": a point on the view axis is at the centre",
        string.format("%s, %s, expected %g, %g", tostring(px), tostring(py), cx, cy))

    local rx, ry = ScreenProjection.toScreen(projector, self:pointAt(AHEAD, SIDE, 0))
    self:check(rx and rx > cx and near(ry, cy), what .. ": right is +x", tostring(rx))
    local ux, uy = ScreenProjection.toScreen(projector, self:pointAt(AHEAD, 0, SIDE))
    self:check(uy and uy < cy and near(ux, cx), what .. ": up is -y", tostring(uy))

    local tanHalfFov = math.tan(math.rad(Config.render.camera.fov) / 2)
    local ex, ey = ScreenProjection.toScreen(projector, self:pointAt(AHEAD, 0, AHEAD * tanHalfFov))
    self:check(near(ex, cx) and near(ey, 0), what .. ": half the field of view is at the top edge",
        string.format("%s, %s", tostring(ex), tostring(ey)))

    local bx, by = ScreenProjection.toScreen(projector, self:pointAt(-AHEAD, 0, 0))
    self:check(bx == nil and by == nil, what .. ": a point behind the eye gives nothing")

    local ox, oy = ScreenProjection.relativeToScreen(projector,
        self.rot:getForward().x * AHEAD, self.rot:getForward().y * AHEAD, self.rot:getForward().z * AHEAD)
    self:check(near(ox, cx) and near(oy, cy), what .. ": an offset from the eye projects the same way")

    projector.x, projector.y, projector.w, projector.h = VIEWPORT.x, VIEWPORT.y, VIEWPORT.w, VIEWPORT.h
    local vx, vy = ScreenProjection.toScreen(projector, self:pointAt(AHEAD, 0, 0))
    self:check(near(vx, VIEWPORT.x + VIEWPORT.w / 2) and near(vy, VIEWPORT.y + VIEWPORT.h / 2),
        what .. ": the centre follows an offset viewport", string.format("%s, %s", tostring(vx), tostring(vy)))
    self:check(ScreenProjection.inViewport(projector, vx, vy)
        and not ScreenProjection.inViewport(projector, VIEWPORT.x - 1, vy)
        and ScreenProjection.inViewport(projector, VIEWPORT.x - 1, vy, 2), what .. ": the viewport test and its margin")
end

---Points around the view axis project as the camera-axes formula says.
function ScreenProjectionTest:checkAgainstAxes(eye, rot, what)
    self:placeCamera(eye, rot)
    local projector = ScreenProjection.active()
    for _, offset in ipairs({ { 50, 3, 2 }, { 400, -60, 25 }, { 20, 5, -4 }, { 1000, 250, -90 } }) do
        local pos = self:pointAt(offset[1], offset[2], offset[3])
        local px, py = ScreenProjection.toScreen(projector, pos)
        local ax, ay = self:axisProjection(pos)
        self:check(near(px, ax) and near(py, ay), what .. ": agrees with the camera axes",
            string.format("%s, %s against %g, %g", tostring(px), tostring(py), ax, ay))
    end
end

function ScreenProjectionTest:onInit()
    self.passed, self.failed = 0, 0

    self.camera = CameraEntity()
    CameraManager:registerCamera(CAMERA_NAME, self.camera)
    CameraManager:setActiveCamera(CAMERA_NAME)

    local identity = Quat.Identity()
    local tilted = Quat.FromAxisAngle(Vec3f(0, 1, 0), YAW):mul(Quat.FromAxisAngle(Vec3f(1, 0, 0), PITCH))
    self:checkAxes(Position(0, 0, 0), identity, "at the origin")
    self:checkAxes(FAR_EYE, identity, "far from the origin")
    self:checkAxes(FAR_EYE, tilted, "rotated, far from the origin")
    self:checkAgainstAxes(Position(10, -20, 30), tilted, "rotated")

    CameraManager:unregisterCamera(CAMERA_NAME)
    Log.Info("UIScreenProjectionTest: %d checks passed, %d failed", self.passed, self.failed)
    self:quit(self.failed == 0 and Core.ExitCode.Success or Core.ExitCode.Failure)
end

return ScreenProjectionTest
