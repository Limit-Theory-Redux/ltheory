-- Entities
local Camera = require("_ECS_WIP_TEMP.Entities.Rendering.Camera")                   --!temp path
local Asteroid = require("_ECS_WIP_TEMP.Entities.CelestialObjects.AsteroidEntity")  --!temp path
local BoxEntity = require("_ECS_WIP_TEMP.Entities.Debug.BoxEntity")  --!temp path
-- Storage
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.Storage.GlobalStorage")        --!temp path
local MeshStorage = require("_ECS_WIP_TEMP.Systems.Storage.MeshStorage")            --!temp path
-- Systems
---@type CameraSystem
local CameraSystem = require("_ECS_WIP_TEMP.Systems.Rendering.CameraSystem")        --!temp path
-- Generators
local AsteroidMesh = require("_ECS_WIP_TEMP.Systems.Generators.Mesh.CelestialObjects.AsteroidMesh")
local Boxes = require("_ECS_WIP_TEMP.Systems.Generators.Mesh.ShapeLib.Boxes")

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

    -- Initialize Materials --
    require("_ECS_WIP_TEMP.Shared.Definitions.MaterialDefs")
    require("_ECS_WIP_TEMP.Shared.Definitions.UniformFuncDefs")

    -- Set GameState --
    GameState:SetState(Enums.GameStates.InGame)

    -- Spawn CameraEntity
    local camera = Camera()
    local entityInfo = GlobalStorage:storeEntity(camera)

    CameraSystem:setCamera(entityInfo)

    -- Create First RNG for Scene
    self.rng = RNG.Create(0):managed()
    
    -- Generate RandomBox
    self.boxes = Boxes.RandomBox(self.rng)
    self.boxMesh = Boxes.BoxesToMesh(self.boxes, 1, 1, false)
    -- Get Box Entity and Components
    self.boxEntity = BoxEntity()
    self.boxRend = self.boxEntity:findComponentByArchetype(Enums.ComponentArchetype.RenderComponent)
    self.boxRB = self.boxEntity:findComponentByArchetype(Enums.ComponentArchetype.RigidBodyComponent)
    -- Set RigidBody
    self.boxRB:setRigidBody(RigidBody.CreateBoxFromMesh(self.boxMesh))
    self.boxRB:getRigidBody():setPos(Position(0, 0, 0))
end

---@diagnostic disable-next-line: duplicate-set-field
function RenderingTest:onPreRender(data)
    -- Initialize Profiler
    Profiler.Enable()
    --[[
        < Toggle Profiling >
    ]]--
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

    --[[
        < Previously where Player and UI Canvas Updates were Called
    ]]--

    -- Handle App Resizing
    do
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

    -- < TEST RENDER > --

    do -- Push Defaults
        ClipRect.PushDisabled()
        RenderState.PushAllDefaults()
    end

    do -- Set Camera
        CameraSystem:lookAt(Vec3f(0, 0, 0))
        CameraSystem:setProjection(self.resX,self.resY)
        renderState:setCameraEye(CameraSystem.currentCameraTransform:getPosition())
        CameraSystem:beginCameraDraw(CameraSystem.currentCameraData, CameraSystem.currentCameraTransform)
    end

    -- < What To Render Goes Here > --

    do -- Cleanup
        CameraSystem:endDraw()
        RenderState.PopAll()
        ClipRect.Pop()
    end
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
