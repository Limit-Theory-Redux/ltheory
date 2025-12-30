local Application         = require('States.Application')

---@class DeferredTest: Application
---@field lights table
---@field spheres table
local DeferredTest        = Subclass("DeferredTest", Application)

local Registry            = require("Core.ECS.Registry")
local Entity              = require("Core.ECS.Entity")
local Materials           = require("Shared.Registries.Materials")
local CameraManager       = require("Modules.Cameras.Managers.CameraManager")
local CinematicCamera     = require("Modules.Cameras.Managers.CameraControllers.CinematicCameraController")
local CameraEntity        = require("Modules.Cameras.Entities").Camera
local BoxEntity           = require("Modules.Core.Entities").Box
local PointLightEntity    = require("Modules.Rendering.Entities").PointLight
local Physics             = require("Modules.Physics.Components")
local Rendering           = require("Modules.Rendering.Components")
local RenderCoreSystem    = require("Modules.Rendering.Systems.RenderCoreSystem")
local DeltaTimer          = require("Shared.Tools.DeltaTimer")
local CameraDataComponent = require("Modules.Cameras.Components").CameraData
local RenderOverlay       = require("Shared.Tools.RenderOverlay")
local ShaderHotReload     = require("Render.ShaderHotReload")

function DeferredTest:onInit()
    require("Shared.Definitions.MaterialDefs")
    require("Shared.Definitions.UniformFuncDefs")

    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    -- Enable shader hot reload
    if ShaderHotReload:init() then
        Log.Info("Shader hot reload enabled")
    end

    -- Enable render overlay
    RenderOverlay:setVisible(true)

    -- Timers
    self.timer = DeltaTimer("DeferredTest")
    self.timer:start("camLoop", 0.01, true)

    -- Camera setup
    local cam = CameraEntity()
    CameraManager:registerCamera("CineCam", cam)
    self.cameraController = CinematicCamera(cam, { useFocusPoint = true })
    cam:get(CameraDataComponent):setController(self.cameraController)
    CameraManager:setActiveCamera("CineCam")

    -- Scene center
    self.sceneCenter = Vec3f(0, 0, 0)

    -- Create floor
    self:createFloor()

    -- Create some boxes in the scene
    self.boxes = {}
    self:createBox(Vec3f(-6, 2, 0), 4)
    self:createBox(Vec3f(0, 2, 0), 4)
    self:createBox(Vec3f(6, 2, 0), 4)
    self:createBox(Vec3f(-3, 5, -3), 3)
    self:createBox(Vec3f(3, 5, -3), 3)

    -- Create point lights with different colors
    self.lights = {}
    self:createLight(Vec3f(-8, 6, 5), Vec3f(1, 0.3, 0.3), 3.0)    -- Red
    self:createLight(Vec3f(8, 6, 5), Vec3f(0.3, 0.3, 1), 3.0)     -- Blue
    self:createLight(Vec3f(0, 10, -8), Vec3f(0.3, 1, 0.3), 3.0)   -- Green
    self:createLight(Vec3f(0, 4, 8), Vec3f(1, 1, 0.8), 2.0)       -- Warm white

    -- Camera path parameters
    self.camRadius        = 25
    self.camHeight        = 10
    self.camSpeed         = 0.2
    self.camZoomAmplitude = 5
    self.camZoomSpeed     = 0.3

    EventBus:subscribe(Event.PreRender, self, self.onUpdate)
end

function DeferredTest:createFloor()
    local mesh = Mesh.Box(1)
    local mat = Materials.DebugDeferred()

    local floor = BoxEntity({ { mesh = mesh, material = mat } })
    local rb = floor:get(Physics.RigidBody)
    rb:setRigidBody(RigidBody.CreateBox(50, 0.5, 50))
    rb:getRigidBody():setPos(Position(0, -1, 0))
    rb:getRigidBody():setKinematic(true)
end

function DeferredTest:createBox(pos, size)
    local mesh = Mesh.Box(5)
    local mat = Materials.DebugDeferred()

    local box = BoxEntity({ { mesh = mesh, material = mat } })
    local rb = box:get(Physics.RigidBody)
    rb:setRigidBody(RigidBody.CreateBox(size, size, size))
    rb:getRigidBody():setPos(Position(pos.x, pos.y, pos.z))
    rb:getRigidBody():setKinematic(true)

    table.insert(self.boxes, {
        entity = box,
        basePos = pos,
        rotation = Quat.Identity(),
        rotSpeed = 20 + math.random() * 30
    })

    return box
