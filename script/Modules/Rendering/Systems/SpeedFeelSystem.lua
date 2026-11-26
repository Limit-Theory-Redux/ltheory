local Registry           = require("Core.ECS.Registry")
local CameraManager      = require("Modules.Cameras.Managers.CameraManager")
local ConstructComponents = require("Modules.Constructs.Components")
local PhysicsComponents  = require("Modules.Physics.Components")
local ShipFlightSystem   = require("Modules.Constructs.Systems.ShipFlightSystem")
local RenderCoreSystem   = require("Modules.Rendering.Systems.RenderCoreSystem")
local SpaceDustEntity    = require("Modules.Rendering.Entities.SpaceDustEntity")
local ShipActions        = require("Input.ActionBindings.ShipActions")

--- Speed feel: turns the player ship's speed, boost and travel drive into
--- effects on the view, one cue at a time, tuned in Config.render.speedFeel:
---   fov         field of view widens with speed, kicks on boost / drive engage
---   shake       camera shake: rumble while boosting / charging / in travel,
---               kicks on boost / drive engage / drive drop
---   radialBlur  radial blur from where the ship is heading
---   aberration  chromatic aberration while boosting and in travel
---   vignette    darker edges at speed
---   travelDrive the travel drive overlay (ring + star streaks)
---   dust        space dust streaking past (SpaceDustEntity)
---
--- The game enables it for every game (LTheoryRedux:setupScenario) and
--- disables it when the game ends (destroyScene). Which cues run and how
--- strong comes from Settings > Graphics > Speed Effects (GameState.render,
--- see SETTING_KEYS), read every frame; the Speed Feel Test scenario puts a
--- test override on top (`beginTest` / `endTest`) that its numpad keys
--- switch, so trying things never changes the player's settings. It writes
--- only to the camera manager (field of view offset, shake) and to
--- RenderCoreSystem.motionFx, and puts everything back on `disable()`.
---@class SpeedFeelSystem
local SpeedFeelSystem = Class("SpeedFeelSystem", function(self)
    self.cueNames = { "fov", "shake", "radialBlur", "aberration", "vignette", "travelDrive", "dust" }
    self.test = nil        -- the Speed Feel Test's override: { cues = { name = bool }, intensity }
    self.settingsCues = {} -- reused: the cues as Settings has them this frame
    self.dustWidth = 1.0   -- scales the dust's streak width (the scenario's keys)
    self.exitStyle = nil   -- leaving travel mode: "expand" / "collapse" (nil: the config's)
    self.game = nil
    self.subscription = nil
    self:resetState()
end)

---@private
function SpeedFeelSystem:resetState()
    self.fovSmooth = 0
    self.fovKick = 0
    self.lastBoosting = 0
    self.lastDriveState = "idle"
    self.exitT = nil       -- seconds since the travel drive dropped out (its exit effect), nil when none
    self.exitOverlay = 0   -- the overlay's strength when it dropped out
    -- What drives the cues, for the scenario's text
    self.drive = { speed = 0, maxSpeed = 1, flight = 0, boost = 0, boosting = 0, charge = 0,
        travel = 0, state = "idle", multiplier = 1 }
end

---Start driving the view from `game`'s player ship (the game state: its
---`playerShip` and `mapMode` are read every frame).
---@param game table
function SpeedFeelSystem:enable(game)
    self.game = game
    -- Subscribed once and never unsubscribed: Regenerate disables and enables
    -- within one frame, and an event the bus already queued for a removed
    -- subscription would call a handler that is gone. Disabled, the handler
    -- returns at once.
    if not self.subscription then
        self.subscription = EventBus:subscribe(Event.PreRender, self, self.onPreRender)
    end
end

---Stop, and put the view back as the settings have it.
function SpeedFeelSystem:disable()
    self:setDust(false)
    self:clearOutputs()
    self.game = nil
    self:resetState()
end

