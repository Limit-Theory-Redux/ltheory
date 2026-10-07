--- Auto-exposure convergence test (render API v2, S8).
--- Drives `RenderCoreSystem:tonemap` with auto-exposure on over two uniform scenes (black, then mid-gray) and logs
--- the exposure target and the adapted exposure every frame as `[AutoExposure]` lines, then a PASS/FAIL summary and
--- the main-thread cost of `tonemap`. The luminance of a uniform frame does not depend on which part of it is
--- sampled, so the synchronous version (128 `sample()` reads per frame) and the asynchronous one (one readback of a
--- small mip, resolved a few frames later) must converge to the same values; `tools/render_validation/
--- auto_exposure_test.py` runs it and checks that.
---
--- Run: `ltr -e ./script/Main.lua AutoExposure`
local Application = require('States.Application')
local RenderCoreSystem = require('Modules.Rendering.Systems.RenderCoreSystem')

---@class AutoExposure: Application
local AutoExposure = Subclass('AutoExposure', Application)

function AutoExposure:getTitle() return 'Auto-exposure' end

local DT = 1 / 60
local PHASE_FRAMES = 150
local FRAMES = PHASE_FRAMES * 2
-- Black (no light at all: the exposure target is the maximum) then mid-gray (bright: the minimum).
local PHASES = {
    { name = 'dark', value = 0.0 },
    { name = 'bright', value = 0.5 },
}

function AutoExposure:onInit()
    self.rc = RenderCoreSystem
    self.rc.postSettings.tonemap.autoExpose.enable = true
    self.renderFrames = 0
    self.tonemapMs = 0
    self.last = nil
    self.passDesc = RenderPassDesc.Create('AutoExposure')
    self.passDesc:backbuffer(self.resX, self.resY, LoadOp.Clear, 0, 0, 0, 1)
end

function AutoExposure:onRender()
    self.renderFrames = self.renderFrames + 1
    local frame = self.renderFrames
    if frame > FRAMES then return end

    local phase = PHASES[math.min(#PHASES, math.floor((frame - 1) / PHASE_FRAMES) + 1)]
    local rc = self.rc
    rc.level = 0
    rc.buffers[Enums.BufferName.buffer0]:clear(phase.value, phase.value, phase.value, 1)

    local start = TimeStamp.Now()
    rc:tonemap(DT)
    local elapsed = start:getElapsed() * 1000
    if frame > 30 then self.tonemapMs = self.tonemapMs + elapsed end

    local ae = rc.autoExposure
    Log.Info('[AutoExposure] frame=%d phase=%s target=%.5f current=%.5f', frame, phase.name, ae.target, ae.current)
    self.last = self.last or {}
    self.last[phase.name] = { target = ae.target, current = ae.current }

    Renderer:beginPass(self.passDesc):finish()
end

function AutoExposure:eventLoop()
    Application.eventLoop(self)
    if self.renderFrames < FRAMES then return end

    local dark, bright = self.last.dark, self.last.bright
    -- Fixed-clamp targets of the two scenes (see PostFxConfig: minTarget, maxTarget).
    local ae = Config.render.postFx.tonemap.autoExpose
    local ok = math.abs(dark.target - ae.maxTarget) < 1e-4 and math.abs(bright.target - ae.minTarget) < 1e-4
    -- The adapted exposure moves towards the target at `speed` per second: after the dark phase it has gone most of
    -- the way up, after the bright one it has come most of the way down again.
    ok = ok and dark.current > 1.5 and bright.current < dark.current - 0.2
    Log.Info('[AutoExposure] RESULT dark_target=%.5f dark_current=%.5f bright_target=%.5f bright_current=%.5f',
        dark.target, dark.current, bright.target, bright.current)
    Log.Info('[AutoExposure] tonemap_main_thread_ms=%.4f (mean of frames 31..%d)',
        self.tonemapMs / (FRAMES - 30), FRAMES)
    Log.Info('[AutoExposure] %s', ok and 'PASS' or 'FAIL')
    self:quit()
end

return AutoExposure
