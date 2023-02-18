local CameraBindings = require('Systems.Controls.Bindings.CameraBindings')
local Disposition = require('GameObjects.Components.NPC.Dispositions')

local BackgroundControl = {}
BackgroundControl.__index = BackgroundControl
setmetatable(BackgroundControl, UI.Panel)

BackgroundControl.name      = 'BackgroundControl'
BackgroundControl.focusable = false

function BackgroundControl:controlThrust (e)
  if not e:hasThrustController() then return end
  local c = e:getThrustController()
  c:setThrust(
    0,
    0,
    0,
    0.03,       --Flat: spin the invisible ship
    -0.01,      --      ever so slightly
    0,
    0)
  self.aimX = c.yaw
  self.aimY = c.pitch
end

function BackgroundControl:onInput (state)
  local camera = self.gameView.camera
  camera:push()
  camera:modRadius(exp(-0.1 * CameraBindings.Zoom:get()))
  local e = self.player:getControlling()
  self:controlThrust(e)
  camera:pop()
end

function BackgroundControl.Create (gameView, player)
  local self = setmetatable({
    gameView        = gameView,
    player          = player,
    icon            = UI.Icon(),

    children  = List(),
  }, BackgroundControl)

  return self
end

return BackgroundControl
