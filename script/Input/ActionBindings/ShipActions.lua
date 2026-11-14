local ActionBinding = require('Input.ActionBinding')
local Control = require('Input.Control')

local ShipActions = {
    ---@type ActionBinding
    ThrustX = ActionBinding({
        label = "Strafe right / left", group = "Flight",
        keyboard = { Control.Pair(Button.KeyboardD, Button.KeyboardA) },
        gamepad = { Control.GamepadAxis(Button.GamepadLeftStickX) },
    }),

    ---@type ActionBinding
    ThrustZ = ActionBinding({
        label = "Thrust forward / back", group = "Flight",
        keyboard = { Control.Pair(Button.KeyboardW, Button.KeyboardS) },
        gamepad = { Control.GamepadAxis(Button.GamepadLeftStickY) },
    }),

    ---@type ActionBinding
    ThrustY = ActionBinding({
        label = "Up / down", group = "Flight",
        keyboard = { Control.Pair(Button.KeyboardSpace, Button.KeyboardControlLeft) },
        gamepad = { Control.Pair(Button.GamepadDPadUp, Button.GamepadDPadDown) },
    }),

    ---@type ActionBinding
    Roll = ActionBinding({
        label = "Roll right / left", group = "Flight",
        keyboard = { Control.Pair(Button.KeyboardE, Button.KeyboardQ) },
        gamepad = { Control.Pair(Button.GamepadRightTrigger2, Button.GamepadLeftTrigger2) },
    }),

    ---@type ActionBinding
    Yaw = ActionBinding({
        label = "Steer left / right", group = "Flight",
        mouse = { Control.MouseX() },
        gamepad = { Control.GamepadAxis(Button.GamepadRightStickX) },
    }),

    ---@type ActionBinding
    Pitch = ActionBinding({
        label = "Steer up / down", group = "Flight",
        mouse = { Control.MouseY():invert() },
        gamepad = { Control.GamepadAxis(Button.GamepadRightStickY):invert() },
    }),

    ---@type ActionBinding
    Boost = ActionBinding({
        label = "Boost", group = "Flight",
        keyboard = { Control.Single(Button.KeyboardShiftLeft), Control.Single(Button.KeyboardShiftRight) },
        gamepad = { Control.GamepadAxis(Button.GamepadLeftTrigger) },
    }),

    ---@type ActionBinding
    Fire = ActionBinding({
        label = "Fire", group = "Flight",
        mouse = { Control.Single(Button.MouseLeft) },
        gamepad = { Control.GamepadAxis(Button.GamepadRightTrigger) },
    }),

    ---@type ActionBinding
    LockTarget = ActionBinding({
        hidden = true, -- not used by the game yet
        keyboard = { Control.Single(Button.KeyboardT) },
        gamepad = { Control.Single(Button.GamepadWest) },
    }),

    ---@type ActionBinding
    ClearTarget = ActionBinding({
        hidden = true, -- not used by the game yet
        keyboard = { Control.Single(Button.KeyboardG) },
        gamepad = { Control.Single(Button.GamepadEast) },
    }),

    ---@type ActionBinding
    NearestTarget = ActionBinding({
        hidden = true, -- not used by the game yet
        keyboard = { Control.Single(Button.KeyboardN), },
        gamepad = { Control.Single(Button.GamepadWest) },
    }),

    ---@type ActionBinding
    Dock = ActionBinding({
        hidden = true, -- not used by the game yet
        keyboard = { Control.Single(Button.KeyboardF) },
        gamepad = { Control.Single(Button.GamepadNorth) },
    }),

    ---@type ActionBinding
    Undock = ActionBinding({
        hidden = true, -- not used by the game yet
        keyboard = { Control.Single(Button.KeyboardJ) },
        gamepad = { Control.Single(Button.GamepadSouth) },
    }),

    ---@type ActionBinding
    TravelDrive = ActionBinding({
        label = "Travel drive", group = "Flight",
        keyboard = { Control.Single(Button.KeyboardT) },
        gamepad = { Control.Single(Button.GamepadLeftThumb) },
    }),

    ---@type ActionBinding
    SquadAttackTarget = ActionBinding({
        hidden = true, -- not used by the game yet
        gamepad = { Control.Single(Button.GamepadSelect) },
    }),

    ---@type ActionBinding
    SquadScramble = ActionBinding({
        hidden = true, -- not used by the game yet
        gamepad = { Control.Single(Button.GamepadMode) },
    }),
}

-- handle pitch invert
if GameState and GameState.input and GameState.input.invertPitch then
    local order = ShipActions.Pitch.order -- keep its place in the Controls screen
    ---@type ActionBinding
    ShipActions.Pitch = ActionBinding({
        label = "Steer up / down", group = "Flight",
        mouse = { Control.MouseY() },  -- non-inverted
        gamepad = { Control.GamepadAxis(Button.GamepadRightStickY) },
    })
    ShipActions.Pitch.order = order
end

return ShipActions
