local Bindings = require('States.ApplicationBindings')
local MainMenu = require('Legacy.Systems.Menus.MainMenu')
local ShaderHotReload = require('Render.ShaderHotReload')
local ShaderErrorOverlay = require('Shared.Tools.ShaderErrorOverlay')

---@class Application
local Application = Class("Application", function(self) end)

function Application:getDefaultSize()
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
    -- On the first frame (and after each completed collect) the threshold
    -- is set to (currentMem + margin), so a settled state never exceeds
    -- it and pays no GC tax; genuine growth past the baseline still
    -- triggers the drain. Fixed thresholds (Config.gc.thresholdKB > 0)
    -- keep the old behavior.
    local GC_MARGIN_KB = 8192 -- 8 MB of headroom above the baseline
    if self.gcAdaptive then
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

    Profiler.End()

    -- Flush accumulated scope frame-times into the totals once per frame.
    -- Without this, every scope's total stays 0 and the printed table is
    -- empty (begin/end only accumulate into scope.frame).
    Profiler.LoopMarker()
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
    RenderState.PushAllDefaults()
    ClipRect.PushDisabled()

    do
        renderFn()
    end

    -- Close again
    ClipRect.Pop()
    RenderState.PopAll()
    Window:endDraw()
end

return Application