-- The Settings value (GameState.render) of each cue
local SETTING_KEYS = {
    fov = "speedFov", shake = "speedShake", radialBlur = "speedBlur",
    aberration = "speedAberration", vignette = "speedVignette",
    travelDrive = "speedTravelDrive", dust = "speedDust",
}

---Start trying cues without touching the settings: every cue on, strength
---1, until `endTest` (the Speed Feel Test scenario).
function SpeedFeelSystem:beginTest()
    local cues = {}
    for _, name in ipairs(self.cueNames) do cues[name] = true end
    self.test = { cues = cues, intensity = 1.0 }
    self.exitStyle = nil
    self.dustWidth = 1.0
end

---Back to the settings.
function SpeedFeelSystem:endTest()
    self.test = nil
    self.exitStyle = nil
    self.dustWidth = 1.0
end

---Switch a cue for the test (starts one if none is running).
---@param name string one of `cueNames`
---@param on boolean
function SpeedFeelSystem:setCue(name, on)
    assert(SETTING_KEYS[name], "SpeedFeelSystem: no cue " .. tostring(name))
    if not self.test then self:beginTest() end
    self.test.cues[name] = on
end

---@param name string
---@return boolean on in the test, or else in the settings
function SpeedFeelSystem:isCueOn(name)
    if self.test then return self.test.cues[name] == true end
    return GameState.render[SETTING_KEYS[name]] == true
end

---@param intensity number scales every cue for the test (1 = as configured)
function SpeedFeelSystem:setIntensity(intensity)
    if not self.test then self:beginTest() end
    self.test.intensity = math.max(0, intensity)
end

---@return number the test's strength, or else the settings'
function SpeedFeelSystem:getIntensity()
    if self.test then return self.test.intensity end
    return GameState.render.speedIntensity or 1.0
end

---@private
---@return table<string, boolean> the cues on this frame
function SpeedFeelSystem:currentCues()
    if self.test then return self.test.cues end
    local cues = self.settingsCues
    for name, key in pairs(SETTING_KEYS) do cues[name] = GameState.render[key] == true end
    return cues
end

---@param scale number the dust's streak width, x the configured
function SpeedFeelSystem:setDustWidth(scale)
    self.dustWidth = math.max(0.1, scale)
end

---@return number
function SpeedFeelSystem:getDustWidth()
    return self.dustWidth
end

---@param style string|nil leaving travel mode: "expand" (the ring and bubble
---grow and fade, a blink as it drops) or "collapse" (they shrink to a point,
---then a blink); nil = Config.render.speedFeel.travelExit.style
function SpeedFeelSystem:setExitStyle(style)
    self.exitStyle = style
end

---@return string
function SpeedFeelSystem:getExitStyle()
    return self.exitStyle or Config.render.speedFeel.travelExit.style
end

---What drives the cues this frame (see Config.render.speedFeel).
---@return table { speed, maxSpeed, flight, boost, boosting, charge, travel, state, multiplier }
function SpeedFeelSystem:getDrive()
    return self.drive
end

---@private
function SpeedFeelSystem:setDust(on)
    if on and not self.dustEntity then
        local cfg = Config.render.speedFeel.dust
        local layers = {}
        for i, layer in ipairs(cfg.layers) do
            layers[i] = {
                cell = layer.cell,
                count = layer.count,
                widthPx = layer.widthPx,
                color = Color(layer.color[1], layer.color[2], layer.color[3], 1),
            }
        end
        self.dustEntity, self.dust = SpaceDustEntity(1234, layers)
    elseif not on and self.dustEntity then
        if self.dustEntity:isValid() then Registry:destroyEntity(self.dustEntity) end
        self.dustEntity, self.dust = nil, nil
    end
end

---@private
function SpeedFeelSystem:clearOutputs()
    CameraManager:setFovOffset(0)
    CameraManager:getShake():reset()
    local fx = RenderCoreSystem.motionFx
    fx.aberration, fx.vignette, fx.radialBlur, fx.travelDrive = 0, 0, 0, 0
    fx.travelFlash, fx.blurClearRadius = 0, 0
