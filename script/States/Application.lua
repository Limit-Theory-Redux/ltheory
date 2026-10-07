local Bindings = require('States.ApplicationBindings')
local MainMenu = require('Legacy.Systems.Menus.MainMenu')
local ShaderHotReload = require('Render.ShaderHotReload')
local ShaderErrorOverlay = require('Shared.Tools.ShaderErrorOverlay')
local RenderInfoOverlay = require('Shared.Tools.RenderInfoOverlay')
local GeneralActions = require('Input.ActionBindings.GeneralActions')

---@class Application
local Application = Class("Application", function(self) end)

-- Opt-in deterministic screenshot capture (render validation):
--   LTHEORY_CAPTURE=<out.png>   save the final backbuffer after frame N and exit
--   LTHEORY_CAPTURE_FRAME=<n>   frame to capture (default 120)
-- Also fixes the window size and (engine-side) the frame delta time.
local CAPTURE_SIZE_X, CAPTURE_SIZE_Y = 1280, 720

function Application:getDefaultSize()
    if self.captureMode then return CAPTURE_SIZE_X, CAPTURE_SIZE_Y end
    return Config.render.window.defaultResX, Config.render.window.defaultResY
end

function Application:getTitle()
    return Config.gameTitle
end

function Application:getWindowMode()
    return Bit.Or32(WindowMode.Shown, WindowMode.Resizable)
end

function Application:onInit() end
function Application:onDraw() end
function Application:onResize(sx, sy) end
function Application:onUpdate(dt) end
function Application:onExit() end

function Application:quit()
    Engine:exit()
end

function Application:eventLoop()
    if not self.eventsRegistered then
        self:registerEvents()
        self.eventsRegistered = true
    end

    EventBus:startEventIteration()

    local eventData, payload = EventBus:nextEvent()
    while eventData ~= nil do
        EventTunnels[eventData:tunnelId()](eventData, payload)
        eventData, payload = EventBus:nextEvent()
    end
end

function Application:appInit()
    ShaderHotReload:init()

    self.eventsRegistered = false
    self.capturePath = os.getenv('LTHEORY_CAPTURE')
    self.captureMode = self.capturePath ~= nil
    self.captureFrame = tonumber(os.getenv('LTHEORY_CAPTURE_FRAME')) or 120
    self.frameCount = 0
    self.resX, self.resY = self:getDefaultSize()

    Window:setTitle(self:getTitle())
    Window:setCenteredPosition()
    Window:setSize(self.resX, self.resY)

    self.audio = Audio.Create()
    GameState.audio.manager = self.audio
    GameState.render.gameWindow = Window
    Window:setPresentMode(GameState.render.presentMode)

    if Config.jit.profile and Config.jit.profileInit then Jit.StartProfile() end

    Preload.Run()

    -- Settings
    self.profilerFont = Font.Load('NovaMono', 10)
    RenderInfoOverlay:init()
    self.lastUpdate = TimeStamp.Now()
    self.profiling = false
    self.toggleProfiler = false
    self.showBackgroundModeHints = true

    -- GC CONTROL: Disable automatic collection
    GC.Stop()
    -- Threshold for the manual drain (Application:onPostRender). Fixed
    -- 64 MB was below the game's real steady-state heap (66-90 MB), so
    -- GC.Step ran every frame draining overshoot at a constant ~12-15 ms
    -- tax (measured in the benchmark perf work). Instead of chasing a
    -- magic constant, track the heap high-water mark and only start
    -- collecting when memory GROWS beyond the previous peak: a state that
    -- has settled (menu idle, gameplay cruise) stops paying the tax
    -- entirely, while genuine growth (world gen, ship spawning) still
    -- gets collected.
    self.gcThresholdKB = Config.gc and Config.gc.thresholdKB or 0 -- 0 = adaptive
    self.gcAdaptive = self.gcThresholdKB == 0
    self.gcHighWaterMark = nil -- set on first onPostRender

    self:onInit()
    self:onResize(self.resX, self.resY)

    if Config.jit.dumpasm then Jit.StartDump() end
    if Config.jit.profile and not Config.jit.profileInit then Jit.StartProfile() end
    if Config.jit.verbose then Jit.StartVerbose() end

    Window:cursor():setGrabMode(CursorGrabMode.Confined)
    Window:cursor():setGrabMode(CursorGrabMode.None)
    Window:setCursorPosition(Vec2f(self.resX / 2, self.resY / 2))
