local ActionBinding = require('Input.ActionBinding')
local Control = require('Input.Control')

--- General actions shared across game states (camera cycling, map, autopilot, debug)
local GeneralActions = {
    ---@type ActionBinding
    CycleCamera = ActionBinding({
        label = "Cycle camera", group = "Camera",
        keyboard = { Control.Single(Button.KeyboardC) },
        gamepad = { Control.Single(Button.GamepadSelect) },
    }),

    ---@type ActionBinding
    CycleMapMode = ActionBinding({
        label = "Open / switch map", group = "Map",
        keyboard = { Control.Single(Button.KeyboardM) },
    }),

    ---@type ActionBinding
    AutoPilotToggle = ActionBinding({
        label = "Autopilot to selection / cancel", group = "Map",
        keyboard = { Control.Single(Button.KeyboardN) },
    }),

    ---@type ActionBinding
    ReloadShaders = ActionBinding({
        label = "Reload shaders", group = "Debug",
        keyboard = { Control.Single(Button.KeyboardF5) },
    }),

    --- Render info overlay (Application): Ctrl+Shift+F3, either Ctrl and either Shift
    ---@type ActionBinding
    RenderInfoOverlay = ActionBinding({
        label = "Render info", group = "Debug",
        keyboard = {
            Control.Combo(Button.KeyboardControlLeft, Button.KeyboardShiftLeft, Button.KeyboardF3),
            Control.Combo(Button.KeyboardControlLeft, Button.KeyboardShiftRight, Button.KeyboardF3),
            Control.Combo(Button.KeyboardControlRight, Button.KeyboardShiftLeft, Button.KeyboardF3),
            Control.Combo(Button.KeyboardControlRight, Button.KeyboardShiftRight, Button.KeyboardF3),
        },
    }),

    ---@type ActionBinding
    Regenerate = ActionBinding({
        label = "Regenerate system", group = "Debug",
        keyboard = { Control.Single(Button.KeyboardB) },
    }),
}

return GeneralActions