end

local MIN_LENGTH = 1e-9           -- a speed or depth at or below this has no direction to show
local SCREEN_CENTER = 0.5         -- screen points are 0..1
local BOOST_PRESSED = 0.5         -- the boost input counts as pressed above this
local BLINK_TAIL = 4              -- a collapse's blink is over this many blink times after it
local DEFAULT_RING_RADIUS = 0.15  -- the ring's radius (share of the screen height) when behind the camera
local MAX_RING_RADIUS = 0.4       -- ... and its largest on screen
local OVERLAY_ENGAGED = 0.7       -- the overlay's strength as the drive engages
local OVERLAY_FULL_MULTIPLIER = 50 -- the drive multiplier at which the overlay is at full strength

-- A point on screen (0..1, y down) from a camera-relative direction or
-- position, the way the drawn view projects it, and its depth along the view;
-- nil when behind the camera.
local function project(x, y, z)
    local rot = CameraManager:getViewRot()
    local fwd, right, up = rot:getForward(), rot:getRight(), rot:getUp()
    local depth = x * fwd.x + y * fwd.y + z * fwd.z
    if depth <= MIN_LENGTH then return nil end
    local tanHalf = math.tan(math.rad(CameraManager:getFov()) / 2)
    local aspect = Window:width() / Window:height()
    local sx = (x * right.x + y * right.y + z * right.z) / (depth * tanHalf * aspect)
    local sy = (x * up.x + y * up.y + z * up.z) / (depth * tanHalf)
    return (sx + 1) * 0.5, (1 - sy) * 0.5, depth
end

local function clamp01(x)
    return math.max(0, math.min(1, x))
end

-- 0 below a, 1 above b, smooth in between (b < a fades the other way)
local function band(x, a, b)
    local t = clamp01((x - a) / (b - a))
    return t * t * (3 - 2 * t)
end

-- The travel overlay's strength while the drive is active: grows with its multiplier.
local function activeOverlay(multiplier)
    return OVERLAY_ENGAGED + (1 - OVERLAY_ENGAGED) * math.min(1, multiplier / OVERLAY_FULL_MULTIPLIER)
end

-- The travel overlay's strength by the drive's state, with the strength its
-- material was written with (Shared/Definitions/MaterialDefs.lua, TravelDrive)
---@param d table what drives the cues (`self.drive`)
local function driveOverlay(d)
    if d.state == "charging" then
        return d.charge
    elseif d.state == "active" then
        return activeOverlay(d.multiplier)
    elseif d.state == "decelerating" then
        return clamp01((d.multiplier - 1) / OVERLAY_FULL_MULTIPLIER)
    end
    return 0
end

---@private
---Read the ship: speed, boost, travel drive (fills `self.drive`).
---@return boolean ok false when there is no ship to read
function SpeedFeelSystem:readShip(ship)
    if not ship or not ship:isValid() then return false end
    local d = self.drive
    -- The ship's max speed, as the HUD's speed tape has it
    d.maxSpeed = ConstructComponents.FlightProfile.ProfileOf(ship).maxSpeed
    d.speed = ShipFlightSystem:getSpeed(ship)

    local driveCmp = ship:get(ConstructComponents.TravelDrive)
    d.state = driveCmp and driveCmp:getState() or "idle"
    d.multiplier = driveCmp and driveCmp:getCurrentMult() or 1
    d.charge = d.state == "charging" and clamp01(driveCmp:getChargeProgress()) or 0

    d.flight = clamp01(d.speed / d.maxSpeed)
    d.boost = clamp01((d.speed - d.maxSpeed) / (2 * d.maxSpeed))
    -- The boost key only counts while the drive is off (the flight system
    -- ignores it otherwise)
    d.boosting = d.state == "idle" and clamp01(ShipActions.Boost:get()) or 0
    local travelling = d.state == "active" or d.state == "decelerating"
    local cap = Config.game.travelDrive.maxSpeedByZone.openSpace
    d.travel = travelling and clamp01(math.log(math.max(1, d.multiplier)) / math.log(cap)) or 0
    return true
