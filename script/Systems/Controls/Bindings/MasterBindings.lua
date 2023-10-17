local Control = require('Systems.Controls.Control')

local self = {
    TogglePanel = Control.Key(Button.KeyboardBackslash):delta(),
    Controls    = {
        Control.Key(Button.KeyboardKey1):delta(),
        Control.Key(Button.KeyboardKey2):delta(),
        Control.Key(Button.KeyboardKey3):delta(),
    },
}

return self
