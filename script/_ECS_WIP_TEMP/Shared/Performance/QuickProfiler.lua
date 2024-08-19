---@class QuickProfiler
---@overload fun(self: QuickProfiler, name: string, enable: boolean): QuickProfiler class internal
---@overload fun(name: string, isEnabled: boolean): QuickProfiler class external
local QuickProfiler = Class(function(self, name, enable)
    self:init(name, enable)
end)

function QuickProfiler:init(name, enable)
    self.name = name or "Undefined"
    self.enabled = enable or false
end

function QuickProfiler:start()
    if self.enabled then
        self.start = TimeStamp.Now()
    end
end

function QuickProfiler:stop()
    if self.enabled and self.start then
        local stop = TimeStamp.Now()
        Log.Debug(format("%s, %.3f ms", self.name, self.start:getDifference(stop) * 1000))

        self.start = nil
    end
end

return QuickProfiler
