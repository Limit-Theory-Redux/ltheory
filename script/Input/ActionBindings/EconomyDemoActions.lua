local ActionBinding = require('Input.ActionBinding')
local Control = require('Input.Control')

--- Actions of the economy demo: pick the station or the ship the camera follows
local EconomyDemoActions = {
    ---@type ActionBinding
    Prev = ActionBinding({
        label = "Previous station or ship", group = "Economy demo",
        keyboard = { Control.Single(Button.KeyboardLeft) },
    }),

    ---@type ActionBinding
    Next = ActionBinding({
        label = "Next station or ship", group = "Economy demo",
        keyboard = { Control.Single(Button.KeyboardRight) },
    }),
}

return EconomyDemoActions
