local Control = require('Systems.Controls.Control')

return {
    Select    = Control.Or(
        Control.MouseButton(Button.MouseLeft),
        Control.Key(Button.KeyboardEnter)
    ):delta(),
    Cancel    = Control.Or(
        Control.MouseButton(Button.MouseRight),
        Control.Key(Button.KeyboardEscape),
        Control.Key(Button.KeyboardShiftLeft)
    ):delta(),
    Up        = Control.Key(Button.KeyboardW):delta(),
    Down      = Control.Key(Button.KeyboardS):delta(),
    Left      = Control.Key(Button.KeyboardA):delta(),
    Right     = Control.Key(Button.KeyboardD):delta(),
    PrevGroup = Control.Key(Button.KeyboardQ):delta(),
    NextGroup = Control.Key(Button.KeyboardE):delta(),
    ScrollH   = Control.MouseButton(Button.MouseScrollX),
    ScrollV   = Control.MouseButton(Button.MouseScrollY),
}
