-- StationTest: a shortcut into the game's Station Showcase scenario
-- (script/Config/Scenarios/StationShowcase.lua): a generated station next to your ship. B builds a new one.
--
-- Starts the game (LTheoryRedux) straight into the scenario, skipping the main
-- menu; after that it is the normal game (C cycles the cameras, Escape opens
-- the pause menu, Back to Main Menu leads to the menu).
--
-- Run: ./bin/ltr.exe StationTest

local Game = require('States.App.LTheoryRedux')

Game.autostartScenario = require('Config.Scenarios').find('StationShowcase')
Game.autostartSeed = 1 -- the same scene every run (B for another)

return Game
