local CameraBindings = require('Systems.Controls.Bindings.CameraBindings')
local Disposition = require('GameObjects.Elements.NPC.Dispositions')

local BackgroundControl = {}
BackgroundControl.__index = BackgroundControl
setmetatable(BackgroundControl, UI.Panel)

BackgroundControl.name      = 'BackgroundControl'
BackgroundControl.focusable = false

function BackgroundControl:controlThrust(e)
    if not e:hasThrustController() then return end
    local c = e:getThrustController()
    c:setThrust(
        0,
        0,
        0,
        0.03,  -- spin the invisible ship
        -0.01, -- ever so slightly
        0,
        0)
    self.aimX = c.yaw
    self.aimY = c.pitch
end

function BackgroundControl:onInput(state)
    local camera = self.gameView.camera
    camera:push()

    if camera.modRadius then
        camera:modRadius(exp(-0.1 * CameraBindings.Zoom:get()))
    end

    local e = self.player:getControlling()
    self:controlThrust(e)
    camera:pop()
end

function BackgroundControl:onDrawIcon(iconButton, focus, active)
    -- Draw Background Control icon
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
    UI.DrawEx.Ring(x + sx / 2, y + 22, 15, contentColor, false)
    UI.DrawEx.Ring(x + sx / 2, y + 22, 10, contentColor, false)
    UI.DrawEx.Ring(x + sx / 2, y + 22, 5, contentColor, false)
end

function BackgroundControl.Create(gameView, player)
    local self = setmetatable({
        gameView = gameView,
        player   = player,
        icon     = UI.Icon(),

        children = List(),
    }, BackgroundControl)

    self.gameView:setCameraMode(Enums.CameraMode.Chase)

    self.icon:setOnDraw(function(ib, focus, active)
        self:onDrawIcon(ib, focus, active)
    end)

    return self
end

return BackgroundControl
