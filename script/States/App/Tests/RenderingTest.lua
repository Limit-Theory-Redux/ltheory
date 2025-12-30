local Application         = require('States.Application')

---@class RenderingTest: Application
local RenderingTest       = Subclass("RenderingTest", Application)

local Registry            = require("Core.ECS.Registry")
local Materials           = require("Shared.Registries.Materials")
local CameraManager       = require("Modules.Cameras.Managers.CameraManager")
local CinematicCamera     = require("Modules.Cameras.Managers.CameraControllers.CinematicCameraController")
local CameraEntity        = require("Modules.Cameras.Entities").Camera
local BoxEntity           = require("Modules.Core.Entities").Box
local Physics             = require("Modules.Physics.Components")
local RenderCoreSystem    = require("Modules.Rendering.Systems.RenderCoreSystem")
local CameraSystem        = require("Modules.Cameras.Systems.CameraSystem") -- Self-registers to EventBus
local DeltaTimer          = require("Shared.Tools.DeltaTimer")
local Entity              = require("Core.ECS.Entity")
local CameraDataComponent = require("Modules.Cameras.Components").CameraData
local RenderOverlay       = require("Shared.Tools.RenderOverlay")
local ShaderHotReload     = require("Render.ShaderHotReload")

function RenderingTest:onInit()
    require("Shared.Definitions.MaterialDefs")
    require("Shared.Definitions.UniformFuncDefs")

    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    -- Render thread toggle (experimental)
    self.useRenderThread = false

    -- Enable shader hot reload (edit shaders in res/shader/ to see live updates)
    if ShaderHotReload:init() then
        Log.Info("Shader hot reload enabled - edit shaders to see live changes")
    end

    -- Enable render overlay by default for this test (Shift+O to toggle)
    RenderOverlay:setVisible(true)

    -- Timers
    self.timer = DeltaTimer("RenderingTest")
    self.timer:start("camLoop", 0.01, true)

    -- Camera setup
    local cam = CameraEntity()
    CameraManager:registerCamera("CineCam", cam)
    self.cameraController = CinematicCamera(cam, { useFocusPoint = true })
    cam:get(CameraDataComponent):setController(self.cameraController)
    CameraManager:setActiveCamera("CineCam")

    -- Grid setup
    self.boxes = {}
    local nx, ny, nz = 6, 6, 6
    local spacing = 7
    local boxRes = 7
    local zOffset = -10
    local idCounter = 1

    self.gridCenter = Vec3f(
        (nx - 1) * spacing / 2,
        (ny - 1) * spacing / 2,
        (nz - 1) * spacing / 2 + zOffset
    )

    self.gridParentEntity = Entity.Create("Anchor")
    local parentRB = self.gridParentEntity:add(Physics.RigidBody())
    local parentBody = RigidBody.CreateSphere()
    parentBody:setPos(Position(self.gridCenter.x, self.gridCenter.y, self.gridCenter.z))
    parentBody:setKinematic(true)
    parentBody:setCollidable(false)
    parentRB:setRigidBody(parentBody)

    self.gridRotation = Quat.Identity()

    -- Create boxes
    for x = 1, nx do
        for y = 1, ny do
            for z = 1, nz do
                local relativePos = Position(
                    (x - 1) * spacing - (nx - 1) * spacing / 2,
                    (y - 1) * spacing - (ny - 1) * spacing / 2,
                    (z - 1) * spacing - (nz - 1) * spacing / 2
                )
                self:createBox(relativePos, idCounter, boxRes)
                idCounter = idCounter + 1
            end
        end
    end

    -- Camera circular path parameters
    self.camRadius        = 40
    self.camHeight        = 20
    self.camSpeed         = 0.2
    self.camZoomAmplitude = 5
    self.camZoomSpeed     = 0.5

    EventBus:subscribe(Event.PreRender, self, self.updateBoxes)
end

