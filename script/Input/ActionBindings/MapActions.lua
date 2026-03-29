local ActionBinding = require('Input.ActionBinding')
local Control = require('Input.Control')

--- Actions for 2D and 3D system map interaction
local MapActions = {
    ---@type ActionBinding
    Select = ActionBinding({
        mouse = { Control.Single(Button.MouseLeft) },
    }),

    ---@type ActionBinding
    Pan = ActionBinding({
        mouse = { Control.Single(Button.MouseMiddle) },
    }),

    ---@type ActionBinding
    Drag = ActionBinding({
        mouse = { Control.Single(Button.MouseRight) },
    }),

    ---@type ActionBinding
    Zoom = ActionBinding({
        mouse = { Control.MouseWheel() },
    }),

    -- Movement for 3D map
    ---@type ActionBinding
    MoveX = ActionBinding({
        keyboard = { Control.Pair(Button.KeyboardD, Button.KeyboardA) },
    }),

    ---@type ActionBinding
    MoveZ = ActionBinding({
        keyboard = { Control.Pair(Button.KeyboardW, Button.KeyboardS) },
    }),

    ---@type ActionBinding
    RotateYaw = ActionBinding({
        mouse = { Control.Combo(Button.MouseRight, Control.MouseDX()) },
    }),

    ---@type ActionBinding
    RotatePitch = ActionBinding({
        mouse = { Control.Combo(Button.MouseRight, Control.MouseDY()) },
    }),
}

return MapActions
