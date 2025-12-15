local ActionBinding = require('Input.ActionBinding')
local Control = require('Input.Control')

local GamepadCursorActions = {
    ---@type ActionBinding
    MoveX = ActionBinding({
        gamepad = {
            Control.GamepadAxis(Button.GamepadLeftStickX),
            Control.GamepadAxis(Button.GamepadRightStickX)
        }
    }),

    ---@type ActionBinding
    MoveY = ActionBinding({
        gamepad = {
            Control.GamepadAxis(Button.GamepadLeftStickY),
            Control.GamepadAxis(Button.GamepadRightStickY)
        }
    }),

    ---@type ActionBinding
    Boost = ActionBinding({
        gamepad = { 
            Control.GamepadAxis(Button.GamepadLeftTrigger2),
            Control.GamepadAxis(Button.GamepadRightTrigger2)
        },
    }),

    ---@type ActionBinding
    Confirm = ActionBinding({
        gamepad = { Control.Single(Button.GamepadSouth) },
    }),

    ---@type ActionBinding
    Cancel = ActionBinding({
        gamepad = { Control.Single(Button.GamepadEast) },
    })
}

return GamepadCursorActions