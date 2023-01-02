local Control = require('Systems.Controls.Control')

local self = {
    Reload           = Control.Key(Button.Keyboard.F5),
    ToggleWireframe  = Control.Key(Button.Keyboard.Tab),
    ProfilerToggle   = Control.Key(Button.Keyboard.F10),
    ToggleFullscreen = Control.Key(Button.Keyboard.F11),
    Screenshot       = Control.Key(Button.Keyboard.F12),
    TimeAccel        = Control.Key(Button.Keyboard.H),
    Exit             = function () return Control.And(Control.Ctrl, Control.Key(Button.Keyboard.Escape)) end,
    ScoreNebulaBad   = Control.Key(Button.Keyboard.Minus),
    ScoreNebulaGood  = Control.Key(Button.Keyboard.Equals),
}

return self
