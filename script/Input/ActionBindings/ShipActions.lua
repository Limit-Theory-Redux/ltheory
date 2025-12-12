local ActionBinding = require('Input.ActionBinding')
local Control = require('Input.Control')

local ShipActions = {
    ---@type ActionBinding
    ThrustX = ActionBinding({
        keyboard = { Control.Pair(Button.KeyboardD, Button.KeyboardA) },
        gamepad = { Control.Axis(Button.GamepadLeftStickX) },
    }),

    ---@type ActionBinding
    ThrustZ = ActionBinding({
        keyboard = { Control.Pair(Button.KeyboardW, Button.KeyboardS) },
        gamepad = { Control.Axis(Button.GamepadLeftStickY) },
    }),

    ---@type ActionBinding
    ThrustY = ActionBinding({
        keyboard = { Control.Pair(Button.KeyboardSpace, Button.KeyboardControlLeft) },
        gamepad = { Control.Pair(Button.GamepadDPadUp, Button.GamepadDPadDown) },
    }),

    ---@type ActionBinding
    Roll = ActionBinding({
        keyboard = { Control.Pair(Button.KeyboardE, Button.KeyboardQ) },
        gamepad = { Control.Pair(Button.GamepadRightTrigger2, Button.GamepadLeftTrigger2) },
    }),

    ---@type ActionBinding
    Yaw = ActionBinding({
        mouse = { Control.MouseDX():setMult(0.1) },
        gamepad = { Control.Axis(Button.GamepadRightStickX) },
    }),

    ---@type ActionBinding
    Pitch = ActionBinding({
        mouse = { Control.MouseDY():setMult(-0.1) },
        gamepad = { Control.Axis(Button.GamepadRightStickY):invert() },
    }),

    ---@type ActionBinding
    Boost = ActionBinding({
        keyboard = { Button.KeyboardShiftLeft, Button.KeyboardShiftRight },
        gamepad = { Control.Axis(Button.GamepadLeftTrigger) },
    }),

    ---@type ActionBinding
    Fire = ActionBinding({
        mouse = { Button.MouseLeft },
        gamepad = { Control.Axis(Button.GamepadRightTrigger) },
    }),

    ---@type ActionBinding
    LockTarget = ActionBinding({
        keyboard = { Button.KeyboardT },
        gamepad = { Button.GamepadWest },
    }),

    ---@type ActionBinding
    ClearTarget = ActionBinding({
        keyboard = { Button.KeyboardG },
        gamepad = { Button.GamepadEast },
    }),

    ---@type ActionBinding
    NearestTarget = ActionBinding({
        keyboard = { Button.KeyboardN },
        gamepad = { Button.GamepadWest },
    }),

    ---@type ActionBinding
    Dock = ActionBinding({
        keyboard = { Button.KeyboardF },
        gamepad = { Button.GamepadNorth },
    }),

    ---@type ActionBinding
    Undock = ActionBinding({
        keyboard = { Button.KeyboardJ },
        gamepad = { Button.GamepadSouth },
    }),

    ---@type ActionBinding
    SquadAttackTarget = ActionBinding({
        gamepad = { Button.GamepadSelect },
    }),

    ---@type ActionBinding
    SquadScramble = ActionBinding({
        gamepad = { Button.GamepadMode },
    }),
}

-- handle pitch invert
if GameState and GameState.input and GameState.input.invertPitch then
    -- re-create Pitch binding with non-inverted values
    ---@type ActionBinding
    ShipActions.Pitch = ActionBinding({
        mouse = { Control.MouseY() },
        gamepad = { Control.Axis(Button.GamepadRightStickY) },
    })
end

return ShipActions