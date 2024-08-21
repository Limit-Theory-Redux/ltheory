---@class QuickProfiler
---@overload fun(self: QuickProfiler, name: string, enable: boolean, withMovingAverage: boolean, disablePrints: boolean): QuickProfiler class internal
---@overload fun(name: string, isEnabled: boolean, withMovingAverage: boolean, disablePrints: boolean): QuickProfiler class external
local QuickProfiler = Class(function(self, name, enable, withMovingAverage, disablePrints)
    ---@diagnostic disable-next-line: invisible
    self:init(name, enable, withMovingAverage, disablePrints)
end)

---@private
function QuickProfiler:init(name, enable, withMovingAverage, disablePrints)
    ---@private
    self.name = name or "Undefined"
    ---@private
    self.enabled = enable or false
    ---@private
    self.times = {}
    ---@private
    self.withMovingAverage = withMovingAverage
    ---@private
    self.disablePrints = disablePrints
end

function QuickProfiler:start()
    if self.enabled then
        ---@private
        self.startTime = TimeStamp.Now()

        if self.withMovingAverage and not self.movingAverageStartTime then
            ---@private
            self.movingAverageStartTime = TimeStamp.Now()
        end
    end
end

---@return integer|nil
function QuickProfiler:stop()
    if self.enabled and self.startTime then
        local stop = TimeStamp.Now()
        local diff = self.startTime:getDifference(stop)
        insert(self.times, diff)

        if not self.disablePrints then Log.Debug(format("%s, %.3f ms", self.name, diff * 1000)) end

        if self.withMovingAverage and self.movingAverageStartTime:getDifference(stop) >= 1.0 then
            local total = 0
            for _, time in ipairs(self.times) do
                total = total + time
            end

            total = total / #self.times
            local totalMS = total * 1000

            if not self.disablePrints then
                Log.Debug(self.name .. ", Moving average (1 Frame): " .. format("%.3f ms", totalMS))
                Log.Debug(self.name .. ", Percentage of 60FPS (16.67 ms) goal frametime: " .. format("%.2f", totalMS / 16.67 * 100) .. "%%") -- hack to make %% work with Log.Debug´s string.format()
                Log.Debug(self.name .. ", Percentage of 120FPS (8.33 ms) goal frametime: " .. format("%.2f", totalMS / 8.33 * 100) .. "%%") -- hack to make %% work with Log.Debug´s string.format()
            end

            table.clear(self.times)
            self.movingAverageStartTime = nil
        end
        self.startTime = nil
        return diff * 1000
    end
end

return QuickProfiler
