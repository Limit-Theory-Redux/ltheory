-- Logs the GPU's frame time and its heaviest render passes every LTHEORY_GPU_LOG seconds (for
-- measuring what a scene costs to draw, in a log rather than the render info overlay). The times
-- are the renderer's smoothed per-pass timestamps (`Renderer:statsGpu*`); without GPU timing
-- (`LTHEORY_GPU_TIMING=0`, or a backend that can't) it logs once that there are none.

local ffi = require("ffi")

---@class GpuTimingLog
---@overload fun(self: GpuTimingLog): GpuTimingLog class internal
---@overload fun(): GpuTimingLog class external
local GpuTimingLog = Class("GpuTimingLog", function(self)
    self.intervalS = tonumber(os.getenv("LTHEORY_GPU_LOG"))
    self.enabled = self.intervalS ~= nil and self.intervalS > 0
    self.passes = 8
    self.clock = nil
    self.nextAt = 0
    self.warned = false
end)

---Logs the report when it is due; call once a frame.
function GpuTimingLog:frame()
    if not self.enabled then return end
    self.clock = self.clock or TimeStamp.Now()
    local now = self.clock:getElapsed()
    if now < self.nextAt then return end
    self.nextAt = now + self.intervalS
    if not Renderer:statsGpuAvailable() or tonumber(Renderer:statsGpuFrames()) == 0 then
        if not self.warned then
            self.warned = true
            Log.Info("GpuTimingLog: no GPU timings")
        end
        return
    end
    Log.Info("GpuTimingLog: %.2f ms a frame (busy %.2f) | %s", tonumber(Renderer:statsGpuTotalSmoothUs()) / 1000,
        tonumber(Renderer:statsGpuBusyUs()) / 1000, ffi.string(Renderer:statsGpuSummary(self.passes)))
end

return GpuTimingLog()
