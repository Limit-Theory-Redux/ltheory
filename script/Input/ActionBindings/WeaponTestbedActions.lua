-- Input bindings for the Weapon Testbed scenario (Config/Scenarios/WeaponTestbed).
-- F6 toggle the capital's AI, F7 cycle the fire mode (volley / sequence) and
-- F9 hold the trigger while the AI is off, R reset the drones. The game's own
-- keys stay free: A strafes, P pauses, the left mouse button fires the
-- player's ship, F8 toggles the sound.
local ActionBinding = require("Input.ActionBinding")
local Control = require("Input.Control")

return {
    AI = ActionBinding({
        label = "Toggle capital AI", group = "Weapon Testbed",
        keyboard = { Control.Single(Button.KeyboardF6) },
    }),
    CycleMode = ActionBinding({
        label = "Cycle fire mode (AI off)", group = "Weapon Testbed",
        keyboard = { Control.Single(Button.KeyboardF7) },
    }),
    Fire = ActionBinding({
        label = "Hold trigger (AI off)", group = "Weapon Testbed",
        keyboard = { Control.Single(Button.KeyboardF9) },
    }),
    Reset = ActionBinding({
        label = "Reset drones", group = "Weapon Testbed",
        keyboard = { Control.Single(Button.KeyboardR) },
    }),
}
