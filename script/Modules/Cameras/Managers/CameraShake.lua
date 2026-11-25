--- Camera shake: small, smooth turns of the view (yaw, pitch, roll) on top of
--- whatever camera is active. Only what is drawn turns: the camera's own
--- transform, and with it steering and aiming, stays as its controller set it.
---
--- Trauma model (Squirrel Eiserloh, "Juicing Your Cameras With Math", GDC 2016):
---   - `addTrauma(x)` for a kick (boost starts, travel drive engages). Trauma
---     is 0..1 and decays by `decay` per second.
---   - `setRumble(x)` for a steady shake (boosting, travel mode). It holds until
---     it is set again, so whoever sets it sets it every frame.
---   - The shake amount is max(trauma, rumble) squared, so small values stay
---     gentle and big ones grow quickly.
---   - Each axis follows its own smooth wobble (three sines at unrelated
---     frequencies), scaled by the amount and the axis's maximum angle.
---@class CameraShake
---@field maxYaw number degrees at full shake
---@field maxPitch number degrees at full shake
---@field maxRoll number degrees at full shake
---@field frequency number Hz of the slowest wobble (the others run about 2x and 4.6x faster)
---@field decay number trauma lost per second
local CameraShake = Class("CameraShake", function(self)
    self.maxYaw = 1.0
    self.maxPitch = 1.0
    self.maxRoll = 1.5
    self.frequency = 5.0
    self.decay = 1.2
    self.trauma = 0
    self.rumble = 0
    self.lastTime = nil
end)

---@param amount number 0..1, added to the current trauma (capped at 1)
function CameraShake:addTrauma(amount)
    self.trauma = math.min(1, self.trauma + math.max(0, amount))
end

---@param amount number 0..1, the steady shake from now on
function CameraShake:setRumble(amount)
    self.rumble = math.max(0, math.min(1, amount))
end

--- Stop shaking at once.
function CameraShake:reset()
    self.trauma = 0
    self.rumble = 0
end

---@return number amount 0..1: max(trauma, rumble) squared
function CameraShake:getAmount()
    local level = math.max(self.trauma, self.rumble)
    return level * level
end

-- A smooth wobble between -1 and 1: three sines at unrelated frequencies and
-- phases, so it never visibly repeats. `axis` picks the phases.
local function wobble(t, axis)
    local p = axis * 2.399 -- the golden angle: well-spread phases per axis
    return (math.sin(t * 6.283 + p)
        + 0.5 * math.sin(t * 13.71 + 2.0 * p)
        + 0.25 * math.sin(t * 29.17 + 3.0 * p)) / 1.75
end

--- The shake's turn at `time`, and the trauma decay since the last call.
---@param time number seconds (any clock that runs on)
---@return number yaw radians
---@return number pitch radians
---@return number roll radians
function CameraShake:sample(time)
    local dt = self.lastTime and math.max(0, time - self.lastTime) or 0
    self.lastTime = time
    self.trauma = math.max(0, self.trauma - self.decay * dt)

    local amount = self:getAmount()
    if amount <= 0 then return 0, 0, 0 end

    local t = time * self.frequency -- the wobble's slowest sine runs at `frequency` Hz
    return math.rad(self.maxYaw) * amount * wobble(t, 1),
        math.rad(self.maxPitch) * amount * wobble(t, 2),
        math.rad(self.maxRoll) * amount * wobble(t, 3)
end

return CameraShake
