local ActionBinding = require("Input.ActionBinding")
local Control = require("Input.Control")

return {
    Fire = ActionBinding({
        mouse = { Control.Single(Button.MouseLeft) },
    }),
    Volley = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardKey1) },
    }),
    Sequence = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardKey2) },
    }),
    AI = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardA) },
    }),
    Reset = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardR) },
    }),
}
