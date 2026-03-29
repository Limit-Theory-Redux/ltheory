local ActionBinding = require('Input.ActionBinding')
local Control = require('Input.Control')

--- General actions shared across game states (camera cycling, map, autopilot, debug)
local GeneralActions = {
    ---@type ActionBinding
    CycleCamera = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardC) },
        gamepad = { Control.Single(Button.GamepadSelect) },
    }),

    ---@type ActionBinding
    CycleMapMode = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardM) },
    }),

    ---@type ActionBinding
    AutoPilotToggle = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardN) },
    }),

    ---@type ActionBinding
    ReloadShaders = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardF5) },
    }),

    ---@type ActionBinding
    Regenerate = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardB) },
    }),
}

return GeneralActions
