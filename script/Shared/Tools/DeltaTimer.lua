---@class TimerEntry
---@field totalDt number
---@field threshold number
---@field paused boolean
---@field maxDeltaClamp number|nil
---@field data table<string, any>
---@field autoReset boolean

---@class DeltaTimer
---@field private _timers table<string, TimerEntry>
---@overload fun(self): DeltaTimer
local DeltaTimer = Class("DeltaTimer", function(self, name)
    self.name = name
    self._timers = {}
end)

---@private
function DeltaTimer:_get(key)
    local e = self._timers[key]
    if not e then
        e = {
            totalDt       = 0,
            threshold     = 0,
            paused        = false,
            maxDeltaClamp = nil,
            data          = {},
            autoReset     = true,
        }
        self._timers[key] = e
    end
    return e
end

---@param key string
---@param threshold number
---@param autoReset? boolean   default = true
function DeltaTimer:start(key, threshold, autoReset)
    local e     = self:_get(key)
    e.threshold = threshold
    e.totalDt   = 0
    e.paused    = false
    e.autoReset = autoReset ~= false
end

---@param dt number
function DeltaTimer:update(dt)
    for _, e in pairs(self._timers) do
        if not e.paused then
            local clamped = e.maxDeltaClamp and math.min(dt, e.maxDeltaClamp) or dt
            e.totalDt = e.totalDt + clamped
        end
    end
end

---@param key string
---@return boolean triggered
function DeltaTimer:check(key)
    local e = self._timers[key]
    if not e or e.totalDt < e.threshold then return false end

    if e.autoReset then
        e.totalDt = e.totalDt - e.threshold
    else
        e.totalDt = e.threshold
    end
    return true
end

---@param key string
function DeltaTimer:stop(key)
    local e = self._timers[key]
    if e then e.paused = true end
end

---@param key string
function DeltaTimer:resume(key)
    local e = self._timers[key]
    if e then e.paused = false end
end

---@param key string
function DeltaTimer:remove(key)
    self._timers[key] = nil
end

---@param key string
function DeltaTimer:reset(key)
    local e = self._timers[key]
    if e then e.totalDt = 0 end
end

---@param key string
---@return number? time left until threshold
function DeltaTimer:timeLeft(key)
    local e = self._timers[key]
    if not e then return nil end
    return math.max(0, e.threshold - e.totalDt)
end

---@param key string
---@return boolean is active and running
function DeltaTimer:isActive(key)
    local e = self._timers[key]
    return e ~= nil and not e.paused
end

function DeltaTimer:getTotal(key)
    local e = self._timers[key]; return e and e.totalDt or 0
end
function DeltaTimer:getFactor(key)
    local e = self._timers[key]; return e and math.min(1, e.totalDt / e.threshold) or 0
end
function DeltaTimer:has(key) return self._timers[key] ~= nil end

function DeltaTimer:set(key, k, v)
    local e = self:_get(key); e.data[k] = v
end
function DeltaTimer:get(key, k, d)
    local e = self._timers[key]; return e and e.data[k] or d
end
function DeltaTimer:clearData(key)
    local e = self._timers[key]; if e then e.data = {} end
end

function DeltaTimer:__tostring()
    local lines = { "DeltaTimer {" }
    for k, e in pairs(self._timers) do
        local state = e.paused and "paused" or "running"
        table.insert(lines,
            string.format('  "%s": %.3f/%.3f [%s]', k, e.totalDt, e.threshold, state))
    end
    table.insert(lines, "}")
    return table.concat(lines, "\n")
end

return DeltaTimer
