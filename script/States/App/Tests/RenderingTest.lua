local Application      = require('States.Application')

---@class RenderingTest: Application
local RenderingTest    = Subclass("RenderingTest", Application)

local Registry         = require("Core.ECS.Registry")
local Materials        = require("Shared.Registries.Materials")
local CameraSystem     = require("Modules.Rendering.Systems.CameraSystem")
local CameraEntity     = require("Modules.Rendering.Entities").Camera
local BoxEntity        = require("Modules.Core.Entities").Box
local Physics          = require("Modules.Physics.Components")
local RenderComp       = require("Modules.Rendering.Components").Render
local RenderCoreSystem = require("Modules.Rendering.Systems.RenderCoreSystem")
local DeltaTimer       = require("Shared.Tools.DeltaTimer")
local Entity           = require("Core.ECS.Entity")

function RenderingTest:onInit()
    require("Shared.Definitions.MaterialDefs")
    require("Shared.Definitions.UniformFuncDefs")

    Window:setPresentMode(PresentMode.NoVsync)
    Window:setFullscreen(false, true)

    -- Timers
    self.timer = DeltaTimer("RenderingTest")
    self.timer:start("fps", 0.1)
    self.timer:start("camLoop", 0.01, true) -- continuous loop

    -- FPS tracking
    self.frameCount = 0
    self.smoothFPS = 0
    self.fpsText = "FPS: 0"
    self.time = 0

    -- Camera setup
    local cam = CameraEntity()
    CameraSystem:setCamera(cam)

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
    self.camRadius        = 40  -- zoomed out a bit more
    self.camHeight        = 20
    self.camSpeed         = 0.2 -- radians/sec
    self.camZoomAmplitude = 5   -- zoom in/out
    self.camZoomSpeed     = 0.5 -- frequency of zoom oscillation

    -- Initialize camera transform
    local angle           = 0
    local camPos          = self.gridCenter + Vec3f(
        self.camRadius * math.cos(angle),
        self.camHeight,
        self.camRadius * math.sin(angle)
    )
    CameraSystem.currentCameraTransform:setPosition(Position(camPos.x, camPos.y, camPos.z))
    CameraSystem.currentCameraTransform:setRotation(Quat.LookAt(camPos, self.gridCenter, Vec3f(0, 1, 0)))

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
    self.time = self.time + dt
    self.timer:update(dt)

    -- FPS
    self.frameCount = self.frameCount + 1
    if self.timer:check("fps") then
        local instantFPS = self.frameCount * 10
        self.smoothFPS = self.smoothFPS * 0.3 + instantFPS * 0.7
        self.fpsText = "FPS: " .. math.floor(self.smoothFPS + 0.5)
        self.frameCount = 0
    end

    -- Rotate grid
    local gridRotationSpeed = 10
    local gridAxis = Vec3f(0, 1, 0)
    local gridAngle = math.rad(gridRotationSpeed) * dt
    self.gridRotation = self.gridRotation * Quat.FromAxisAngle(gridAxis, gridAngle)

    -- Update boxes
    for _, boxData in ipairs(self.boxes) do
        local id = boxData.id
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

            -- Start blinking 1s before deletion
            local deleteTimeLeft = self.timer:timeLeft("delete_" .. id)
            if deleteTimeLeft and deleteTimeLeft < 1.0 and not self.timer:isActive("colorChange_" .. id) then
                self.timer:start("colorChange_" .. id, 0.1, true)
            end

            -- Apply blinking color
            if self.timer:check("colorChange_" .. id) then
                local rend = entity:get(RenderComp)
                local r, g, b = math.random(), math.random(), math.random()
                for meshmat in Iterator(rend:getMeshes()) do
                    meshmat.material:addStaticShaderVar("color", Enums.UniformType.Float3,
                        function() return r, g, b end)
                end
            end

            -- Deletion
            if self.timer:check("delete_" .. id) then
                local pos = Position()
                boxData.deletedPos = rb:getPos(pos)
                Registry:destroyEntity(entity)
                boxData.deleted = true

                -- Stop blinking
                self.timer:stop("colorChange_" .. id)

                -- Start recreation timer
                self.timer:start("recreate_" .. id, 3 + math.random() * 3, false)
            end
        elseif boxData.deleted then
            if self.timer:check("recreate_" .. id) then
                local mesh = Mesh.Box(2.5)
                local mat  = Materials.DebugColor()
                mat:addStaticShaderVar("color", Enums.UniformType.Float3,
                    function() return 1.0, 0.0, 1.0 end)

                local boxEntity = BoxEntity({ { mesh = mesh, material = mat } })
                local rbCmp = boxEntity:get(Physics.RigidBody)
                rbCmp:setRigidBody(RigidBody.CreateBoxFromMesh(mesh))

                local relVec = Vec3f(boxData.relativePos.x, boxData.relativePos.y, boxData.relativePos.z)
                local rotatedVec = self.gridRotation + relVec
                rbCmp:getRigidBody():setPos(Position(
                    self.gridCenter.x + rotatedVec.x,
                    self.gridCenter.y + rotatedVec.y,
                    self.gridCenter.z + rotatedVec.z
                ))
                rbCmp:getRigidBody():setRot(self.gridRotation * boxData.rotation)

                Registry:attachEntity(self.gridParentEntity, boxEntity)

                boxData.entity = boxEntity
                boxData.deleted = false
                boxData.deletedPos = nil

                -- Reset delete timer
                self.timer:reset("delete_" .. id)
                self.timer:resume("delete_" .. id)
            end
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
    CameraSystem.currentCameraTransform:setPosition(Position(camPos.x, camPos.y, camPos.z))
    CameraSystem.currentCameraTransform:setRotation(Quat.LookAt(camPos, self.gridCenter, Vec3f(0, 1, 0)))
end

function RenderingTest:onRender(data)
    RenderCoreSystem:render(data)

    self:immediateUI(function()
        local mem = GC.GetMemory()
        UI.DrawEx.TextAdditive('Unageo-Medium', self.fpsText, 20,
            40, 50, 40, 20, 0.9, 0.9, 0.9, 0.9, 0.0, 0.5)
        UI.DrawEx.TextAdditive('Unageo-Medium', string.format("Lua Memory: %.2f KB", mem),
            20, 40, 70, 40, 20, 0.9, 0.9, 0.9, 0.9, 0.0, 0.5)

        for _, boxData in ipairs(self.boxes) do
            if boxData.deleted and boxData.deletedPos then
                local pos = boxData.deletedPos
                UI.DrawEx.TextAdditive('Unageo-Medium',
                    string.format("Entity %d Deleted", boxData.id),
                    20, pos.x + RenderCoreSystem.resX / 2, pos.y + RenderCoreSystem.resY / 2,
                    40, 20, 1.0, 0.3, 0.3, 1.0, 0.5, 0.5)
            end
        end
    end)
end

return RenderingTest
