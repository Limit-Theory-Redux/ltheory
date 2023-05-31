local CameraBindings = require('Systems.Controls.Bindings.CameraBindings')
local CommandBindings = require('Systems.Controls.Bindings.CommandBindings')
local Disposition = require('GameObjects.Components.NPC.Dispositions')
local Entity = require('GameObjects.Entity')

local GenTestControl = {}
GenTestControl.__index = GenTestControl
setmetatable(GenTestControl, UI.Panel)

GenTestControl.name      = 'Command Control'
GenTestControl.focusable = true
GenTestControl.draggable = true
GenTestControl:setPadUniform(8)

local selectionPredicate = function (unit) return unit:isAlive() end

local SelectionMode = {
  Replace = 1,
  Toggle  = 2,
  Add     = 3,
  Remove  = 4,
}

function GenTestControl:onEnable ()
  local pCamera = self.gameView.camera
  self.camera = self.gameView:setCameraMode(Enums.CameraMode.Orbit)

  if self.firstRun then
    self.firstRun = false
    self.camera:setYaw(-Math.Pi2)
    self.camera:setPitch(Math.Pi2)
    self.camera:setRadius(1000)
  end
  self.camera:setTarget(GameState.player.humanPlayer:getControlling())
  self.camera:setRelative(false)

  self.camera:warp()
  self.camera:lerpFrom(pCamera.pos, pCamera.rot)
end

function GenTestControl:onDisable ()
  self.gameView:setCameraMode(GameState.player.lastCamera)
end

function GenTestControl:onInput (state)
  self.camera:push()
  self.camera:modYaw(        0.005 * CameraBindings.Yaw:get())
  self.camera:modPitch(      0.005 * CameraBindings.Pitch:get())

  if self.camera.modRadius then
    self.camera:modRadius(exp(-0.1  * CameraBindings.Zoom:get()))
  end

  self.camera:pop()
end

function GenTestControl:onUpdate (state)

end

function GenTestControl:onDraw (focus, active)

end

function GenTestControl:onDrawIcon (iconButton, focus, active)

end

function GenTestControl.Create (gameView, player)
  local self = setmetatable({
    gameView      = gameView,
    player        = player,
    icon          = UI.Icon(),
    camera        = nil,
    firstRun      = true,

    children      = List(),
  }, GenTestControl)

  return self
end

return GenTestControl
