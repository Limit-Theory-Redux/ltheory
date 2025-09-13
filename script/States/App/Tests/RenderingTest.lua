local Log = require("Core.Util.Log")
local Inspect = require("Core.Util.Inspect")
local Registry = require("Core.ECS.Registry")
local RenderingTest = require('States.Application')
local Materials = require("Shared.Registries.Materials")
local CameraSystem = require("Modules.Rendering.Systems").Camera
local CameraEntity = require("Modules.Rendering.Entities").Camera
local BoxEntity = require("Modules.Core.Entities").Box
local Physics = require("Modules.Physics.Components")
local Rendering = require("Modules.Rendering.Components")

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onInit()
    -- Mark as initialized
    self.initialized = true

    -- Set App Settings --
    -- self.profilerFont = Font.Load('NovaMono', 20)
    -- self.profiling = false

    self.renderer = RenderPipeline()

    -- Initialize Materials --
    require("Shared.Definitions.MaterialDefs")
    require("Shared.Definitions.UniformFuncDefs")

    -- Set GameState --
    GameState:SetState(Enums.GameStates.InGame)

    -- Spawn CameraEntity
    local camera = CameraEntity()

    CameraSystem:setCamera(camera)
    CameraSystem.currentCameraTransform:setPosition(Position(0, 0, 0))
    CameraSystem.currentCameraTransform:setRotation(Quat.Identity())

    -- Create First RNG for Scene
    -- local rng = RNG.Create(0)

    -- Generate Box Mesh
    self.boxMesh = Mesh.Box(7)
    -- Get Box Entity and Components
    local boxMaterial = Materials.DebugColor() ---@type Material
    boxMaterial:addStaticShaderVar("color", Enums.UniformType.Float3, function() return 1.0, 0.0, 1.0 end)
    self.boxEntity = BoxEntity(boxMaterial)
    self.boxRend = self.boxEntity:get(Rendering.Render)
    -- Log.Warn(Inspect(self.boxRend:getMaterial(BlendMode.Disabled)))
    self.boxRB = self.boxEntity:get(Physics.RigidBody)
    -- Set RigidBody
    self.boxRB:setRigidBody(RigidBody.CreateBoxFromMesh(self.boxMesh))
    self.boxRB:getRigidBody():setPos(Position(0, 0, -5))

    self.rotationQuaternion = Quat(0, 0, 0, 1) -- Identity quaternion
end

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onPreRender(data)
    -- Initialize Profiler
    -- Profiler.Enable()

    -- Start onPreRender Profiler
    -- Profiler.SetValue('gcmem', GC.GetMemory())
    -- Profiler.Begin('App.onPreRender')

    -- Set Timescale
    self.timeScale = 1.0

    -- Set Timescale on EventBus
    if self.timeScale ~= EventBus:getTimeScale() then
        EventBus:setTimeScale(self.timeScale)
    end

    -- Get Delta Time
    local timeScaledDt = data:deltaTime()

    -- Define the rotation axis
    local rotationAxis = Vec3f(1, 1, 1)

    -- Manually compute the rotation quaternion for the incremental rotation
    --! Since Quat.FromAxisAngle && Quat.SetRotLocal do not work as intended
    -- TODO: Fix
    -- Probably a small refactor where passing a precreated Quat is not necessary would be nice to
    -- e.g. local quat = Quat.FromAxisAngle(x, x)
    local angle = math.rad(10) * timeScaledDt
    local halfAngle = angle / 2
    local sinHalfAngle = math.sin(halfAngle)
    local cosHalfAngle = math.cos(halfAngle)
    local rotateByQuaternion = Quat(
        rotationAxis.x * sinHalfAngle,
        rotationAxis.y * sinHalfAngle,
        rotationAxis.z * sinHalfAngle,
        cosHalfAngle
    )

    -- Update the accumulated rotation quaternion
    self.rotationQuaternion = self.rotationQuaternion * rotateByQuaternion

    -- Apply the combined rotation to the rigid body
    -- Shouldnt setRotLocal do this?
    self.boxRB:getRigidBody():setRot(self.rotationQuaternion)

    --[[
        < Previously where Player and UI Canvas Updates were Called
    ]] --

    do -- Handle App Resizing
        Profiler.SetValue('gcmem', GC.GetMemory())
        Profiler.Begin('App.onResize')
        local size = Window:size()
        Window:cursor():setGrabMode(CursorGrabMode.None)
        if size.x ~= self.resX or size.y ~= self.resY then
            self.resX = size.x
            self.resY = size.y
            GameState.render.resX = self.resX
            GameState.render.resY = self.resY
            self:onResize(self.resX, self.resY)
        end
        Profiler.End()
    end

    -- Stop Pre Render Profiler
    -- Profiler.End()
end

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onRender(data)
    -- Start onRender Profiler
    -- Profiler.SetValue('gcmem', GC.GetMemory())
    -- Profiler.Begin('App.onRender')

    -- Start Window Draw()
    Window:beginDraw()

    -- Originally in Canvas:draw
    --RenderState.PushBlendMode(BlendMode.Alpha)
    --Draw.PushAlpha(2)
    --DrawEx.PushAlpha(2)

    Draw.Clear(0, 0.1, 0.2, 1)
    Draw.ClearDepth(1)

    -- < TEST RENDER > --
    ClipRect.PushDisabled()
    RenderState.PushAllDefaults()

    CameraSystem:updateViewMatrix()
    CameraSystem:updateProjectionMatrix(self.resX, self.resY)

    local camEye = CameraSystem:getCurrentCameraEye()
    CameraSystem:beginDraw(CameraSystem.currentCameraData, CameraSystem.currentCameraTransform)

    -- self.renderer:start(self.resX, self.resY)

    local boxMat = self.boxRend:getMaterial(BlendMode.Disabled)
    boxMat.shaderState:start()
    boxMat:setAllShaderVars(camEye, self.boxEntity)
    self.boxMesh:draw()
    boxMat.shaderState:stop()

    -- self.renderer:stop()

    CameraSystem:endDraw()

    -- From GameView - Composited UI Pass
    --[[
    local ss = 1
    Viewport.Push(0, 0, ss * self.resX, ss * self.resY, true)
    ClipRect.PushTransform(0, 0, ss, ss)
    ShaderVar.PushMatrix("mWorldViewUI", Matrix.Scaling(ss, ss, 1.0))

    ShaderVar.Pop("mWorldViewUI")
    ClipRect.PopTransform()
    Viewport.Pop()
    --]]
    -- self.renderer:presentAll(0, 0, self.resX, self.resY, false)

    RenderState.PopAll()
    ClipRect.Pop()

    -- Originally in Canvas:draw
    -- DrawEx.PopAlpha()
    -- Draw.PopAlpha()
    -- RenderState.PopBlendMode()

    -- Stop onRender Profiler
    -- Profiler.End()
    -- Profiler.LoopMarker()
end

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onPostRender(data)
    do -- End Draw
        -- Profiler.SetValue('gcmem', GC.GetMemory())
        -- Profiler.Begin('App.onPostRender')
        Window:endDraw()
        -- Profiler.End()
    end
end

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onInput(data) end

return RenderingTest
