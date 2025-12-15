local ActionBinding = require('Input.ActionBinding')
local Control = require('Input.Control')

local ShipActions = {
    ---@type ActionBinding
    ThrustX = ActionBinding({
        keyboard = { Control.Pair(Button.KeyboardD, Button.KeyboardA) },
        gamepad = { Control.GamepadAxis(Button.GamepadLeftStickX) },
    }),

    ---@type ActionBinding
    ThrustZ = ActionBinding({
        keyboard = { Control.Pair(Button.KeyboardW, Button.KeyboardS) },
        gamepad = { Control.GamepadAxis(Button.GamepadLeftStickY) },
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
        gamepad = { Control.GamepadAxis(Button.GamepadRightStickX) },
    }),

    ---@type ActionBinding
    Pitch = ActionBinding({
        mouse = { Control.MouseDY():setMult(-0.1) },
        gamepad = { Control.GamepadAxis(Button.GamepadRightStickY):invert() },
    }),

    ---@type ActionBinding
    Boost = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardShiftLeft), Control.Single(Button.KeyboardShiftRight) },
        gamepad = { Control.GamepadAxis(Button.GamepadLeftTrigger) },
    }),

    ---@type ActionBinding
    Fire = ActionBinding({
        mouse = { Control.Single(Button.MouseLeft) },
        gamepad = { Control.GamepadAxis(Button.GamepadRightTrigger) },
    }),

    ---@type ActionBinding
    LockTarget = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardT) },
        gamepad = { Control.Single(Button.GamepadWest) },
    }),

    ---@type ActionBinding
    ClearTarget = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardG) },
        gamepad = { Control.Single(Button.GamepadEast) },
    }),

    ---@type ActionBinding
    NearestTarget = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardN), },
        gamepad = { Control.Single(Button.GamepadWest) },
    }),

    ---@type ActionBinding
    Dock = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardF) },
        gamepad = { Control.Single(Button.GamepadNorth) },
    }),

    ---@type ActionBinding
    Undock = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardJ) },
        gamepad = { Control.Single(Button.GamepadSouth) },
    }),

    ---@type ActionBinding
    SquadAttackTarget = ActionBinding({
        gamepad = { Control.Single(Button.GamepadSelect) },
    }),

    ---@type ActionBinding
    SquadScramble = ActionBinding({
        gamepad = { Control.Single(Button.GamepadMode) },
    }),
}

-- handle pitch invert
if GameState and GameState.input and GameState.input.invertPitch then
    -- re-create Pitch binding with non-inverted values
    ---@type ActionBinding
    ShipActions.Pitch = ActionBinding({
        mouse = { Control.MouseY() },
        gamepad = { Control.GamepadAxis(Button.GamepadRightStickY) },
    })
end

return ShipActions
