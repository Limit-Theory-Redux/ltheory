-- What a sensor shows, ready to draw: every object with its bearing, height
-- and distance in the ship's frame, its reading and what the sensor remembers
-- of it, the target with its scan, and the sensor's own state. A contact the
-- sensor has lost but had classed shows as a ghost where it was last seen. Built from a
-- `SensorComponent` once per frame; plain tables only.

local SensorModel = require("Modules.Sensors.SensorModel")

local SensorPicture = {}

local DEG = math.pi / 180
local PULSE_HEARD_TIME = 4 -- seconds the HUD shows who heard a pulse

---@class SensorContact
---@field object SensorObject
---@field reading SensorResult
---@field bearing number degrees clockwise from the ship's nose, in its plane
---@field elevation number degrees above the ship's plane (negative: below)
---@field distance number metres
---@field world table|nil where it is in the world (game units); for a ghost, where it was last seen
---@field known integer the best tier the sensor has had of it (0 = none)
---@field age number seconds since it last had that tier
---@field stale boolean a contact now, but it senses less of it than it knows: the details are as last seen
---@field ghost boolean out of contact, shown where it was last seen
---@field lost number seconds since it was last a contact
---@field index integer its place in the sensor's object list (stable per object)
---@field echo number seconds of pulse echo left (0 = none)
---@field heardPulse boolean a ship or station within earshot of the last pulse, while it's recent

---@class SensorTargetScan
---@field contact SensorContact
---@field echo SensorScanEcho
---@field read integer the best tier a scan has read (0 = none)
---@field progress number the running scan's progress, 0 to 1
---@field reading boolean a scan is reading a tier now
---@field ranges SensorRanges where it would reach each tier, in open space

---@class SensorPictureData
---@field dials SensorDials
---@field scanning boolean
---@field heat number own heat, 0 to 1
---@field bloom boolean own heat shows as a thermal bloom
---@field heardWithin number how far others hear the ship's own power draw, metres
---@field pulseAge number seconds since the last pulse
---@field pulseHeardWithin number how far the pulse is heard, metres
---@field objects SensorContact[] every object, contact or not
---@field target SensorTargetScan|nil

local function dot(a, b)
    return a.x * b.x + a.y * b.y + a.z * b.z
end

--- Bearing and elevation (degrees) and distance (m) of a sensor-relative
--- position, in the ship's frame.
local function place(pos, forward, right, up)
    local f, r, u = dot(pos, forward), dot(pos, right), dot(pos, up)
    local flat = math.sqrt(f * f + r * r)
    return math.atan2(r, f) / DEG, math.atan2(u, flat) / DEG, math.sqrt(f * f + r * r + u * u)
end

--- The picture of a sensor now.
---@param sensor SensorComponent
---@param forward Vec3f|SensorVec the ship's nose (unit length)
---@param right Vec3f|SensorVec the ship's right (unit length)
---@param up Vec3f|SensorVec the ship's up (unit length)
---@return SensorPictureData
function SensorPicture.build(sensor, forward, right, up)
    local state = sensor:getState()
    local readings = sensor:getReadings()
    local dials = state.dials

    local objects, targetContact = {}, nil
    local pulseHeardWithin = SensorModel.heardWithin(dials.power)
    local pulseRecent = state.pulseAge < PULSE_HEARD_TIME
    for i, o in ipairs(state.objects) do
        local r = readings[o]
        if r then
            local m = state.memory[o]
            local ghost = r.tier == 0 and m ~= nil and m.tier >= 2 and o.lastPos ~= nil
            local bearing, elevation, distance = place(ghost and o.lastPos or o.pos, forward, right, up)
            local c = {
                object = o,
                reading = r,
                bearing = bearing,
                elevation = elevation,
                distance = distance,
                world = ghost and o.seenAt or o.world,
                index = i,
                echo = state.echoes[o] or 0,
                known = m and m.tier or 0,
                age = m and m.age or 0,
                stale = r.tier >= 1 and m ~= nil and m.tier > r.sensed and m.age > 0,
                ghost = ghost,
                lost = m and m.lost or 0,
            }
            c.heardPulse = not ghost and pulseRecent and (o.kind == "ship" or o.kind == "station")
                and c.distance <= pulseHeardWithin
            objects[#objects + 1] = c
            if o == state.target then targetContact = c end
        end
    end
    local target
    if targetContact then
        local o = targetContact.object
        local echo = SensorModel.scanEcho(state, targetContact.reading.passive)
        local read = state.reads[o] or 0
        target = {
            contact = targetContact,
            echo = echo,
            read = read,
            progress = state.progress[o] or 0,
            reading = state.scanning and targetContact.reading.tier >= 1 and echo.level > read,
            ranges = SensorModel.ranges(state, o),
        }
    end

    return {
        dials = dials,
        scanning = state.scanning,
        heat = state.heat,
        bloom = state.heat >= SensorModel.constants.bloom,
        heardWithin = SensorModel.heardWithin(dials.ownMw),
        pulseAge = state.pulseAge,
        pulseHeardWithin = SensorModel.heardWithin(dials.power),
        objects = objects,
        target = target,
    }
end

return SensorPicture
