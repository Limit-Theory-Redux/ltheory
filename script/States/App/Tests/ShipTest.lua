-- ShipTest: a shortcut into the game's Ship Showcase scenario
-- (script/Config/Scenarios/ShipShowcase.lua): your ship up close. B builds the next hull (fighter, basic,
-- capital).
--
-- Starts the game (LTheoryRedux) straight into the scenario, skipping the main
-- menu; after that it is the normal game (C cycles the cameras, Escape opens
-- the pause menu, Back to Main Menu leads to the menu).
--
-- Run: ./bin/ltr.exe ShipTest

local Game = require('States.App.LTheoryRedux')

Game.autostartScenario = require('Config.Scenarios').find('ShipShowcase')
Game.autostartSeed = 1 -- the same scene every run (B for another)

return Game
