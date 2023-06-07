local Control = require('Systems.Controls.Control')

local self = {
    ThrustX           = Control.Or(
        Control.Pair(
            Control.Key(Button.Keyboard.D),
            Control.Key(Button.Keyboard.A)),
        Control.GamepadAxis(Button.Gamepad.LStickX)),

    ThrustZ           = Control.Or(
        Control.Pair(
            Control.Key(Button.Keyboard.W),
            Control.Key(Button.Keyboard.S)),
        Control.GamepadAxis(Button.Gamepad.LStickY)),

    ThrustY           = Control.Or(
        Control.Pair(
            Control.Key(Button.Keyboard.Space),
            Control.Ctrl()),
        Control.Pair(
            Control.GamepadAxis(Button.Gamepad.Up),
            Control.GamepadAxis(Button.Gamepad.Down))),

    Roll              = Control.Or(
        Control.Pair(
            Control.Key(Button.Keyboard.E),
            Control.Key(Button.Keyboard.Q)),
        Control.Pair(
            Control.GamepadButton(Button.Gamepad.RBumper),
            Control.GamepadButton(Button.Gamepad.LBumper))),

    Yaw               = Control.Or(
        Control.MouseX(),
        Control.GamepadAxis(Button.Gamepad.RStickX)),

    Pitch             = Control.Or(
        Control.MouseY():invert(),
        Control.GamepadAxis(Button.Gamepad.RStickY):invert()),

    Boost             = Control.Or(
        Control.Shift(),
        Control.GamepadAxis(Button.Gamepad.LTrigger)),

    Fire              = Control.Or(
        Control.MouseButton(Button.Mouse.Left),
        Control.GamepadAxis(Button.Gamepad.RTrigger)),

    LockTarget        = Control.Or(
            Control.Key(Button.Keyboard.T),
            Control.GamepadButton(Button.Gamepad.X))
        :delta(),

    ClearTarget       = Control.Or(
            Control.Key(Button.Keyboard.G),
            Control.GamepadButton(Button.Gamepad.B))
        :delta(),

    NearestTarget     = Control.Or(
            Control.Key(Button.Keyboard.N),
            Control.GamepadButton(Button.Gamepad.X))
        :delta(),

    Dock              = Control.Or(
            Control.Key(Button.Keyboard.F),
            Control.GamepadButton(Button.Gamepad.Y))
        :delta(),

    Undock            = Control.Or(
            Control.Key(Button.Keyboard.J),
            Control.GamepadButton(Button.Gamepad.A))
        :delta(),

    SquadAttackTarget = Control.GamepadButton(Button.Gamepad.Back):delta(),
    SquadScramble     = Control.GamepadButton(Button.Gamepad.Guide):delta(),
}

if GameState.input.invertPitch then
    self.Pitch = self.Pitch:invert()
end

return self