end

function Application:registerEvents()
    EventBus:subscribe(Event.PreSim, self, self.onPreSim)
    EventBus:subscribe(Event.Sim, self, self.onSim)
    EventBus:subscribe(Event.PostSim, self, self.onPostSim)
    EventBus:subscribe(Event.PreRender, self, self.onPreRender)
    EventBus:subscribe(Event.Render, self, self.onRender)
    EventBus:subscribe(Event.PostRender, self, self.onPostRender)
    EventBus:subscribe(Event.PreInput, self, self.onPreInput)
    EventBus:subscribe(Event.Input, self, self.onInput)
    EventBus:subscribe(Event.PostInput, self, self.onPostInput)
end

function Application:onPreSim(data) end
function Application:onSim(data) end
function Application:onPostSim(data) end

function Application:onPreRender(data)
    ShaderHotReload:update()

    -- Dashboard toggle requests are picked up here (same safe point as the
    -- F10 binding): the profiler must only be toggled from the main thread
    -- outside any active scope, never from the HTTP thread.
    if Profiler.PendingToggle() then
        self.toggleProfiler = true
    end

    if self.toggleProfiler then
        self.toggleProfiler = false
        self.profiling = not self.profiling
        if self.profiling then Profiler.Enable() else Profiler.Disable() end
    end

    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onPreRender')

    self.timeScale = 1.0
    self.doScreenshot = false

    if GameState.paused then
        self.timeScale = 0.0
    else
        self.timeScale = 1.0
    end

    if Input:isDown(Bindings.TimeAccel) then
        self.timeScale = GameState.debug.timeAccelFactor
    end

    if self.timeScale ~= EventBus:getTimeScale() then
        EventBus:setTimeScale(self.timeScale)
    end

    local timeScaledDt = data:deltaTime()

    if GameState.player.humanPlayer and GameState.player.humanPlayer:getRoot().update then
        GameState.player.humanPlayer:getRoot():update(timeScaledDt)
        GameState.render.uiCanvas:update(timeScaledDt)
    end

    do
        Profiler.SetValue('gcmem', GC.GetMemory())
        Profiler.Begin('App.onResize')
        local size = Window:size()
        if size.x ~= self.resX or size.y ~= self.resY then
            self.resX = size.x
            self.resY = size.y
            GameState.render.resX = self.resX
            GameState.render.resY = self.resY
            self:onResize(self.resX, self.resY)
        end
        Profiler.End()
    end
    Profiler.End()
end

function Application:onRender(data)
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onRender')

    Profiler.End()
end