function RenderingTest:createBox(relativePos, id, res)
    local mesh      = Mesh.Box(res)
    local mat       = Materials.DebugColor()

    local boxEntity = BoxEntity({ { mesh = mesh, material = mat } })
    local rbCmp     = boxEntity:get(Physics.RigidBody)
    rbCmp:setRigidBody(RigidBody.CreateBoxFromMesh(mesh))
    rbCmp:getRigidBody():setPos(Position(
        self.gridCenter.x + relativePos.x,
        self.gridCenter.y + relativePos.y,
        self.gridCenter.z + relativePos.z
    ))
    rbCmp:getRigidBody():setRotLocal(Quat.Identity())

    Registry:attachEntity(self.gridParentEntity, boxEntity)

    -- Start delete timer
    self.timer:start("delete_" .. id, 5 + math.random() * 10, false)

    table.insert(self.boxes, {
        id = id,
        entity = boxEntity,
        relativePos = relativePos,
        rotation = Quat.Identity(),
        rotationSpeed = 20 + math.random() * 40,
        rotationDir = 1,
        deleted = false,
        deletedPos = nil
    })

    return boxEntity
end

function RenderingTest:updateBoxes(data)
    local dt = data:deltaTime()
    self.timer:update(dt)

    -- Poll for shader hot reload changes
    ShaderHotReload:update()

    -- Rotate grid
    local gridRotationSpeed = 10
    local gridAxis = Vec3f(0, 1, 0)
    local gridAngle = math.rad(gridRotationSpeed) * dt
    self.gridRotation = self.gridRotation * Quat.FromAxisAngle(gridAxis, gridAngle)

    -- Update boxes
    for _, boxData in ipairs(self.boxes) do
        local entity = boxData.entity
        if entity:isValid() and not boxData.deleted then
            local rb = entity:get(Physics.RigidBody):getRigidBody()

            -- Self-rotation
            local axis = Vec3f(1, 1, 1):normalize()
            local angle = math.rad(boxData.rotationSpeed * boxData.rotationDir) * dt
            boxData.rotation = boxData.rotation * Quat.FromAxisAngle(axis, angle)
            rb:setRot(self.gridRotation * boxData.rotation)

            local relVec = Vec3f(boxData.relativePos.x, boxData.relativePos.y, boxData.relativePos.z)
            local rotatedVec = self.gridRotation + relVec
            rb:setPos(Position(
                self.gridCenter.x + rotatedVec.x,
                self.gridCenter.y + rotatedVec.y,
                self.gridCenter.z + rotatedVec.z
            ))
        end
    end

    -- Camera circular path around grid
    local t = self.timer:getTotal("camLoop")
    local angle = t * self.camSpeed
    local zoomOffset = math.sin(t * self.camZoomSpeed) * self.camZoomAmplitude
    local camPos = self.gridCenter + Vec3f(
        (self.camRadius + zoomOffset) * math.cos(angle),
        self.camHeight + zoomOffset * 0.5,
        (self.camRadius + zoomOffset) * math.sin(angle)
    )

    -- Set target position and focus for CinematicCamera
    self.cameraController:setPositionAndFocus(camPos, self.gridCenter)
end

function RenderingTest:onInput(data)
    -- Toggle render overlay (Shift+O) - handled by RenderCoreSystem
    RenderCoreSystem:handleInput()

    -- Toggle render thread with 'R' key (experimental)
    if Input:keyboard():isPressed(Button.KeyboardR) then
        if Engine:isRenderThreadActive() then
            Log.Info("Stopping render thread...")
            Engine:stopRenderThread()
            self.useRenderThread = false
        else
            Log.Info("Starting render thread...")
            if Engine:startRenderThread() then
                self.useRenderThread = true
                Log.Info("Render thread started successfully")
            else
                Log.Error("Failed to start render thread")
            end
        end
    end
end

function RenderingTest:onRender(data)
    RenderCoreSystem:render(data)
    Draw.Flush()
end

return RenderingTest
