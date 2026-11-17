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
        flight     = 6,
        boost      = 16,
        travel     = 20,
        boostKick  = 8,   -- a short push when boost starts
        travelKick = 14,  -- ... when the travel drive engages
        kickDecay  = 3.0, -- 1/s, how fast a kick dies away
        smoothing  = 4.0, -- 1/s, how fast the offset follows the speed
    },
    -- Camera shake (CameraShake): steady rumble 0..1, kicks add trauma 0..1.
    -- The shake moves with the square of these.
    shake = {
        boostRumble  = 0.6, -- while boosting
        chargeRumble = 0.7, -- at the end of the charge (grows with it)
        travelRumble = 0.4, -- while the travel drive is active
        boostKick    = 0.5,
        travelKick   = 1.0, -- when the drive engages
        dropKick     = 0.7, -- when it drops out
    },
    -- Radial blur from where the ship is heading on screen: added to the post
    -- setting's strength (capped at 1), and it lengthens the streaks
    -- (RenderCoreSystem, MOTION_BLUR_REACH)
    radialBlur = {
        boost  = 0.5,
        travel = 0.9,
    },
    -- Chromatic aberration: added to the post setting's strength (capped at
    -- 1), and it widens the colour split towards the edges
    aberration = {
        boosting = 0.8,
        travel   = 0.8,
    },
    -- A round vignette on top of the post setting's (0..1, 1 = black edges)
    vignette = {
        boost  = 0.25,
        travel = 0.45,
    },
    -- The travel drive overlay (filter/traveldrive.glsl): ring + star streaks
    travelDrive = {
        gain = 0.7,
    },
    -- First person: the camera is in the ship and the middle of the view is
    -- where you look; these multiply the travel overlay and the radial blur
    firstPerson = {
        travelDrive = 0.4,
        radialBlur  = 0.6,
    },
    -- Space dust (SpaceDustEntity). Speeds in multiples of the ship's max
    -- speed; a layer with `fadeIn` fades in between those two speeds.
    dust = {
        streakTime = 0.05, -- seconds of motion a streak shows
        layers = {
            -- Close specks, from rest through boost and into travel mode
            -- (cube 3 km wide); they move at up to 5x max speed
            { cell = 0.3, count = 2000, widthPx = 2.0, color = { 0.75, 0.8, 0.9 },
              maxSpeed = 5 },
            -- Far specks that join when the travel drive takes off (30 km)
            { cell = 3.0, count = 1500, widthPx = 2.2, color = { 0.8, 0.85, 0.95 },
              maxSpeed = 25, fadeIn = { 3.5, 6 } },
        },
        -- Travel mode: long, bright streaks, pushed aside around the ship by
        -- the drive (all grow with `travel`)
        travel = {
            bubbleRadius = 0.2, -- on screen, fraction of its height (the overlay's ring: 0.15)
            curl         = 40,  -- degrees the bubble turns the dust around the ship (0: straight out)
            streakCells  = 0.6, -- streak length, in cubes
            brightness   = 1.8, -- x the colour
            width        = 1.4, -- x the width
        },
    },
}