function Application:onPostRender(data)
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onPostRender')

    local currentMem = GC.GetMemory()

    -- Initialize previous memory if needed
    if not self.prevMem then
        self.prevMem = currentMem
    end

    -- Adaptive threshold (gcThresholdKB == 0): baseline follows the heap.
    -- The threshold is set ONCE (first frame) from the initial heap, then
    -- re-baselined only AFTER a completed collect (see below). It must
    -- NOT be refreshed every frame: that would keep the threshold glued
    -- to currentMem + margin, so the heap is always BELOW it, cleaning
    -- never starts, GC.Step never runs, and the Lua heap grows unbounded
    -- (measured ~30 MB/s -> 3 GB in minutes).
    local GC_MARGIN_KB = 8192 -- 8 MB of headroom above the baseline
    if self.gcAdaptive and self.gcThresholdKB == 0 then
        self.gcThresholdKB = currentMem + GC_MARGIN_KB
    end

    -- Start cleaning if memory exceeds threshold
    if not self.cleaning and currentMem > self.gcThresholdKB then
        self.cleaning = true
        GC.debug.spreadFrames = 0 -- reset frame counter for new cycle
    end

    if self.cleaning then
        Profiler.Begin('GC.Step')

        -- Adaptive step size (KB of GC work per frame).
        --
        -- Old policy: stepSize = max(1000, ceil(growth/10)) capped at
        -- 10000 - only ~10% of the allocation rate, so the heap climbed
        -- past the threshold until the 5x-emergency fired a synchronous
        -- full collect (measured 303 ms pause in-game). That emergency
        -- full GC is the frame-killing spike.
        --
        -- v2 (overshoot/4) drained too hard: with a large overshoot it
        -- stepped ~32 MB/frame, a constant ~35 ms tax every frame.
        --
        -- v3: drain a FRACTION of the overshoot per frame (1/16, capped
        -- at 10 MB/frame). The heap pins near the threshold, the drain is
        -- spread over many frames at a bounded per-frame cost, and the
        -- synchronous full collect is gone entirely.
        local overshoot = currentMem - self.gcThresholdKB
        local stepSize
        if overshoot > 0 then
            stepSize = math.ceil(overshoot / 16)
        else
            stepSize = 1000
        end
        stepSize = math.min(stepSize, 10000)

        local done = GC.Step(stepSize)
        if done then
            self.cleaning = false
            -- Re-baseline the adaptive threshold after a completed
            -- collect: memory now sits at the post-collect level; the
            -- next drain should only fire when the heap GROWS beyond
            -- it again (by the margin), not on the very next frame.
            if self.gcAdaptive then
                self.gcThresholdKB = GC.GetMemory() + GC_MARGIN_KB
            end
        end

        -- **! seems to be a bug: engine restarts GC on collect, so we stop it again**
        GC.Stop()

        Profiler.End()
    end

    -- Update previous memory for next frame
    self.prevMem = currentMem

    -- Expose debug values to profiler/UI
    Profiler.SetValue('gc_debug_stepSize', GC.debug.stepSize)
    Profiler.SetValue('gc_debug_lastMem', GC.debug.lastMem)
    Profiler.SetValue('gc_debug_emergencyTriggered', GC.debug.emergencyTriggered and 1 or 0)
    Profiler.SetValue('gc_debug_spreadFrames', GC.debug.spreadFrames)

    self:immediateUI(function() ShaderErrorOverlay:draw() end)

    -- Render info overlay: drawn last, on top. Hidden in capture mode unless
    -- LTHEORY_OVERLAY=1 (so validation captures stay identical).
    RenderInfoOverlay:tick()
    GeneralActions.RenderInfoOverlay:update(data:deltaTime())
    if GeneralActions.RenderInfoOverlay:isPressed() then RenderInfoOverlay:toggle() end
    if RenderInfoOverlay.visible and (not self.captureMode or RenderInfoOverlay.forced) then
        self:immediateUI(function() RenderInfoOverlay:draw() end)
    end

    Profiler.End()

    if self.captureMode then self:captureTick() end

    -- Flush accumulated scope frame-times into the totals once per frame.
    -- Without this, every scope's total stays 0 and the printed table is
    -- empty (begin/end only accumulate into scope.frame).
    Profiler.LoopMarker()
end

