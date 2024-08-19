---@class QuickProfiler
---@overload fun(self: QuickProfiler, name: string, enable: boolean, withMovingAverage: boolean): QuickProfiler class internal
---@overload fun(name: string, isEnabled: boolean, withMovingAverage: boolean): QuickProfiler class external
local QuickProfiler = Class(function(self, name, enable, withMovingAverage)
    self:init(name, enable, withMovingAverage)
end)

function QuickProfiler:init(name, enable, withMovingAverage)
    self.name = name or "Undefined"
    self.enabled = enable or false
    self.times = {}
    self.withMovingAverage = withMovingAverage
end

function QuickProfiler:start()
    if self.enabled then
        self.start = TimeStamp.Now()

        if self.withMovingAverage and not self.movingAverageStart then
            self.movingAverageStart = TimeStamp.Now()
        end
    end
end

function QuickProfiler:stop()
    if self.enabled and self.start then
        local stop = TimeStamp.Now()
        local diff = self.start:getDifference(stop)
        insert(self.times, diff)
        Log.Debug(format("%s, %.3f ms", self.name, diff * 1000))

        if self.withMovingAverage and self.movingAverageStart:getDifference(stop) >= 1.0 then
            local total = 0
            for _, time in ipairs(self.times) do
                total = total + time
            end

            total = total / #self.times
            local totalMS = total * 1000

            Log.Debug(self.name .. ", Moving average (1.0 s): " .. format("%.3f ms", totalMS))
            Log.Debug(self.name .. ", Percentage of 60FPS goal frametime: " .. format("%.2f", totalMS / 16.67 * 100) .. "%%") -- hack to make %% work with Log.Debug´s string.format()
            Log.Debug(self.name .. ", Percentage of 120FPS goal frametime: " .. format("%.2f", totalMS / 8.33 * 100) .. "%%") -- hack to make %% work with Log.Debug´s string.format()

            self.times = {}
            self.movingAverageStart = nil
        end
        self.start = nil
    end
end

return QuickProfiler
