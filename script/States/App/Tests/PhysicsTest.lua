-- PhysicsTest: a shortcut into the game's Physics Test scenario
-- (script/Config/Scenarios/PhysicsTest.lua): colliders, compound bodies and triggers next to your ship,
-- drawn as wireframes; its keys are on the numpad (see the scenario file).
--
-- Starts the game (LTheoryRedux) straight into the scenario, skipping the main
-- menu; after that it is the normal game (C cycles the cameras, Escape opens
-- the pause menu, Back to Main Menu leads to the menu).
--
-- Run: ./bin/ltr.exe PhysicsTest

local Game = require('States.App.LTheoryRedux')

Game.autostartScenario = require('Config.Scenarios').find('PhysicsTest')
Game.autostartSeed = 1 -- the same scene every run (B for another)

return Game
