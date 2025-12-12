local Bindings = require('States.ApplicationBindings')
local MainMenu = require('Legacy.Systems.Menus.MainMenu')

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
    self.gcThresholdKB = Config.gc and Config.gc.thresholdKB or 65536 -- 64 MB
    self.gcHighWaterMark = nil                                        -- Will be set on first onPostRender

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

    -- Calculate memory growth per frame
    local growth = currentMem - self.prevMem

    -- Start cleaning if memory exceeds threshold
    if not self.cleaning and currentMem > self.gcThresholdKB then
        self.cleaning = true
        GC.debug.spreadFrames = 0 -- reset frame counter for new cycle
    end

    if self.cleaning then
        Profiler.Begin('GC.Step')

        -- Adaptive step size
        local baseStep = 1000
        local growthFactor = math.ceil(growth / 10)
        local stepSize = math.max(baseStep, growthFactor)

        -- Cap step size to avoid hitches
        local maxStep = 10000
        stepSize = math.min(stepSize, maxStep)

        -- Emergency full collection if memory spikes
        local emergencyThreshold = self.gcThresholdKB * 5
        if currentMem > emergencyThreshold then
            GC.Collect() -- sets GC.debug.emergencyTriggered = true
            self.cleaning = false
        else
            -- Incremental GC
            local done = GC.Step(stepSize)
            if done then
                self.cleaning = false
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

    Profiler.End()
end

function Application:onPreInput(data) end

function Application:onInput(data)
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onInput')

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
