-- Entities
local Camera = require("_ECS_WIP_TEMP.Entities.Rendering.Camera")                   --!temp path
local BoxEntity = require("_ECS_WIP_TEMP.Entities.Debug.BoxEntity")                 --!temp path
-- Storage
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.Storage.GlobalStorage")        --!temp path
-- Systems
---@type CameraSystem
local CameraSystem = require("_ECS_WIP_TEMP.Systems.Rendering.CameraSystem")        --!temp path
-- Generators

-- Rendering
local renderState = require("_ECS_WIP_TEMP.Shared.Rendering.RenderState")
-- Utilities
local Log = require("Core.Util.Log")
local Inspect = require("Core.Util.Inspect")

local RenderingTest = require('States.Application')

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onInit()
    -- Mark as initialized
    self.initialized = true

    -- Set App Settings --
    self.profilerFont = Font.Load('NovaMono', 20)
    self.profiling = true
    
    self.renderer = RenderPipeline()

    -- Initialize Materials --
    require("_ECS_WIP_TEMP.Shared.Definitions.MaterialDefs")
    require("_ECS_WIP_TEMP.Shared.Definitions.UniformFuncDefs")

    -- Set GameState --
    GameState:SetState(Enums.GameStates.InGame)

    -- Spawn CameraEntity
    local camera = Camera()
    local entityInfo = GlobalStorage:storeEntity(camera)

    CameraSystem:setCamera(entityInfo)
    CameraSystem.currentCameraTransform:setPosition(Position(0,0,0))
    CameraSystem.currentCameraTransform:setRotation(Quat.Identity())

    -- Create First RNG for Scene
    -- local rng = RNG.Create(0):managed()
    
    -- Generate Box Mesh
    self.boxMesh = Mesh.Box(7)
    -- Get Box Entity and Components
    self.boxEntity = BoxEntity()
    self.boxRend = self.boxEntity:findComponentByArchetype(Enums.ComponentArchetype.RenderComponent)
    -- Log.Warn(Inspect(self.boxRend:getMaterial(BlendMode.Disabled)))
    self.boxRB = self.boxEntity:findComponentByArchetype(Enums.ComponentArchetype.RigidBodyComponent)
    -- Set RigidBody
    self.boxRB:setRigidBody(RigidBody.CreateBoxFromMesh(self.boxMesh))
    self.boxRB:getRigidBody():setPos(Position(0, 0, -5))
    self.position = 0.0
end

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onPreRender(data)
    -- Initialize Profiler
    Profiler.Enable()

    -- Start onPreRender Profiler
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onPreRender')

    -- Set Timescale
    self.timeScale = 1.0

    -- Set Timescale on EventBus
    if self.timeScale ~= EventBus:getTimeScale() then
        EventBus:setTimeScale(self.timeScale)
    end
    -- Get Delta Time
    local timeScaledDt = data:deltaTime()
    self.position = self.position + 1*timeScaledDt
    self.boxRB:getRigidBody():setPos(Position(self.position, self.position, -5))

    --[[
        < Previously where Player and UI Canvas Updates were Called
    ]]--

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
    Profiler.End()
end

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onRender(data)
    -- Start onRender Profiler
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onRender')

    -- Start Window Draw()
    Window:beginDraw()

    -- Originally in Canvas:draw
    --RenderState.PushBlendMode(BlendMode.Alpha)
    --Draw.PushAlpha(2)
    --DrawEx.PushAlpha(2)

    Draw.Clear(0, 0, 0, 1)
    Draw.ClearDepth(1)

    -- < TEST RENDER > --
    ClipRect.PushDisabled()
    RenderState.PushAllDefaults()

    CameraSystem:updateViewMatrix()
    CameraSystem:updateProjectionMatrix(self.resX,self.resY)

    renderState:setCameraEye(CameraSystem.currentCameraTransform:getPosition())
    CameraSystem:beginDraw(CameraSystem.currentCameraData, CameraSystem.currentCameraTransform)

    -- self.renderer:start(self.resX, self.resY)

    local boxMat = self.boxRend:getMaterial(BlendMode.Disabled)
    boxMat.shaderState:start()
    boxMat:setAllShaderVars(renderState, self.boxEntity)
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
    Profiler.End()
    Profiler.LoopMarker()
end

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onPostRender(data)
    do -- End Draw
        Profiler.SetValue('gcmem', GC.GetMemory())
        Profiler.Begin('App.onPostRender')
        Window:endDraw()
        Profiler.End()
    end
end

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onInput(data) end

return RenderingTest
