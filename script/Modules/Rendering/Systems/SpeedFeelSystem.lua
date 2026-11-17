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
--- Off unless enabled (`enable(game)`): the Speed Feel Test scenario enables
--- it and switches cues; the game itself doesn't use it yet. It writes only
--- to the camera manager (field of view offset, shake) and to
--- RenderCoreSystem.motionFx, and puts everything back on `disable()`.
---@class SpeedFeelSystem
local SpeedFeelSystem = Class("SpeedFeelSystem", function(self)
    self.cueNames = { "fov", "shake", "radialBlur", "aberration", "vignette", "travelDrive", "dust" }
    self.cues = {}
    for _, name in ipairs(self.cueNames) do self.cues[name] = false end
    self.intensity = 1.0
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

---@param name string one of `cueNames`
---@param on boolean
function SpeedFeelSystem:setCue(name, on)
    assert(self.cues[name] ~= nil, "SpeedFeelSystem: no cue " .. tostring(name))
    self.cues[name] = on
    if name == "dust" then self:setDust(on) end
end

---@param name string
---@return boolean
function SpeedFeelSystem:isCueOn(name)
    return self.cues[name] == true
end

---@param intensity number scales every cue (1 = as configured)
function SpeedFeelSystem:setIntensity(intensity)
    self.intensity = math.max(0, intensity)
end

---@return number
function SpeedFeelSystem:getIntensity()
    return self.intensity
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
        self.dust.streakTime = cfg.streakTime
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
end

-- A point on screen (0..1, y down) from a camera-relative direction or
-- position, the way the drawn view projects it; nil when behind the camera.
local function project(x, y, z)
    local rot = CameraManager:getViewRot()
    local fwd, right, up = rot:getForward(), rot:getRight(), rot:getUp()
    local depth = x * fwd.x + y * fwd.y + z * fwd.z
    if depth <= 1e-9 then return nil end
    local tanHalf = math.tan(math.rad(CameraManager:getFov()) / 2)
    local aspect = Window:width() / Window:height()
    local sx = (x * right.x + y * right.y + z * right.z) / (depth * tanHalf * aspect)
    local sy = (x * up.x + y * up.y + z * up.z) / (depth * tanHalf)
    return (sx + 1) * 0.5, (1 - sy) * 0.5
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
    -- The ship's max speed: the same source as the HUD's speed tape
    d.maxSpeed = Config.game.shipFlight.maxSpeedGame
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
    local I = self.intensity
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

    -- Field of view: follows the speed smoothly, plus short kicks
    if self.cues.fov then
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
    if self.cues.shake and not paused then
        local c = cfg.shake
        local drivingRumble = d.state == "active" and c.travelRumble or 0
        shake:setRumble(I * (c.boostRumble * d.boosting + c.chargeRumble * d.charge + drivingRumble))
        if boostStarted then shake:addTrauma(I * c.boostKick) end
        if engaged then shake:addTrauma(I * c.travelKick) end
        if dropped then shake:addTrauma(I * c.dropKick) end
    else
        shake:reset()
    end

    -- Where the ship is heading, on screen: the radial blur streaks from there
    local rb = game.playerShip:get(PhysicsComponents.RigidBody):getRigidBody()
    local v = rb:getVelocity()
    fx.blurCenterX, fx.blurCenterY = 0.5, 0.5
    if d.speed > 1e-9 then
        local u, w = project(v.x, v.y, v.z)
        if u then fx.blurCenterX, fx.blurCenterY = clamp01(u), clamp01(w) end
    end

    fx.radialBlur = self.cues.radialBlur
        and I * (cfg.radialBlur.boost * d.boost + cfg.radialBlur.travel * d.travel) or 0
    fx.aberration = self.cues.aberration
        and I * (cfg.aberration.boosting * d.boosting + cfg.aberration.travel * d.travel) or 0
    fx.vignette = self.cues.vignette
        and I * (cfg.vignette.boost * d.boost + cfg.vignette.travel * d.travel) or 0

    -- Travel drive overlay: the strength its material was written with
    -- (Shared/Definitions/MaterialDefs.lua, TravelDrive)
    local overlay = 0
    if self.cues.travelDrive then
        if d.state == "charging" then
            overlay = d.charge
        elseif d.state == "active" then
            overlay = 0.7 + 0.3 * math.min(1, d.multiplier / 50)
        elseif d.state == "decelerating" then
            overlay = clamp01((d.multiplier - 1) / 50)
        end
        local eye = CameraManager:getEye()
        local p = rb:getPos()
        local u, w = project(p.x - eye.x, p.y - eye.y, p.z - eye.z)
        fx.shipScreenX, fx.shipScreenY = u or 0.5, w or 0.5
    end
    fx.travelDrive = I * cfg.travelDrive.gain * overlay
    fx.driveSpeed = d.multiplier

    -- Dust: the camera's motion is the ship's; each layer has its speed band
    if self.dust then
        self.dust.vx, self.dust.vy, self.dust.vz = v.x, v.y, v.z
        local rel = d.speed / d.maxSpeed
        for i, layer in ipairs(self.dust.layers) do
            local lc = cfg.dust.layers[i]
            local fadeIn = lc.fadeIn and band(rel, lc.fadeIn[1], lc.fadeIn[2]) or 1
            local fadeOut = lc.fadeOut and (1 - band(rel, lc.fadeOut[1], lc.fadeOut[2])) or 1
            layer.opacity = math.min(1, I) * fadeIn * fadeOut
        end
    end
end

return SpeedFeelSystem()
