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

-- A point on screen (0..1, y down) from a camera-relative direction or
-- position, the way the drawn view projects it, and its depth along the view;
-- nil when behind the camera.
local function project(x, y, z)
    local rot = CameraManager:getViewRot()
    local fwd, right, up = rot:getForward(), rot:getRight(), rot:getUp()
    local depth = x * fwd.x + y * fwd.y + z * fwd.z
    if depth <= 1e-9 then return nil end
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

---@private
---Read the ship: speed, boost, travel drive (fills `self.drive`).
---@return boolean ok false when there is no ship to read
function SpeedFeelSystem:readShip(ship)
    if not ship or not ship:isValid() then return false end
    local d = self.drive
    -- The ship's max speed, as the HUD's speed tape has it: the ship's flight
    -- profile where there is one (the navigation work replaces
    -- Config.game.shipFlight with per-ship profiles), else the config
    local profile = ConstructComponents.FlightProfile
    d.maxSpeed = profile and profile.ProfileOf(ship).maxSpeed or Config.game.shipFlight.maxSpeedGame
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
---@param e EventData
function SpeedFeelSystem:onPreRender(e)
    local game = self.game
    if not game then return end -- disabled: `disable()` put everything back
    local dt = e:deltaTime()
    local cfg = Config.render.speedFeel
    local I = self:getIntensity()
    local cues = self:currentCues()
    self:setDust(cues.dust)
    local fx = RenderCoreSystem.motionFx
    local shake = CameraManager:getShake()
    local d = self.drive

    -- Nothing in a map, or without a ship
    if (game.mapMode or 0) ~= 0 or not self:readShip(game.playerShip) then
        self:clearOutputs()
        if self.dust then
            for _, layer in ipairs(self.dust.layers) do layer.opacity = 0 end
        end
        return
    end
    local paused = GameState.paused

    -- Moments: boost starts, the drive engages, the drive drops out
    local boostStarted = d.boosting > 0.5 and self.lastBoosting <= 0.5
    local engaged = d.state == "active" and self.lastDriveState == "charging"
    local dropped = d.state == "decelerating" and self.lastDriveState == "active"
    self.lastBoosting, self.lastDriveState = d.boosting, d.state

    -- Leaving travel mode (Config.render.speedFeel.travelExit): the ring and
    -- the bubble expand and fade with a blink as the drive drops, or collapse
    -- to a point and then blink. exitScale sizes them, exitFade (1 -> 0)
    -- holds what travel mode showed, flash is the blink.
    if dropped then
        self.exitT = 0
        self.exitOverlay = 0.7 + 0.3 * math.min(1, d.multiplier / 50)
        self.exitTravel = d.travel
    elseif d.state == "charging" or d.state == "active" then
        self.exitT = nil
    end
    local exitScale, exitFade, flash = 1, 0, 0
    if self.exitT then
        local e = cfg.travelExit
        local t = self.exitT
        local k = clamp01(t / e.duration)
        local done
        if self:getExitStyle() == "collapse" then
            exitScale = (1 - k) ^ 2
            exitFade = t < e.duration and 1 or 0
            flash = t >= e.duration and e.blink * math.exp(-(t - e.duration) / e.blinkTime) or 0
            done = t > e.duration + 4 * e.blinkTime
        else
            exitScale = 1 + (e.expandTo - 1) * (1 - (1 - k) ^ 2)
            exitFade = (1 - k) ^ 2
            flash = e.blink * math.exp(-t / e.blinkTime)
            done = t > e.duration
        end
        self.exitT = (not done) and (t + (paused and 0 or dt)) or nil
    end
    -- What the dust and the bubble show of travel mode: fading out on exit
    local travelShown = math.max(d.travel, exitFade * (self.exitTravel or 0))

    -- Field of view: follows the speed smoothly, plus short kicks
    if cues.fov then
        local c = cfg.fov
        local target = I * (c.flight * d.flight + c.boost * d.boost + c.travel * d.travel)
        self.fovSmooth = self.fovSmooth + (target - self.fovSmooth) * (1 - math.exp(-c.smoothing * dt))
        if boostStarted then self.fovKick = self.fovKick + I * c.boostKick end
        if engaged then self.fovKick = self.fovKick + I * c.travelKick end
        self.fovKick = self.fovKick * math.exp(-c.kickDecay * dt)
        CameraManager:setFovOffset(self.fovSmooth + self.fovKick)
    else
        self.fovSmooth, self.fovKick = 0, 0
        CameraManager:setFovOffset(0)
    end

    -- Shake: no rumble while paused (the clock it wobbles on keeps running)
    if cues.shake and not paused then
        local c = cfg.shake
        local drivingRumble = d.state == "active" and c.travelRumble or 0
        shake:setRumble(I * (c.boostRumble * d.boosting + c.chargeRumble * d.charge + drivingRumble))
        if boostStarted then shake:addTrauma(I * c.boostKick) end
        if engaged then shake:addTrauma(I * c.travelKick) end
        if dropped then shake:addTrauma(I * c.dropKick) end
    else
        shake:reset()
    end

    -- Where the ship is heading, on screen (0..1, y down)
    local rb = game.playerShip:get(PhysicsComponents.RigidBody):getRigidBody()
    local v = rb:getVelocity()
    local headU, headV = 0.5, 0.5
    if d.speed > 1e-9 then
        local u, w = project(v.x, v.y, v.z)
        if u then headU, headV = clamp01(u), clamp01(w) end
    end

    -- First person: the camera is in the ship, and the middle of the view is
    -- where you look, so the overlay and the blur ease off there
    local firstPerson = game.player ~= nil and game.player:getMode() == Enums.CameraMode.FirstPerson
    local fp = firstPerson and cfg.firstPerson or nil

    -- Where the effects come from: in third person the ship on screen; in
    -- first person (the camera is in it), or with the ship behind the camera,
    -- where it is heading. The radial blur, the overlay's ring and streaks
    -- and the dust (see updateDust) all come from there.
    local srcU, srcV = headU, headV
    if not firstPerson then
        local eye = CameraManager:getEye()
        local shipPos = rb:getPos()
        local u, w = project(shipPos.x - eye.x, shipPos.y - eye.y, shipPos.z - eye.z)
        if u then srcU, srcV = clamp01(u), clamp01(w) end
    end
    fx.blurCenterX, fx.blurCenterY = srcU, srcV

    -- The travel drive's ring: a ring `size` ship radii wide, `ahead` ship
    -- radii in front of the nose (in first person: of the camera, which sits
    -- above the ship's centre), projected: where it is on screen (0..1, y
    -- down) and its radius as a share of the screen height
    local ringU, ringV, ringR = srcU, srcV, 0.15
    do
        local rc = cfg.travelDrive
        local shipRadius = game.playerShip:get(PhysicsComponents.RigidBody):getRadius()
        local fwd = rb:getRot():getForward()
        local ahead = rc.ahead * shipRadius
        local x, y, z = fwd.x * ahead, fwd.y * ahead, fwd.z * ahead
        if not firstPerson then
            local eye = CameraManager:getEye()
            local p = rb:getPos()
            x, y, z = x + p.x - eye.x, y + p.y - eye.y, z + p.z - eye.z
        end
        local u, w, depth = project(x, y, z)
        fx.ringDistance = math.sqrt(x * x + y * y + z * z)
        if u then
            local tanHalf = math.tan(math.rad(CameraManager:getFov()) / 2)
            ringU, ringV = u, w
            ringR = Math.Clamp(rc.size * shipRadius / (2 * depth * tanHalf), rc.minRadius, 0.4)
            if fp then ringR = ringR * fp.ring end
        end
    end

    fx.radialBlur = cues.radialBlur
        and I * (cfg.radialBlur.boost * d.boost + cfg.radialBlur.travel * d.travel)
        * (fp and fp.radialBlur or 1) or 0
    -- The blur leaves the bubble sharp, so it doesn't smear streaks into it
    fx.blurClearX, fx.blurClearY = ringU, ringV
    fx.blurClearRadius = travelShown > 0 and cfg.dust.travel.bubbleRadius * ringR * exitScale or 0
    fx.aberration = cues.aberration
        and I * (cfg.aberration.boosting * d.boosting + cfg.aberration.travel * d.travel) or 0
    fx.vignette = cues.vignette
        and I * (cfg.vignette.boost * d.boost + cfg.vignette.travel * d.travel) or 0

    -- Travel drive overlay: the strength its material was written with
    -- (Shared/Definitions/MaterialDefs.lua, TravelDrive)
    local overlay = 0
    if cues.travelDrive then
        if d.state == "charging" then
            overlay = d.charge
        elseif d.state == "active" then
            overlay = 0.7 + 0.3 * math.min(1, d.multiplier / 50)
        elseif d.state == "decelerating" then
            overlay = clamp01((d.multiplier - 1) / 50)
        end
        overlay = math.max(overlay, exitFade * self.exitOverlay)
        if fp then overlay = overlay * fp.travelDrive end
        fx.ringX, fx.ringY, fx.ringRadius = ringU, ringV, ringR * exitScale
    end
    fx.travelDrive = I * cfg.travelDrive.gain * overlay
    fx.travelFlash = cues.travelDrive and I * flash or 0
    fx.driveSpeed = d.multiplier

    if self.dust then
        self:updateDust(cfg.dust, d, travelShown, v, headU, headV, srcU, srcV, ringU, ringV,
            ringR * exitScale, paused and 0 or dt)
    end