end

---@private
---Show no effect at all, and keep the dust (if any) hidden.
function SpeedFeelSystem:hideEffects()
    self:clearOutputs()
    if self.dust then
        for _, layer in ipairs(self.dust.layers) do layer.opacity = 0 end
    end
end

---@private
---The moments since the last frame (call once per frame, after `readShip`).
---@return boolean boostStarted
---@return boolean engaged the travel drive engaged
---@return boolean dropped the travel drive dropped out
function SpeedFeelSystem:readMoments()
    local d = self.drive
    local boostStarted = d.boosting > BOOST_PRESSED and self.lastBoosting <= BOOST_PRESSED
    local engaged = d.state == "active" and self.lastDriveState == "charging"
    local dropped = d.state == "decelerating" and self.lastDriveState == "active"
    self.lastBoosting, self.lastDriveState = d.boosting, d.state
    return boostStarted, engaged, dropped
end

---@private
---Leaving travel mode: the ring and the bubble expand and fade with a blink as
---the drive drops, or collapse to a point and then blink.
---@param tc table Config.render.speedFeel.travelExit
---@param dropped boolean the drive dropped out this frame
---@param dt number seconds since the last frame (0 while paused)
---@return number scale sizes the ring and the bubble
---@return number fade 1 -> 0, holds what travel mode showed
---@return number flash the blink
function SpeedFeelSystem:updateTravelExit(tc, dropped, dt)
    local d = self.drive
    if dropped then
        self.exitT = 0
        self.exitOverlay = activeOverlay(d.multiplier)
        self.exitTravel = d.travel
    elseif d.state == "charging" or d.state == "active" then
        self.exitT = nil
    end
    local t = self.exitT
    if not t then return 1, 0, 0 end

    local k = clamp01(t / tc.duration)
    local scale, fade, flash, done
    if self:getExitStyle() == "collapse" then
        scale = (1 - k) ^ 2
        fade = t < tc.duration and 1 or 0
        flash = t >= tc.duration and tc.blink * math.exp(-(t - tc.duration) / tc.blinkTime) or 0
        done = t > tc.duration + BLINK_TAIL * tc.blinkTime
    else
        scale = 1 + (tc.expandTo - 1) * (1 - (1 - k) ^ 2)
        fade = (1 - k) ^ 2
        flash = tc.blink * math.exp(-t / tc.blinkTime)
        done = t > tc.duration
    end
    self.exitT = (not done) and (t + dt) or nil
    return scale, fade, flash
end

---@private
---Field of view: follows the speed smoothly, plus short kicks.
---@param c table Config.render.speedFeel.fov
function SpeedFeelSystem:updateFov(c, on, intensity, boostStarted, engaged, dt)
    if not on then
        self.fovSmooth, self.fovKick = 0, 0
        CameraManager:setFovOffset(0)
        return
    end
    local d = self.drive
    local target = intensity * (c.flight * d.flight + c.boost * d.boost + c.travel * d.travel)
    self.fovSmooth = self.fovSmooth + (target - self.fovSmooth) * (1 - math.exp(-c.smoothing * dt))
    if boostStarted then self.fovKick = self.fovKick + intensity * c.boostKick end
    if engaged then self.fovKick = self.fovKick + intensity * c.travelKick end
    self.fovKick = self.fovKick * math.exp(-c.kickDecay * dt)
    CameraManager:setFovOffset(self.fovSmooth + self.fovKick)
end

