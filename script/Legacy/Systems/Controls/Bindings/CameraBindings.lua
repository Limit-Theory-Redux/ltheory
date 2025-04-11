local Control = require('Legacy.Systems.Controls.Control')

local self = {
    Yaw        = Control.Or(
        Control.And(
            Control.MouseButton(Button.MouseRight),
            Control.MouseDX()),
        Control.GamepadAxis(Button.GamepadRightStickX)
        :setMult(3):setExponent(2)),

    Pitch      = Control.Or(
        Control.And(
            Control.MouseButton(Button.MouseRight),
            Control.MouseDY()),
        Control.GamepadAxis(Button.GamepadRightStickY)
        :setMult(3):setExponent(2):invert()),
    Zoom       = Control.MouseWheel(),

    TranslateX = Control.Pair(Control.Key(Button.KeyboardD), Control.Key(Button.KeyboardA)),
    TranslateY = Control.Pair(Control.Key(Button.KeyboardSpace), Control.Key(Button.KeyboardControlLeft)),
    TranslateZ = Control.Pair(Control.Key(Button.KeyboardW), Control.Key(Button.KeyboardS)),
}

return self
