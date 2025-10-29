local Application      = require('States.Application')

---@class RenderingTest: Application
local RenderingTest    = Subclass("RenderingTest", Application)

local Materials        = require("Shared.Registries.Materials")
local CameraSystem     = require("Modules.Rendering.Systems.CameraSystem")
local CameraEntity     = require("Modules.Rendering.Entities").Camera
local BoxEntity        = require("Modules.Core.Entities").Box
local Physics          = require("Modules.Physics.Components")
local RenderComp       = require("Modules.Rendering.Components").Render
local RenderCoreSystem = require("Modules.Rendering.Systems.RenderCoreSystem")

function RenderingTest:onInit()
    self.initialized = true
    require("Shared.Definitions.MaterialDefs")
    require("Shared.Definitions.UniformFuncDefs")
    GameState:SetState(Enums.GameStates.InGame)

    -- Camera
    local cam = CameraEntity()
    CameraSystem:setCamera(cam)
    CameraSystem.currentCameraTransform:setPosition(Position(0, 0, 0))
    CameraSystem.currentCameraTransform:setRotation(Quat.Identity())

    -- Box
    local mesh = Mesh.Box(7)
    local mat  = Materials.DebugColor()
    mat:addStaticShaderVar("color", Enums.UniformType.Float3,
        function() return 1.0, 0.0, 1.0 end)

    self.boxEntity = BoxEntity({ { mesh = mesh, material = mat } })
    local rb = self.boxEntity:get(Physics.RigidBody)
    rb:setRigidBody(RigidBody.CreateBoxFromMesh(mesh))
    rb:getRigidBody():setPos(Position(0, 0, -5))

    self.rotation = Quat(0, 0, 0, 1)

    -- Subscriptions
    EventBus:subscribe(Event.PreRender, self, self.rotateBox)
end

function RenderingTest:rotateBox(data)
    local dt = data:deltaTime()
    local axis = Vec3f(1, 1, 1)
    local angle = math.rad(10) * dt
    local h = angle / 2
    local inc = Quat(
        axis.x * math.sin(h),
        axis.y * math.sin(h),
        axis.z * math.sin(h),
        math.cos(h)
    )
    self.rotation = self.rotation * inc
    self.boxEntity:get(Physics.RigidBody):getRigidBody():setRot(self.rotation)
end

function RenderingTest:onRender(dt)
    -- Call RenderCoreSystem
    RenderCoreSystem:render(dt)

    self:immediateUI(function()
        local mem = GC.GetMemory()
        UI.DrawEx.TextAdditive(
            'Unageo-Medium',
            string.format("Lua Memory: %.2f KB", mem),
            20,
            RenderCoreSystem.resX / 2 - 20, 50, 40, 20,
            0.75, 0.75, 0.75, 0.75,
            0.5, 0.5
        )
    end)
end

return RenderingTest