function Application:captureTick()
    self.frameCount = self.frameCount + 1
    -- Wall-clock timing over the second half of the run (first half is warmup)
    local half = math.floor(self.captureFrame / 2)
    if self.frameCount == half then
        self.captureStart = TimeStamp.Now()
        self.captureAcc = { lastFrame = -1, recv = {}, present = {}, busy = {}, commands = {}, draws = {}, mainWait = {} }
    elseif self.frameCount > half and self.captureAcc then
        -- Per-frame render-thread stats (of the previous completed frame), averaged below
        local acc = self.captureAcc
        local rtFrame = tonumber(Renderer:statsFrameCount())
        if rtFrame ~= acc.lastFrame then -- skip ticks without a new render-thread frame
            acc.lastFrame = rtFrame
            table.insert(acc.recv, tonumber(Renderer:statsRecvWaitUs()))
            table.insert(acc.present, tonumber(Renderer:statsPresentWaitUs()))
            table.insert(acc.busy, tonumber(Renderer:statsFrameTimeUs()))
            table.insert(acc.commands, tonumber(Renderer:statsCommands()))
            table.insert(acc.draws, tonumber(Renderer:statsDrawCalls()))
            table.insert(acc.mainWait, tonumber(Renderer:statsMainWaitUs()))
        end
    end
    if self.captureDone or self.frameCount < self.captureFrame then return end
    self.captureDone = true

    Renderer:sync()
    Window:beginDraw() -- ScreenCapture measures the open pass target and reads the backbuffer (readSync)
    local tex = Tex2D.ScreenCapture()
    Window:endDraw()
    tex:save(self.capturePath)

    -- RenderCoreSystem's smoothed FPS is derived from the fixed capture dt, so
    -- report real wall-clock frame time measured here instead.
    local frames = self.frameCount - math.floor(self.captureFrame / 2)
    local ft = frames > 0 and self.captureStart:getElapsed() * 1000 / frames or 0
    local fps = ft > 0 and 1000 / ft or 0
    -- Producer/consumer balance: share of the frame the render thread spent
    -- not executing/presenting (>= 15% idle => Lua/main thread is the bottleneck).
    -- Medians (microseconds -> ms) so one-off hitches don't skew the balance.
    local function median(t)
        table.sort(t)
        return t[math.floor(#t / 2) + 1] or 0
    end
    local acc = self.captureAcc
    local recvMs, mainWaitMs = median(acc.recv) / 1000, median(acc.mainWait) / 1000
    local execMs = median(acc.busy) / 1000 - recvMs -- render-thread time spent executing commands
    local presentMs = median(acc.present) / 1000
    local idlePct = ft > 0 and math.max(0, 100 * (1 - (execMs + presentMs) / ft)) or 0
    Log.Info('CAPTURE frame=%d fps=%.1f frametime_ms=%.2f render_thread_ms=%.2f render_recv_wait_ms=%.2f render_idle_pct=%.1f render_exec_ms=%.2f render_present_ms=%.2f main_wait_ms=%.2f commands_per_frame=%d draw_calls=%.0f vertices=%d bound=%s path=%s',
        self.frameCount, fps, ft, tonumber(Renderer:statsFrameTimeUs()) / 1000,
        recvMs, idlePct, execMs, presentMs, mainWaitMs, median(acc.commands), median(acc.draws),
        tonumber(Renderer:statsVertices()), idlePct >= 15 and 'producer' or 'consumer', self.capturePath)
    self:quit()
end

function Application:onPreInput(data) end

function Application:onInput(data)
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onInput')

    if ShaderErrorOverlay:handleInput() then
        Profiler.End()
        return
    end

    if Input:isKeyboardAltPressed() and Input:isPressed(Button.KeyboardQ) then self:quit() end
    if Input:isPressed(Bindings.Exit) then self:quit() end

    if Input:isPressed(Bindings.ToggleProfiler) then
        self.toggleProfiler = true
    end

    if Input:isPressed(Bindings.Screenshot) then
        self.doScreenshot = true
        if Settings.exists('render.superSample') then
            self.prevSS = Settings.get('render.superSample')
        end
    end

    if Input:isPressed(Bindings.ToggleFullscreen) then
        GameState.render.fullscreen = not GameState.render.fullscreen
        Window:setFullscreen(GameState.render.fullscreen, GameState.render.fullscreenExclusive)
    end

    if Input:isPressed(Bindings.Reload) then
        Profiler.Begin('Engine.Reload')
        Cache.Clear()
        SendEvent('Engine.Reload')
        Preload.Run()
        Profiler.End()
    end

    if Input:isPressed(Bindings.Pause) and GameState:GetCurrentState() == Enums.GameStates.InGame then
        if GameState.paused then
            GameState.paused = false
            if not GameState.panelActive and not GameState.debug.instantJobs then
                Input:setCursorVisible(false)
            end
        else
            GameState.paused = true
            Input:setCursorVisible(true)
        end
    end

    if not Gui:hasActiveInput() then
        if Input:isPressed(Bindings.ToggleWireframe) then
            GameState.debug.physics.drawWireframe = not GameState.debug.physics.drawWireframe
        end

        if Input:isPressed(Bindings.ToggleMetrics) then
            GameState.debug.metricsEnabled = not GameState.debug.metricsEnabled
        end

        if MainMenu.inBackgroundMode and Input:isPressed(Bindings.ToggleHUD) then
            self.showBackgroundModeHints = not self.showBackgroundModeHints
        end
    end

    if GameState.render.uiCanvas ~= nil then
        GameState.render.uiCanvas:input()
    end

    Profiler.End()
end

function Application:onPostInput(data) end

function Application:doExit()
    if self.profiling then Profiler.Disable() end
    if Config.jit.dumpasm then Jit.StopDump() end
    if Config.jit.profile then Jit.StopProfile() end
    if Config.jit.verbose then Jit.StopVerbose() end

    -- Final collection before exit
    GC.Collect()

    self:onExit()
end

---@param renderFn function render function for immediate ui
function Application:immediateUI(renderFn)
    -- Re-open backbuffer for immediate UI
    Window:beginDraw()
    ClipRect.PushDisabled()

    do
        renderFn()
    end

    -- Close again
    ClipRect.Pop()
    Window:endDraw()
end

return Application
