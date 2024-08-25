-- Entities
local Camera = require("_ECS_WIP_TEMP.Entities.Rendering.Camera")                  --!temp path
local Asteroid = require("_ECS_WIP_TEMP.Entities.CelestialObjects.AsteroidEntity") --!temp path
-- Systems
local GlobalStorage = require("_ECS_WIP_TEMP.Systems.Storage.GlobalStorage")               --!temp path
---@type CameraSystem
local CameraSystem = require("_ECS_WIP_TEMP.Systems.Rendering.CameraSystem")
-- Utilities
local Material = require("_ECS_WIP_TEMP.Shared.Rendering.Material")           --!temp path
local AutoShaderVar = require("_ECS_WIP_TEMP.Shared.Rendering.AutoShaderVar") --!temp path
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

    -- Set GameState --
    GameState:SetState(Enums.GameStates.InGame)

    -- Spawn CameraEntity
    local camera = Camera()
    local entityInfo = GlobalStorage:storeEntity(camera)

    -- Testing Materials
    local Materials = requireAll("_ECS_WIP_TEMP.Shared.Materials")
    local AsteroidMaterial = Materials.AsteroidMaterial()
    Log.Warn(Inspect(AsteroidMaterial))

    CameraSystem:setCamera(entityInfo)

    local rng = RNG.Create(0):managed()

    -- Spawn a Asteroid
    local a = Asteroid(rng:get64())
    GlobalStorage:storeEntity(a)
    local rb = a:findComponentByName("PhysicsRigidBody")
    --rb:setRigidBody(RigidBody())
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

    -- Calling uiCanvas Draw
    --TODO: Should subscribe to onRender instead
    if GameState.render.uiCanvas ~= nil then
        GameState.render.uiCanvas:draw(self.resX, self.resY)
        Gui:draw()
    end

    do -- Metrics display
        if true then
            local dt = data:deltaTime()

            local s = string.format(
                '%.2f ms / %.0f fps / %.2f MB / %.1f K tris / %d draws / %d imms / %d swaps',
                1000.0 * dt,
                1.0 / dt,
                GC.GetMemory() / 1000.0,
                Metric.Get(Metric.TrisDrawn) / 1000,
                Metric.Get(Metric.DrawCalls),
                Metric.Get(Metric.Immediate),
                Metric.Get(Metric.FBOSwap))
            RenderState.PushBlendMode(BlendMode.Alpha)
            UI.DrawEx.SimpleRect(0, self.resY - 30, self.resX, self.resY, Color(0.1, 0.1, 0.1, 0.5))
            self.profilerFont:draw(s, 10, self.resY - 5, Color(1, 1, 1, 1))

            local y = self.resY - 5
            if self.profiling then
                self.profilerFont:draw('>> PROFILER ACTIVE <<', self.resX - 128, y, Color(1, 0, 0.15, 1))
                y = y - 12
            end
            RenderState.PopBlendMode()
        end
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
