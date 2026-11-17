-- Speed feel: what flying, boosting and travel mode do to the view
-- (SpeedFeelSystem). Every cue is off in the game; the Speed Feel Test
-- scenario turns them on one by one to try them. All values are at intensity
-- 1 (the scenario's + and - scale them together).
--
-- What drives the cues (see SpeedFeelSystem):
--   flight  0..1  speed from 0 to the ship's max speed
--   boost   0..1  speed from max to boost speed (3x max)
--   boosting 0..1 the boost key, while the travel drive is off
--   charge  0..1  the travel drive charging
--   travel  0..1  the travel drive's multiplier on a log scale, 1 at the
--                 open-space cap (Config.game.travelDrive.maxSpeedByZone)
Config.render.speedFeel = {
    -- Field of view, degrees added to the setting
    fov = {
        flight     = 3,
        boost      = 8,
        travel     = 14,
        boostKick  = 4,   -- a short push when boost starts
        travelKick = 10,  -- ... when the travel drive engages
        kickDecay  = 3.0, -- 1/s, how fast a kick dies away
        smoothing  = 4.0, -- 1/s, how fast the offset follows the speed
    },
    -- Camera shake (CameraShake): steady rumble 0..1, kicks add trauma 0..1
    shake = {
        boostRumble  = 0.45, -- while boosting
        chargeRumble = 0.5,  -- at the end of the charge (grows with it)
        travelRumble = 0.3,  -- while the travel drive is active
        boostKick    = 0.35,
        travelKick   = 0.8,  -- when the drive engages
        dropKick     = 0.5,  -- when it drops out
    },
    -- Radial blur, added to the post setting (0..1); streaks from where the
    -- ship is heading on screen
    radialBlur = {
        boost  = 0.25,
        travel = 0.7,
    },
    -- Chromatic aberration, added to the post setting
    aberration = {
        boosting = 0.4,
        travel   = 0.6,
    },
    -- Vignette, added to the post setting
    vignette = {
        boost  = 0.1,
        travel = 0.25,
    },
    -- The travel drive overlay (filter/traveldrive.glsl): ring + star streaks
    travelDrive = {
        gain = 0.5,
    },
    -- Space dust (SpaceDustEntity). Speeds are in multiples of the ship's max
    -- speed: a layer fades in between fadeIn[1] and fadeIn[2] and out between
    -- fadeOut[1] and fadeOut[2] (fadeIn nil: always in from rest).
    dust = {
        streakTime = 0.05, -- seconds of motion a streak shows
        layers = {
            -- Close specks for normal flight (cube 3 km wide)
            { cell = 0.3, count = 1500, widthPx = 1.5, color = { 0.45, 0.5, 0.6 },
              fadeOut = { 1.5, 3.0 } },
            -- Far specks for boost and the start of travel mode (30 km)
            { cell = 3.0, count = 1500, widthPx = 1.8, color = { 0.5, 0.55, 0.65 },
              fadeIn = { 1.0, 2.0 }, fadeOut = { 10, 40 } },
        },
    },
}
