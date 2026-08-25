local ActionBinding = require("Input.ActionBinding")
local Control = require("Input.Control")

return {
    Spawn = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardF) },
    }),
    Burst = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardG) },
    }),
    Grid = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardL) },
    }),
    Clear = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardC) },
    }),
    SizeUp = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardE) },
    }),
    SizeDown = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardQ) },
    }),
    DurationUp = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardX) },
    }),
    DurationDown = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardZ) },
    }),
    Preset = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardT) },
    }),
    LoopRing = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardK) },
    }),
    StepBack = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardComma) },
    }),
    StepFwd = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardPeriod) },
    }),
    Freeze = ActionBinding({
        keyboard = { Control.Single(Button.KeyboardM) },
    }),
}
