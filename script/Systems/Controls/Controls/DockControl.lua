local Actions = requireAll('GameObjects.Actions')
local ShipBindings = require('Systems.Controls.Bindings.ShipBindings')
local CameraBindings = require('Systems.Controls.Bindings.CameraBindings')

local DockControl = {}
DockControl.__index = DockControl
setmetatable(DockControl, UI.Panel)

function DockControl:onEnable ()
  local pCamera = self.gameView.camera
  self.gameView:setCameraMode(Enums.CameraMode.Orbit)

  local station = self.player:getControlling():getParent()

  self.camera = self.gameView.camera
  self.camera:setYaw(-Math.Pi2)
  self.camera:setPitch(Math.Pi4)
  self.camera:setRadius(1000)
  self.camera:setTarget(station)
  self.camera:setRelative(true)
  self.camera:warp()
  self.camera:lerpFrom(pCamera.pos, pCamera.rot)
end

function DockControl:onInput (state)
  if not GameState.paused and ShipBindings.Undock:get() > 0 then
    printf("*** Undocking (manual)!")
    self.player:getControlling():pushAction(Actions.Undock())
    Input.SetMouseVisible(false)
  end

  self.camera:push()
  self.camera:modYaw(        0.005 * CameraBindings.Yaw:get())
  self.camera:modPitch(      0.005 * CameraBindings.Pitch:get())
  self.camera:modRadius(exp(-0.1  * CameraBindings.Zoom:get()))
  self.camera:pop()
end

function DockControl:onDraw (focus, active)
  -- TODO : Unify this with HUD
  local x, y, sx, sy = self:getRectGlobal()
  local cx, cy = sx / 2, sy / 2

  local dockText = "Press J to Undock" -- TODO: Connect Undocking input to bindings

  UI.DrawEx.TextAdditive(
    "NovaMono",
    dockText,
    24,
    cx, cy - 68, 1, 1,
    0, 0, 0, 1,
    0.5, 0.5
  )
  UI.DrawEx.TextAdditive(
    "NovaMono",
    dockText,
    24,
    cx, cy - 68, 1, 1,
    1, 1, 1, 1,
    0.5, 0.5
  )
end

function DockControl:onDrawIcon (iconButton, focus, active)
  -- Draw Dock Control icon
  local borderColor = iconButton == active
                      and Config.ui.color.controlActive
                      or iconButton == focus
                         and Config.ui.color.controlFocused
                         or Config.ui.color.control
  local contentColor = self:isEnabled()
                       and Config.ui.color.controlFocused
                       or Config.ui.color.control

  local x, y, sx, sy = iconButton:getRectGlobal()
  UI.DrawEx.RectOutline(x, y, sx, sy, borderColor)
  UI.DrawEx.Ring(x + sx/2, y + 10 +  8, 12, contentColor, false)
  UI.DrawEx.Ring(x + sx/2, y + 10 + 14,  9, contentColor, false)
  UI.DrawEx.Ring(x + sx/2, y + 10 + 20,  6, contentColor, false)
end

function DockControl.Create (gameView, player)
  local self = setmetatable({
    gameView = gameView,
    player   = player,
    icon     = UI.Icon(),
    camera   = nil,

    children = List(),
  }, DockControl)

  self.icon:setOnDraw(function (ib, focus, active)
    self:onDrawIcon(ib, focus, active)
  end)

  return self
end

return DockControl
