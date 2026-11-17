-- Explosion testbed: starts the game straight into the Explosion Showcase
-- scenario (Config/Scenarios/ExplosionShowcase.lua), one volumetric
-- explosion next to the player's ship to look at and tune.
--
-- Launch: ./bin/ltr.exe Testbeds/Explosion
--         (or: cargo run -- Testbeds/Explosion)
-- Camera: the orbit camera circles the explosion; C switches to flying your
--         own ship (and back round to the orbit camera).
-- Keys:   B new explosion | numpad Enter loop on/off | numpad 5 next size
--         class | numpad + - size | numpad * / duration | numpad . freeze
--         numpad 4 6 (held) scrub | numpad 0 clear | F5 reload shaders
--         Escape pause menu
local Game = require('States.App.LTheoryRedux')

Game.autostartScenario = require('Config.Scenarios').find('ExplosionShowcase')
Game.autostartSeed = 1 -- the same scene every run (B for another)

return Game
