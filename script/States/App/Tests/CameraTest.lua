-- CameraTest: a shortcut into the game's Camera Test scenario
-- (script/Config/Scenarios/CameraTest.lua): F1 free, F2 orbit, F3 first person, F4 RTS camera, - and =
-- change the field of view.
--
-- Starts the game (LTheoryRedux) straight into the scenario, skipping the main
-- menu; after that it is the normal game (C cycles the cameras, Escape opens
-- the pause menu, Back to Main Menu leads to the menu).
--
-- Run: ./bin/ltr.exe CameraTest

local Game = require('States.App.LTheoryRedux')

Game.autostartScenario = require('Config.Scenarios').find('CameraTest')
Game.autostartSeed = 1 -- the same scene every run (B for another)

return Game