---@private
---Camera shake: a steady rumble, plus a jolt at each moment.
---@param c table Config.render.speedFeel.shake
function SpeedFeelSystem:updateShake(c, on, intensity, boostStarted, engaged, dropped)
    local shake = CameraManager:getShake()
    if not on then
        shake:reset()
        return
    end
    local d = self.drive
    local drivingRumble = d.state == "active" and c.travelRumble or 0
    shake:setRumble(intensity * (c.boostRumble * d.boosting + c.chargeRumble * d.charge + drivingRumble))
    if boostStarted then shake:addTrauma(intensity * c.boostKick) end
    if engaged then shake:addTrauma(intensity * c.travelKick) end
    if dropped then shake:addTrauma(intensity * c.dropKick) end
end

---@private
---Where the ship is heading and where the effects come from, on screen (0..1,
---y down). The effects come from the ship on screen in third person; in first
---person (the camera is in it), or with the ship behind the camera, from where
---it is heading.
---@param rb RigidBody the ship's
---@param v Vec3f the ship's velocity relative to what carries it
---@param firstPerson boolean
---@return number headU
---@return number headV
---@return number srcU
---@return number srcV
function SpeedFeelSystem:locateSource(rb, v, firstPerson)
    local headU, headV = SCREEN_CENTER, SCREEN_CENTER
    if self.drive.speed > MIN_LENGTH then
        local u, w = project(v.x, v.y, v.z)
        if u then headU, headV = clamp01(u), clamp01(w) end
    end

    local srcU, srcV = headU, headV
    if not firstPerson then
        local eye = CameraManager:getEye()
        local shipPos = rb:getPos()
        local u, w = project(shipPos.x - eye.x, shipPos.y - eye.y, shipPos.z - eye.z)
        if u then srcU, srcV = clamp01(u), clamp01(w) end
    end
    return headU, headV, srcU, srcV
end