end

function DeferredTest:createLight(pos, color, intensity)
    local light = PointLightEntity(color, intensity, 50)
    local transform = light:get(Physics.Transform)
    transform:setPos(Position(pos.x, pos.y, pos.z))

    -- Also create a small visible box at the light position as indicator
    local lightMesh = Mesh.Box(1)
    local mat = Materials.DebugColor()

    local indicator = BoxEntity({ { mesh = lightMesh, material = mat } })
    local rb = indicator:get(Physics.RigidBody)
    rb:setRigidBody(RigidBody.CreateBox(0.5, 0.5, 0.5))
    rb:getRigidBody():setPos(Position(pos.x, pos.y, pos.z))
    rb:getRigidBody():setKinematic(true)

    table.insert(self.lights, {
        entity = light,
        indicator = indicator,
        basePos = pos,
        phase = math.random() * math.pi * 2,
        amplitude = 2 + math.random() * 2,
        speed = 0.5 + math.random() * 0.5
    })

    return light
end

function DeferredTest:onUpdate(data)
    local dt = data:deltaTime()
    self.timer:update(dt)

    -- Poll for shader hot reload changes
    ShaderHotReload:update()

    local t = self.timer:getTotal("camLoop")

    -- Animate lights (bob up and down)
    for _, lightData in ipairs(self.lights) do
        local y = lightData.basePos.y + math.sin(t * lightData.speed + lightData.phase) * lightData.amplitude
        local newPos = Position(lightData.basePos.x, y, lightData.basePos.z)

        -- Update both light and indicator
        lightData.entity:get(Physics.Transform):setPos(newPos)
        if lightData.indicator and lightData.indicator:isValid() then
            lightData.indicator:get(Physics.RigidBody):getRigidBody():setPos(newPos)
        end
    end

    -- Animate boxes rotation
    for _, boxData in ipairs(self.boxes) do
        if boxData.entity:isValid() then
            local rb = boxData.entity:get(Physics.RigidBody):getRigidBody()
            local axis = Vec3f(0, 1, 0)
            local angle = math.rad(boxData.rotSpeed) * dt
            boxData.rotation = boxData.rotation * Quat.FromAxisAngle(axis, angle)
            rb:setRot(boxData.rotation)
        end
    end

    -- Camera circular path
    local angle = t * self.camSpeed
    local zoomOffset = math.sin(t * self.camZoomSpeed) * self.camZoomAmplitude
    local camPos = self.sceneCenter + Vec3f(
        (self.camRadius + zoomOffset) * math.cos(angle),
        self.camHeight + zoomOffset * 0.3,
        (self.camRadius + zoomOffset) * math.sin(angle)
    )

    self.cameraController:setPositionAndFocus(camPos, self.sceneCenter)
end

function DeferredTest:onInput(data)
    RenderCoreSystem:handleInput()

    -- Toggle render thread with 'R' key
    if Input:keyboard():isPressed(Button.KeyboardR) then
        if Engine:isRenderThreadActive() then
            Log.Info("Stopping render thread...")
            Engine:stopRenderThread()
        else
            Log.Info("Starting render thread...")
            if Engine:startRenderThread() then
                Log.Info("Render thread started successfully")
            else
                Log.Error("Failed to start render thread")
            end
        end
    end

    -- Add/remove lights with +/-
    if Input:keyboard():isPressed(Button.KeyboardEqual) then
        local pos = Vec3f(
            math.random() * 16 - 8,
            4 + math.random() * 8,
            math.random() * 16 - 8
        )
        local color = Vec3f(math.random(), math.random(), math.random())
        self:createLight(pos, color, 2.0 + math.random())
        Log.Info("Added light, total: %d", #self.lights)
    end

    if Input:keyboard():isPressed(Button.KeyboardMinus) then
        if #self.lights > 0 then
            local lightData = table.remove(self.lights)
            lightData.entity:delete()
            if lightData.indicator and lightData.indicator:isValid() then
                lightData.indicator:delete()
            end
            Log.Info("Removed light, total: %d", #self.lights)
        end
    end
end

function DeferredTest:onRender(data)
    RenderCoreSystem:render(data)
    Draw.Flush()
end

return DeferredTest
