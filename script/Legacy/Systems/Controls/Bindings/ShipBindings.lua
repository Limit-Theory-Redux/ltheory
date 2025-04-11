local Control = require('Legacy.Systems.Controls.Control')

local self = {
    ThrustX = Control.Or(
        Control.Pair(
            Control.Key(Button.KeyboardD),
            Control.Key(Button.KeyboardA)),
        Control.GamepadAxis(Button.GamepadLeftStickX)),

    ThrustZ = Control.Or(
        Control.Pair(
            Control.Key(Button.KeyboardW),
            Control.Key(Button.KeyboardS)),
        Control.GamepadAxis(Button.GamepadLeftStickY)),

    ThrustY = Control.Or(
        Control.Pair(
            Control.Key(Button.KeyboardSpace),
            Control.Ctrl()),
        Control.Pair(
            Control.GamepadAxis(Button.GamepadDPadUp),
            Control.GamepadAxis(Button.GamepadDPadDown))),

    Roll = Control.Or(
        Control.Pair(
            Control.Key(Button.KeyboardE),
            Control.Key(Button.KeyboardQ)),
        Control.Pair(
            Control.GamepadButton(Button.GamepadRightTrigger2),
            Control.GamepadButton(Button.GamepadLeftTrigger2))),

    Yaw = Control.Or(
        Control.MouseX(),
        Control.GamepadAxis(Button.GamepadRightStickX)),

    Pitch = Control.Or(
        Control.MouseY():invert(),
        Control.GamepadAxis(Button.GamepadRightStickY):invert()),

    Boost = Control.Or(
        Control.Shift(),
        Control.GamepadAxis(Button.GamepadLeftTrigger)),

    Fire = Control.Or(
        Control.MouseButton(Button.MouseLeft),
        Control.GamepadAxis(Button.GamepadRightTrigger)),

    LockTarget = Control.Or(
            Control.Key(Button.KeyboardT),
            Control.GamepadButton(Button.GamepadWest))
        :delta(),

    ClearTarget = Control.Or(
            Control.Key(Button.KeyboardG),
            Control.GamepadButton(Button.GamepadEast))
        :delta(),

    NearestTarget = Control.Or(
            Control.Key(Button.KeyboardN),
            Control.GamepadButton(Button.GamepadWest))
        :delta(),

    Dock = Control.Or(
            Control.Key(Button.KeyboardF),
            Control.GamepadButton(Button.GamepadNorth))
        :delta(),

    Undock = Control.Or(
            Control.Key(Button.KeyboardJ),
            Control.GamepadButton(Button.GamepadSouth))
        :delta(),

    SquadAttackTarget = Control.GamepadButton(Button.GamepadSelect):delta(),
    SquadScramble     = Control.GamepadButton(Button.GamepadMode):delta(),
}

if GameState.input.invertPitch then
    self.Pitch = self.Pitch:invert()
end

return self
