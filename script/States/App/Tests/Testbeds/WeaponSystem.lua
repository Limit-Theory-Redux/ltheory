-- WeaponSystem testbed: starts the game straight into the Weapon Testbed
-- scenario (Config/Scenarios/WeaponTestbed.lua), an armed capital ship
-- fighting target drones next to the player's ship in the normal game. The
-- weapons run in the game loop (WeaponCombatSystem), like any armed ship's.
--
-- Launch: ./bin/ltr.exe Testbeds/WeaponSystem
--         (or: cargo run -- Testbeds/WeaponSystem)
-- Camera: the orbit camera follows the capital; C switches to flying your
--         own ship (and back round to the orbit camera).
-- Keys:   F6 capital AI on/off | F7 fire mode, volley / sequence (AI off)
--         F9 hold the trigger (AI off) | R reset the drones
--         B regenerate (resets the drones too) | Escape pause menu
local Game = require('States.App.LTheoryRedux')

Game.autostartScenario = require('Config.Scenarios').find('WeaponTestbed')
Game.autostartSeed = 1 -- the same scene every run (B for another)

return Game
