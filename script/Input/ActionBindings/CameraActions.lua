local ActionBinding = require('Input.ActionBinding')
local Control = require('Input.Control')

local CameraActions = {
    ---@type ActionBinding
    Yaw = ActionBinding({
        label = "Look left / right (orbit, free camera)", group = "Camera",
        mouse = { Control.Combo(Button.MouseRight, Control.MouseDX()) },
        gamepad = {
            Control.GamepadAxis(Button.GamepadRightStickX)
                :setMult(3):setExpn(2)
        }
    }),
    
    ---@type ActionBinding
    Pitch = ActionBinding({
        label = "Look up / down (orbit, free camera)", group = "Camera",
        mouse = { Control.Combo(Button.MouseRight, Control.MouseDY()) },
        gamepad = {
            Control.GamepadAxis(Button.GamepadRightStickY)
                :setMult(3):setExpn(2):invert()
        }
    }),
    
    ---@type ActionBinding
    Zoom = ActionBinding({
        label = "Zoom (orbit camera)", group = "Camera",
        mouse = { Control.MouseWheel() }
    }),

    ---@type ActionBinding
    TranslateX = ActionBinding({
        label = "Move right / left (free camera)", group = "Camera",
        keyboard = { Control.Pair(Button.KeyboardD, Button.KeyboardA) }
    }),

    ---@type ActionBinding
    TranslateY = ActionBinding({
        label = "Move up / down (free camera)", group = "Camera",
        keyboard = { Control.Pair(Button.KeyboardSpace, Button.KeyboardControlLeft) }
    }),

    ---@type ActionBinding
    TranslateZ = ActionBinding({
        label = "Move forward / back (free camera)", group = "Camera",
        keyboard = { Control.Pair(Button.KeyboardW, Button.KeyboardS) }
    }),

    ---@type ActionBinding
    Roll = ActionBinding({
        label = "Roll right / left (free camera)", group = "Camera",
        keyboard = { Control.Pair(Button.KeyboardE, Button.KeyboardQ) },
    }),

    ---@type ActionBinding
    FastMove = ActionBinding({
        label = "Move faster (free camera)", group = "Camera",
        keyboard = { Control.Single(Button.KeyboardShiftLeft) },
    }),

    ---@type ActionBinding
    SlowMove = ActionBinding({
        label = "Move slower (free camera)", group = "Camera",
        keyboard = { Control.Single(Button.KeyboardAltLeft) },
    }),

    ---@type ActionBinding
    MouseCapture = ActionBinding({
        label = "Capture mouse (free camera)", group = "Camera",
        mouse = { Control.Single(Button.MouseRight) },
    }),
}

return CameraActions
