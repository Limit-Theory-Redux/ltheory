-- Times every event subscriber (each system's PreSim, Sim, PostSim, render... handlers) when
-- LTHEORY_PROFILE_SUBSCRIBERS is set, and logs, every `REPORT_S` seconds, the costliest ones in
-- milliseconds per frame, with the total. A subscriber is named by its context's class and the event
-- (`EventTunnelNames`, kept by `EventBus:subscribe`). For measuring what the simulation costs at a
-- scale (`Testbeds/EconomyGame` with LTHEORY_ECONOMY_SHIPS), apart from the renderer's GPU time.

---@class SubscriberProfile
---@overload fun(self: SubscriberProfile): SubscriberProfile class internal
---@overload fun(): SubscriberProfile class external
local SubscriberProfile = Class("SubscriberProfile", function(self)
    self.enabled = os.getenv("LTHEORY_PROFILE_SUBSCRIBERS") ~= nil
end)

local REPORT_S = 5
local TOP = 22

local spent, frames, since = {}, 0, nil
-- Named sections inside a subscriber (`mark`), reported apart.
local sections = {}

---Calls the tunnel and adds its time to its subscriber.
---@param tunnelId integer
---@param tunnel function
function SubscriberProfile:call(tunnelId, tunnel, ...)
    local start = os.clock()
    tunnel(...)
    spent[tunnelId] = (spent[tunnelId] or 0) + (os.clock() - start)
end

---Adds the time since `t0` (an `os.clock()`) to the named section; nothing while disabled. A caller
---times a part of its work: `local t = os.clock() ... SubscriberProfile:mark("physics", t)`.
---@param name string
---@param t0 number
function SubscriberProfile:mark(name, t0)
    if not self.enabled then return end
    sections[name] = (sections[name] or 0) + (os.clock() - t0)
end

---Counts a frame, and logs the report when it is due.
function SubscriberProfile:frame()
    if not self.enabled then return end
    frames = frames + 1
    local now = os.clock()
    since = since or now
    if now - since < REPORT_S then return end
    -- Event ids to names, for the report.
    local eventNames = {}
    for name, id in pairs(Event) do
        if rawtype(id) == "number" then eventNames[tostring(id)] = name end
    end
    local list, total = {}, 0
    -- Sections are parts of subscribers' time: listed, not added to the total.
    for name, seconds in pairs(sections) do
        list[#list + 1] = { name = "  section " .. name, ms = seconds * 1000 / frames }
    end
    for id, seconds in pairs(spent) do
        local name = EventTunnelNames[id] or tostring(id)
        name = name:gsub(":(%d+)$", function(n) return ":" .. (eventNames[n] or n) end)
        list[#list + 1] = { name = name, ms = seconds * 1000 / frames }
        total = total + seconds
    end
    table.sort(list, function(a, b) return a.ms > b.ms end)
    Log.Info("SubscriberProfile: %.2f ms a frame in subscribers over %d frames (CPU)", total * 1000 / frames, frames)
    for i = 1, math.min(TOP, #list) do
        Log.Info("SubscriberProfile: %7.3f ms  %s", list[i].ms, list[i].name)
    end
    spent, sections, frames, since = {}, {}, 0, now
end

return SubscriberProfile()
