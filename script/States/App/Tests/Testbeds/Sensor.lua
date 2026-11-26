-- Sensor testbed: starts the game straight into the Sensor Testbed scenario
-- (Config/Scenarios/SensorTestbed.lua): the player's ship with passive and
-- active sensors, and setups around it that each show one thing about them.
--
-- Launch: ./bin/ltr.exe Testbeds/Sensor
--         (or: cargo run -- Testbeds/Sensor)
-- Keys:   X (hold) scan the target | Z pulse | Tab next target
--         1 2 3 hull (fighter, frigate, capital ship) | 4 surroundings
--         5 the target's activity | 6 resolution rule | 7 merging on/off
--         8 tuning panel (Up/Down dial, Left/Right change, Backspace hull values)
--         B next setup | R this setup again | Escape pause menu
local Game = require('States.App.LTheoryRedux')

Game.autostartScenario = require('Config.Scenarios').find('SensorTestbed')
Game.autostartSeed = 1 -- the same star system every run

return Game
