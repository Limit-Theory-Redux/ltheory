-- PlanetTest: a shortcut into the game's Planet Showcase scenario
-- (script/Config/Scenarios/PlanetShowcase.lua): one planet with moons and sometimes a ring. B makes a new one.
-- Under LTHEORY_CAPTURE, LTHEORY_CAPTURE_SEED picks the planet (render validation:
-- PlanetTestRing uses seed 1, which rolls a ring).
--
-- Starts the game (LTheoryRedux) straight into the scenario, skipping the main
-- menu; after that it is the normal game (C cycles the cameras, Escape opens
-- the pause menu, Back to Main Menu leads to the menu).
--
-- Run: ./bin/ltr.exe PlanetTest

local Game = require('States.App.LTheoryRedux')

Game.autostartScenario = require('Config.Scenarios').find('PlanetShowcase')
Game.autostartSeed = 27 -- the same planet every run, one without a ring (B for another)

return Game
