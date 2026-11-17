-- SolarSystemPlayable: a shortcut into the game's Standard Solar System scenario
-- (script/Config/Scenarios/StandardSolarSystem.lua): several planets, moons and rings, different per seed.
--
-- Starts the game (LTheoryRedux) straight into the scenario, skipping the main
-- menu; after that it is the normal game (C cycles the cameras, Escape opens
-- the pause menu, Back to Main Menu leads to the menu).
--
-- Run: ./bin/ltr.exe SolarSystemPlayable

local Game = require('States.App.LTheoryRedux')

Game.autostartScenario = require('Config.Scenarios').find('StandardSolarSystem')
Game.autostartSeed = 1 -- the same scene every run (B for another)

return Game
