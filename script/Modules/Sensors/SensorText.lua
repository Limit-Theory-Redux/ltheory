-- How sensor readings read as text: distances, classes and contact lines, in
-- the HUD's capitals. Works on the sensor model's plain tables (see
-- SensorModel).

local SensorModel = require("Modules.Sensors.SensorModel")

local SensorText = {}

SensorText.roman = { "I", "II", "III", "IV" }
--- Class icons: I support and drones, II industrial, III capital, IV station.
SensorText.classSymbols = { "○", "Δ", "▼", "⬢" }

local function thousands(n)
    local s = tostring(n)
    local result = s:reverse():gsub("(%d%d%d)", "%1,"):reverse()
    return (result:gsub("^,", ""))
end

--- A distance in metres: whole metres below 1 km, one decimal below 10 km,
--- whole kilometres above. The ranges go by the rounded value, so 999.6 m
--- reads 1.0 km and 9.96 km reads 10 km.
---@param d number metres
---@return string
function SensorText.distance(d)
    if not (d > 0) then return "—" end
    if d == math.huge then return "any distance" end
    local metres = math.floor(d + 0.5)
    if metres < 1000 then return string.format("%d m", metres) end
    local tenths = math.floor(d / 100 + 0.5)
    if tenths < 100 then return string.format("%.1f km", tenths / 10) end
    return thousands(math.floor(d / 1000 + 0.5)) .. " km"
end

--- A power draw: one decimal below 10 MW.
---@param mw number
---@return string
function SensorText.power(mw)
    if mw < 10 then return string.format("%.1f MW", mw) end
    return thousands(math.floor(mw + 0.5)) .. " MW"
end

--- An SNR or another strength: three figures, powers of ten beyond them.
---@param x number
---@return string
function SensorText.number(x)
    if x ~= x or x == math.huge then return "∞" end
    if x == 0 then return "0" end
    if x >= 1e5 or x < 0.01 then
        local e = math.floor(math.log10(x))
        return string.format("%.1fe%d", x / 10 ^ e, e)
    end
    if x >= 100 then return thousands(math.floor(x + 0.5)) end
    return string.format(x >= 10 and "%.1f" or "%.2f", x)
end

---@class SensorIdentity
---@field name string|nil what a T3 read shows, "[Manufacturer] [Model] (Year)" for vessels
---@field details string|nil a second T3 line (shields, faction ...)
---@field contents string[]|nil what a T4 read shows (cargo, logs ...)

--- The name a T3 read shows: the object's own, or its kind.
---@param o SensorObject|SensorIdentity
---@return string
function SensorText.name(o)
    return o.name or string.upper(SensorModel.kinds[o.kind].name)
end

--- A contact as the contact list shows it, for its tier. The class icon is
--- not part of the text (`SensorText.classSymbols` has it).
---@param o SensorObject
---@param r SensorResult
---@return string
function SensorText.contact(o, r)
    local d = string.upper(SensorText.distance(SensorModel.distance(o)))
    local class = SensorModel.shipClass(o)
    if r.tier == 1 then
        if r.count > 1 then return string.format("%d SIGNALS, ONE BLOB · %s", r.count, d) end
        return "UNKNOWN SIGNAL · " .. d
    end
    if r.tier == 2 then
        if class > 0 then return string.format("CLASS %s · %s", SensorText.roman[class], d) end
        return string.upper(SensorModel.kinds[o.kind].name) .. " · " .. d
    end
    if r.tier == 3 then return SensorText.name(o) .. " · " .. d end
    if r.tier >= 4 then return SensorText.name(o) .. " · FULL SCAN" end
    return ""
end

return SensorText
