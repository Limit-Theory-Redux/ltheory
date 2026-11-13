local ActionBinding = require('Input.ActionBinding')
local Control = require('Input.Control')

--- Actions for 2D and 3D system map interaction
local MapActions = {
    ---@type ActionBinding
    Select = ActionBinding({
        label = "Select", group = "Map",
        mouse = { Control.Single(Button.MouseLeft) },
    }),

    ---@type ActionBinding
    Pan = ActionBinding({
        label = "Pan (hold)", group = "Map",
        mouse = { Control.Single(Button.MouseMiddle) },
    }),

    ---@type ActionBinding
    Drag = ActionBinding({
        label = "Pan, alternative (hold)", group = "Map",
        mouse = { Control.Single(Button.MouseRight) },
    }),

    ---@type ActionBinding
    Zoom = ActionBinding({
        label = "Zoom", group = "Map",
        mouse = { Control.MouseWheel() },
    }),

    -- Movement for 3D map
    ---@type ActionBinding
    MoveX = ActionBinding({
        label = "Move right / left (3D map)", group = "Map",
        keyboard = { Control.Pair(Button.KeyboardD, Button.KeyboardA) },
    }),

    ---@type ActionBinding
    MoveZ = ActionBinding({
        label = "Move forward / back (3D map)", group = "Map",
        keyboard = { Control.Pair(Button.KeyboardW, Button.KeyboardS) },
    }),

    ---@type ActionBinding
    RotateYaw = ActionBinding({
        hidden = true, -- not used by the game yet
        mouse = { Control.Combo(Button.MouseRight, Control.MouseDX()) },
    }),

    ---@type ActionBinding
    RotatePitch = ActionBinding({
        hidden = true, -- not used by the game yet
        mouse = { Control.Combo(Button.MouseRight, Control.MouseDY()) },
    }),
}

return MapActions
