-- What a sensor shows, ready to draw: every object with its bearing and
-- distance in the ship's frame and its reading, the contacts in list order,
-- the target with its scan, and the sensor's own state. Built from a
-- `SensorComponent` once per frame; plain tables only.

local SensorModel = require("Modules.Sensors.SensorModel")

local SensorPicture = {}

local DEG = math.pi / 180

---@class SensorContact
---@field object SensorObject
---@field reading SensorResult
---@field bearing number degrees clockwise from the ship's nose, in its plane
---@field distance number metres
---@field index integer its place in the sensor's object list (stable per object)
---@field echo number seconds of pulse echo left (0 = none)

---@class SensorTargetScan
---@field contact SensorContact
---@field echo SensorScanEcho
---@field read integer the best tier a scan has read (0 = none)
---@field progress number the running scan's progress, 0 to 1
---@field reading boolean a scan is reading a tier now
---@field ranges SensorRanges where it would reach each tier, in open space

---@class SensorPictureData
---@field title string what the sensor is (a hull's name)
---@field dials SensorDials
---@field scanning boolean
---@field heat number own heat, 0 to 1
---@field bloom boolean own heat shows as a thermal bloom
---@field heardWithin number how far others hear the ship's own power draw, metres
---@field pulseAge number seconds since the last pulse
---@field pulseHeardWithin number how far the pulse is heard, metres
---@field pulseHeardBy integer ships and stations that heard the last pulse
---@field objects SensorContact[] every object, contact or not
---@field contacts SensorContact[] tier 1 and up: strongest tier first, then nearest
---@field target SensorTargetScan|nil
---@field caption { title: string, hint: string|nil }|nil what the HUD shows above it (set by the caller)

local function dot(a, b)
    return a.x * b.x + a.y * b.y + a.z * b.z
end

--- The picture of a sensor now.
---@param sensor SensorComponent
---@param forward Vec3f|SensorVec the ship's nose (unit length)
---@param right Vec3f|SensorVec the ship's right (unit length)
---@param title string
---@return SensorPictureData
function SensorPicture.build(sensor, forward, right, title)
    local state = sensor:getState()
    local readings = sensor:getReadings()
    local dials = state.dials

    local objects, contacts, targetContact = {}, {}, nil
    for i, o in ipairs(state.objects) do
        local r = readings[o]
        if r then
            local c = {
                object = o,
                reading = r,
                bearing = math.atan2(dot(o.pos, right), dot(o.pos, forward)) / DEG,
                distance = SensorModel.distance(o),
                index = i,
                echo = state.echoes[o] or 0,
            }
            objects[#objects + 1] = c
            if r.tier >= 1 then contacts[#contacts + 1] = c end
            if o == state.target then targetContact = c end
        end
    end
    table.sort(contacts, function(a, b)
        if a.reading.tier ~= b.reading.tier then return a.reading.tier > b.reading.tier end
        return a.distance < b.distance
    end)

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
        title = title,
        dials = dials,
        scanning = state.scanning,
        heat = state.heat,
        bloom = state.heat >= SensorModel.constants.bloom,
        heardWithin = SensorModel.heardWithin(dials.ownMw),
        pulseAge = state.pulseAge,
        pulseHeardWithin = SensorModel.heardWithin(dials.power),
        pulseHeardBy = state.pulseHeardBy,
        objects = objects,
        contacts = contacts,
        target = target,
    }
end

return SensorPicture