end

---@private
---The dust follows the ship's motion at up to each layer's `maxSpeed`; above
---that (travel mode) it falls behind (the layer's offset grows), so specks
---stay visible, and travel mode draws long, bright streaks that the drive's
---bubble pushes aside around the ship.
---
---By perspective, streaks converge on where the ship is heading. In third
---person that isn't where the ship is, so from `fromShip[1]` to `fromShip[2]`
---times max speed the dust is moved on screen until they converge on the
---ship (slower, it stays true to the world).
---@param dc table Config.render.speedFeel.dust
---@param d table what drives the cues (`self.drive`)
---@param travel number what to show of travel mode, 0..1 (d.travel, fading out on exit)
---@param v Vec3f the ship's velocity
---@param headU number where the ship is heading on screen (0..1, y down)
---@param headV number
---@param srcU number where the effects come from: the ship, or where it heads
---@param srcV number
---@param ringU number the travel drive's ring on screen: the bubble wraps it
---@param ringV number
---@param ringR number the ring's radius, share of the screen height
---@param dt number seconds since the last frame (0 while paused)
function SpeedFeelSystem:updateDust(dc, d, travel, v, headU, headV, srcU, srcV, ringU, ringV, ringR, dt)
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
        layer.offsetX = layer.offsetX + dirX * lag
        layer.offsetY = layer.offsetY + dirY * lag
        layer.offsetZ = layer.offsetZ + dirZ * lag

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
