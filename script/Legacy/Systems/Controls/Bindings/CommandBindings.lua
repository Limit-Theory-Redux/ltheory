local Control = require('Legacy.Systems.Controls.Control')

local self = {
    Select          = Control.MouseButton(Button.MouseLeft),
    ToggleSelection = Control.Ctrl(),
    AppendSelection = Control.Shift(),

    Context         = Control.MouseButton(Button.MouseRight):delta(),
    SetFocus        = Control.Key(Button.KeyboardF):delta(),
    LockFocus       = Control.Key(Button.KeyboardG):delta(),

    ToggleDetails   = Control.Key(Button.KeyboardR):delta(),

    SetGroup        = Control.Key(Button.KeyboardE):delta(),
    GetGroup        = Control.Key(Button.KeyboardQ):delta(),
    GroupNumber     = {
        Control.Key(Button.KeyboardKey1):delta(),
        Control.Key(Button.KeyboardKey2):delta(),
        Control.Key(Button.KeyboardKey3):delta(),
        Control.Key(Button.KeyboardKey4):delta(),
    },
}

return self
