local ActionBinding = require('Input.ActionBinding')
local Control = require('Input.Control')

local CameraActions = {
    ---@type ActionBinding
    Yaw = ActionBinding({
        mouse = { Control.Combo(Button.MouseRight, Control.MouseDX()) },
        gamepad = {
            Control.GamepadAxis(Button.GamepadRightStickX)
                :setMult(3):setExpn(2)
        }
    }),
    
    ---@type ActionBinding
    Pitch = ActionBinding({
        mouse = { Control.Combo(Button.MouseRight, Control.MouseDY()) },
        gamepad = {
            Control.GamepadAxis(Button.GamepadRightStickY)
                :setMult(3):setExpn(2):invert()
        }
    }),
    
    ---@type ActionBinding
    Zoom = ActionBinding({
        mouse = { Control.MouseWheel() }
    }),

    ---@type ActionBinding
    TranslateX = ActionBinding({
        keyboard = { Control.Pair(Button.KeyboardD, Button.KeyboardA) }
    }),

    ---@type ActionBinding
    TranslateY = ActionBinding({
        keyboard = { Control.Pair(Button.KeyboardSpace, Button.KeyboardControlLeft) }
    }),

    ---@type ActionBinding
    TranslateZ = ActionBinding({
        keyboard = { Control.Pair(Button.KeyboardW, Button.KeyboardS) }
    }),
}

return CameraActions
