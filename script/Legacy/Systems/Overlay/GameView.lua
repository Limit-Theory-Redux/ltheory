local Material = require('Legacy.GameObjects.Material')
local Registry = require('Core.ECS.Registry')
local RenderSystem = require('Modules.Rendering.Systems.RenderSystem')

local GameView = {}
GameView.__index = GameView
setmetatable(GameView, UI.Container)

GameView.name = 'Game View'

function GameView:draw(focus, active)
    for i = 1, #self.children do
        self.children[i]:draw(focus, active)
    end
end

function GameView:onInputChildren(state)
    self.camera:push()
    for i = 1, #self.children do
        local child = self.children[i]
        if not child.removed then child:input(state) end
    end
    self.camera:pop()
end

function GameView:onUpdate(state)
    --[[ TODO : This may be one frame delayed since onUpdateChildren happens later
              and one of them is responsible for updating the camera position.
              Further reason to invert the current Camera-Control relationship. ]]
    self.camera:onUpdate(state.dt)

    do -- Compute Eye Velocity EMA
        local eye = self.camera.pos
        local v = (eye - self.eyeLast):scale(1.0 / max(1e-10, state.dt))
        self.eyeVel:setv(self.player:getControlling():getVelocity())
        self.eyeLast:setv(eye)
    end

    self.audio:setListenerPos(self.camera.pos)
    self.audio:setListenerRot(self.camera.rot)

    self.camera:pop()
end

function GameView:onUpdateChildren(state)
    self.camera:push()
    for i = 1, #self.children do
        local child = self.children[i]
        if not child.removed then child:update(state) end
    end
    self.camera:pop()
end

function GameView:onLayoutSizeChildren()
    self.camera:push()
    for i = 1, #self.children do self.children[i]:layoutSize() end
    self.camera:pop()
end

function GameView:setCameraMode(cameraMode)
    local lastCamera = self.camera
    GameState.player.lastCamera = GameState.player.currentCamera

    if cameraMode == Enums.CameraMode.FirstPerson then
        self.camera = self.cameraFirstPerson

        if GameState.player.currentShip then
            -- hide ship mesh
            GameState.player.currentShip:setRenderVisibleMesh(false, true)
        end
    elseif cameraMode == Enums.CameraMode.Chase then
        self.camera = self.cameraChase

        if GameState.player.currentShip then
            -- hide ship mesh
            GameState.player.currentShip:setRenderVisibleMesh(true, false)
        end
    elseif cameraMode == Enums.CameraMode.Orbit then
        self.camera = self.cameraOrbit
        self.camera:setRelative(true)

        if GameState.player.currentShip then
            -- hide ship mesh
            GameState.player.currentShip:setRenderVisibleMesh(true, false)
        end
    else
        error("Invalid camera mode passed")
    end

    RenderSystem:setCamera(self.camera)
    GameState.player.currentCamera = cameraMode
    self.camera:setTarget(self.player:getControlling())

    -- NOTE : We're assuming that no one else could have pushed a camera
    local camera = Systems.Camera.Camera.get()
    if camera and camera == lastCamera then
        lastCamera:pop()
        self.camera:push()
    end
    return self.camera
end

function GameView.Create(player, audioInstance)
    if not player then
        Log.Error("No player passed")
    end

    if not audioInstance then
        Log.Error("No audioInstance passed")
    end

    local self = setmetatable({
        player            = player,
        cameraFirstPerson = Systems.Camera.CameraFirstPerson(),
        cameraChase       = Systems.Camera.CameraChase(),
        cameraOrbit       = Systems.Camera.CameraOrbit(),
        camera            = nil,
        eyeLast           = nil,
        eyeVel            = nil,
        children          = List(),
        audio             = audioInstance
    }, GameView)

    self:setCameraMode(GameState.player.currentCamera)
    self.eyeLast = self.camera.pos:clone()
    self.eyeVel  = self.player:getControlling():getVelocity():clone()
    return self
end

return GameView
