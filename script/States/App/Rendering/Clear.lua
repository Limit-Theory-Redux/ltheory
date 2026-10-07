local Application = require("States.Application")

---@class RenderingClear: Application
local RenderingClear = Subclass("RenderingClear", Application)

local CLEAR_COLOR = {0.125, 0.375, 0.625, 1.0}

function RenderingClear:onInit()
    self.frames = 0
    self.renderFrames = 0
end

function RenderingClear:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames == 1 then
        Log.Info(string.format(
            "[ClearProbe] requested rgba=(%.3f,%.3f,%.3f,%.3f)",
            CLEAR_COLOR[1], CLEAR_COLOR[2], CLEAR_COLOR[3], CLEAR_COLOR[4]
        ))
    end
    if self.frames >= 120 then
        self:quit()
    end
end

function RenderingClear:onRender()
    self.renderFrames = self.renderFrames + 1
    if self.renderFrames == 1 then
        Log.Info("[ClearProbe] onRender callback submitted")
    end
    if not self.passDesc then
        self.passDesc = RenderPassDesc.Create("Clear")
        self.passDesc:backbuffer(self.resX, self.resY, LoadOp.Clear,
            CLEAR_COLOR[1], CLEAR_COLOR[2], CLEAR_COLOR[3], CLEAR_COLOR[4])
    end
    Renderer:beginPass(self.passDesc):finish()
end

return RenderingClear