---@private
---The travel drive's ring: a ring `size` ship radii wide, `ahead` ship radii
---in front of the nose (in first person: of the camera, which sits above the
---ship's centre), projected. Also sets how far the ring is from the camera.
---@param c table Config.render.speedFeel.travelDrive
---@param ship Entity
---@param rb RigidBody the ship's
---@param fp table|nil Config.render.speedFeel.firstPerson, in first person
---@param srcU number where the ring is when it is behind the camera
---@param srcV number
---@return number ringU on screen (0..1, y down)
---@return number ringV
---@return number ringR radius, share of the screen height
function SpeedFeelSystem:locateRing(c, ship, rb, fp, srcU, srcV)
    local shipRadius = ship:get(PhysicsComponents.RigidBody):getRadius()
    local fwd = rb:getRot():getForward()
    local ahead = c.ahead * shipRadius
    local x, y, z = fwd.x * ahead, fwd.y * ahead, fwd.z * ahead
    if not fp then
        local eye = CameraManager:getEye()
        local p = rb:getPos()
        x, y, z = x + p.x - eye.x, y + p.y - eye.y, z + p.z - eye.z
    end
    local u, w, depth = project(x, y, z)
    RenderCoreSystem.motionFx.ringDistance = math.sqrt(x * x + y * y + z * z)
    if not u then return srcU, srcV, DEFAULT_RING_RADIUS end

    local tanHalf = math.tan(math.rad(CameraManager:getFov()) / 2)
    local ringR = Math.Clamp(c.size * shipRadius / (2 * depth * tanHalf), c.minRadius, MAX_RING_RADIUS)
    return u, w, ringR * (fp and fp.ring or 1)
end

---@private
---The post filters: radial blur, chromatic aberration and vignette.
---@param cfg table Config.render.speedFeel
---@param cues table<string, boolean>
---@param fp table|nil Config.render.speedFeel.firstPerson, in first person
function SpeedFeelSystem:updatePostFx(cfg, cues, intensity, fp)
    local fx = RenderCoreSystem.motionFx
    local d = self.drive
    fx.radialBlur = cues.radialBlur
        and intensity * (cfg.radialBlur.boost * d.boost + cfg.radialBlur.travel * d.travel)
        * (fp and fp.radialBlur or 1) or 0
    fx.aberration = cues.aberration
        and intensity * (cfg.aberration.boosting * d.boosting + cfg.aberration.travel * d.travel) or 0
    fx.vignette = cues.vignette
        and intensity * (cfg.vignette.boost * d.boost + cfg.vignette.travel * d.travel) or 0
end

---@private
---The travel drive overlay (the ring and its streaks) and the exit's blink.
---@param c table Config.render.speedFeel.travelDrive
---@param fp table|nil Config.render.speedFeel.firstPerson, in first person
function SpeedFeelSystem:updateOverlay(c, on, intensity, fp, exitFade, flash, ringU, ringV, ringR)
    local fx = RenderCoreSystem.motionFx
    local overlay = 0
    if on then
        overlay = math.max(driveOverlay(self.drive), exitFade * self.exitOverlay)
            * (fp and fp.travelDrive or 1)
        fx.ringX, fx.ringY, fx.ringRadius = ringU, ringV, ringR
    end
    fx.travelDrive = intensity * c.gain * overlay
    fx.travelFlash = on and intensity * flash or 0
    fx.driveSpeed = self.drive.multiplier
end

---@private
---@param e EventData
function SpeedFeelSystem:onPreRender(e)
    local game = self.game
    if not game then return end -- disabled: `disable()` put everything back
    local cfg = Config.render.speedFeel
    local intensity = self:getIntensity()
    local cues = self:currentCues()
    self:setDust(cues.dust)

    -- Nothing in a map, or without a ship
    if (game.mapMode or 0) ~= 0 or not self:readShip(game.playerShip) then
        self:hideEffects()
        return
    end
    local d = self.drive
    local dt = e:deltaTime()
    local paused = GameState.paused
    local unpausedDt = paused and 0 or dt

    local boostStarted, engaged, dropped = self:readMoments()
    local exitScale, exitFade, flash = self:updateTravelExit(cfg.travelExit, dropped, unpausedDt)
    -- What the dust and the bubble show of travel mode: fading out on exit
    local travelShown = math.max(d.travel, exitFade * (self.exitTravel or 0))

    -- Both before anything is projected: the projection reads the field of view
    self:updateFov(cfg.fov, cues.fov, intensity, boostStarted, engaged, dt)
    -- No rumble while paused (the clock it wobbles on keeps running)
    self:updateShake(cfg.shake, cues.shake and not paused, intensity, boostStarted, engaged, dropped)

    local rb = game.playerShip:get(PhysicsComponents.RigidBody):getRigidBody()
    -- Relative to what carries the ship, like `d.speed`; the rest is the
    -- carry, which the dust moves along with
    local v = Vec3f(ShipFlightSystem:getRelativeVelocity(game.playerShip))
    local carried = rb:getVelocity() - v
    -- First person: the camera is in the ship, and the middle of the view is
    -- where you look, so the overlay and the blur ease off there
    local firstPerson = game.player ~= nil and game.player:getMode() == Enums.CameraMode.FirstPerson
    local fp = firstPerson and cfg.firstPerson or nil

    -- The radial blur, the overlay's ring and streaks and the dust (see
    -- updateDust) all come from the source point
    local headU, headV, srcU, srcV = self:locateSource(rb, v, firstPerson)
    local ringU, ringV, ringR = self:locateRing(cfg.travelDrive, game.playerShip, rb, fp, srcU, srcV)
    ringR = ringR * exitScale

    self:updatePostFx(cfg, cues, intensity, fp)
    local fx = RenderCoreSystem.motionFx
    fx.blurCenterX, fx.blurCenterY = srcU, srcV
    -- The blur leaves the bubble sharp, so it doesn't smear streaks into it
    fx.blurClearX, fx.blurClearY = ringU, ringV
    fx.blurClearRadius = travelShown > 0 and cfg.dust.travel.bubbleRadius * ringR or 0

    self:updateOverlay(cfg.travelDrive, cues.travelDrive, intensity, fp, exitFade, flash,
        ringU, ringV, ringR)

    if self.dust then
        self:updateDust(cfg.dust, d, travelShown, v, carried, headU, headV, srcU, srcV, ringU, ringV,
            ringR, unpausedDt)
    end
