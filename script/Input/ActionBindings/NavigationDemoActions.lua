local ActionBinding = require('Input.ActionBinding')
local Control = require('Input.Control')

--- Actions of the navigation demo: pick the scenario and the ship the camera follows
local NavigationDemoActions = {
    ---@type ActionBinding
    PrevScenario = ActionBinding({
        label = "Previous scenario", group = "Navigation demo",
        keyboard = { Control.Single(Button.KeyboardLeft) },
    }),

    ---@type ActionBinding
    NextScenario = ActionBinding({
        label = "Next scenario", group = "Navigation demo",
        keyboard = { Control.Single(Button.KeyboardRight) },
    }),

    ---@type ActionBinding
    PrevShip = ActionBinding({
        label = "Previous ship", group = "Navigation demo",
        keyboard = { Control.Single(Button.KeyboardUp) },
    }),

    ---@type ActionBinding
    NextShip = ActionBinding({
        label = "Next ship", group = "Navigation demo",
        keyboard = { Control.Single(Button.KeyboardDown) },
    }),
}

return NavigationDemoActions
