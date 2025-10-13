---@class DeltaTimer
---@overload fun(self: DeltaTimer, name: string): DeltaTimer class internal
---@overload fun(name: string): DeltaTimer class external
local DeltaTimer = Class("DeltaTimer", function(self, name)
    ---@diagnostic disable-next-line
    self:init(name)
end)

---@private
function DeltaTimer:init(name)
    self.name = name or "Undefined"
    self.totalDt = 0         -- Accumulated delta time
    self.paused = false      -- Whether the timer is paused
    self.maxDeltaClamp = nil -- Optional clamp for large frame spikes
    self.data = {}           -- User-defined state
end

---Update the timer by delta time
---@param dt number
function DeltaTimer:update(dt)
    if self.paused then return end
    if self.maxDeltaClamp then
        dt = math.min(dt, self.maxDeltaClamp)
    end
    self.totalDt = self.totalDt + dt
end

---Check if the timer has reached a threshold
---@param threshold number
---@return boolean
function DeltaTimer:hasReached(threshold)
    return self.totalDt >= threshold
end

---Reset accumulated delta time
function DeltaTimer:reset()
    self.totalDt = 0
end

---Get accumulated delta time
---@return number
function DeltaTimer:getTotal()
    return self.totalDt
end

---Pause and resume
function DeltaTimer:pause() self.paused = true end
function DeltaTimer:resume() self.paused = false end

---Set maximum delta clamp
---@param maxDelta number
function DeltaTimer:setMaxDeltaClamp(maxDelta)
    self.maxDeltaClamp = maxDelta
end

---Update and check threshold, auto-reset when reached
---@param dt number
---@param threshold number|string number: threshold value, string: key to stored data
---@return boolean triggered
function DeltaTimer:updateAndCheck(dt, threshold)
    self:update(dt)

    if rawtype(threshold) == "string" then
        threshold = self.data[threshold]
        if rawtype(threshold) ~= "number" then
            Log.Error("DeltaTimer: Stored threshold '%s' is not a number", tostring(threshold))
        end
        ---@cast threshold number
    end
    if self:hasReached(threshold) then
        self:reset()
        return true
    end
    return false
end

---Return a normalized factor (0–1) toward threshold
---@param threshold number
---@return number
function DeltaTimer:getFactor(threshold)
    return math.min(1.0, self.totalDt / threshold)
end

---Store arbitrary key/value data
---@param key string
---@param value any
function DeltaTimer:set(key, value)
    self.data[key] = value
end

---Retrieve stored data
---@param key string
---@param default any|nil
---@return any
function DeltaTimer:get(key, default)
    local v = self.data[key]
    if v == nil then return default end
    return v
end

---Clear all data
function DeltaTimer:clearData()
    self.data = {}
end

---Return a debug string representation
---@return string
function DeltaTimer:__tostring()
    local state = self.paused and "paused" or "running"
    local summary = string.format(
        "[DeltaTimer (\"%s\"): %.4fs total, %s]",
        self.name,
        self.totalDt,
        state
    )

    -- Include key data summary if present
    local keys = {}
    for k, v in pairs(self.data) do
        table.insert(keys, string.format("%s=%s", tostring(k), tostring(v)))
    end
    if #keys > 0 then
        summary = summary .. " { " .. table.concat(keys, ", ") .. " }"
    end
    return summary
end

return DeltaTimer