end

---@private
---The dust is at rest in what carries the ship: its offset follows the carry.
---The ship flies through it at up to each layer's `maxSpeed`; above that
---(travel mode) it falls behind (the offset grows further), so specks stay
---visible, and travel mode draws long, bright streaks that the drive's bubble
---pushes aside around the ship.
---
---By perspective, streaks converge on where the ship is heading. In third
---person that isn't where the ship is, so from `fromShip[1]` to `fromShip[2]`
---times max speed the dust is moved on screen until they converge on the
---ship (slower, it stays true to the world).
---@param dc table Config.render.speedFeel.dust
---@param d table what drives the cues (`self.drive`)
---@param travel number what to show of travel mode, 0..1 (d.travel, fading out on exit)
---@param v Vec3f the ship's velocity relative to what carries it, `d.speed` long
---@param carried Vec3f the velocity of what carries the ship (zero in open space)
---@param headU number where the ship is heading on screen (0..1, y down)
---@param headV number
---@param srcU number where the effects come from: the ship, or where it heads
---@param srcV number
---@param ringU number the travel drive's ring on screen: the bubble wraps it
---@param ringV number
---@param ringR number the ring's radius, share of the screen height
---@param dt number seconds since the last frame (0 while paused)
function SpeedFeelSystem:updateDust(dc, d, travel, v, carried, headU, headV, srcU, srcV, ringU, ringV, ringR, dt)
    local tc = dc.travel
    local speed = d.speed
    local dirX, dirY, dirZ = 0, 0, 0
    if speed > 1e-9 then dirX, dirY, dirZ = v.x / speed, v.y / speed, v.z / speed end
    local rel = speed / d.maxSpeed
    local brightness = self:getIntensity() * (1 + (tc.brightness - 1) * travel)
    local widthScale = (1 + (tc.width - 1) * travel) * self.dustWidth

    for i, layer in ipairs(self.dust.layers) do
        local lc = dc.layers[i]
        local dustSpeed = math.min(speed, lc.maxSpeed * d.maxSpeed)
        local lag = (speed - dustSpeed) * dt
        layer.offsetX = layer.offsetX + carried.x * dt + dirX * lag
        layer.offsetY = layer.offsetY + carried.y * dt + dirY * lag
        layer.offsetZ = layer.offsetZ + carried.z * dt + dirZ * lag

        -- A streak shows `streakTime` of the dust's motion; travel mode
        -- stretches it towards `streakCells` of the cube
        local length = math.max(dustSpeed * dc.streakTime, travel * tc.streakCells * lc.cell)
        length = math.min(length, 0.8 * lc.cell)
        layer.streakX, layer.streakY, layer.streakZ = dirX * length, dirY * length, dirZ * length

        layer.opacity = lc.fadeIn and band(rel, lc.fadeIn[1], lc.fadeIn[2]) or 1
        layer.brightness = brightness
        layer.widthPx = lc.widthPx * widthScale
    end

    local bubble = self.dust.bubble
    bubble.radius = tc.bubbleRadius * ringR
    bubble.curl = math.rad(tc.curl)
    bubble.strength = clamp01(1.5 * travel)
    bubble.x, bubble.y = 2 * ringU - 1, 1 - 2 * ringV
    -- In NDC (y up): from where they converge to where they should
    local k = band(rel, dc.fromShip[1], dc.fromShip[2])
    bubble.shiftX, bubble.shiftY = 2 * (srcU - headU) * k, -2 * (srcV - headV) * k
end

return SpeedFeelSystem()
