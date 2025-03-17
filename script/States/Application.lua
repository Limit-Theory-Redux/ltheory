local Bindings = require('States.ApplicationBindings')
local MainMenu = require('Systems.Menus.MainMenu')

local Application = class(function(self) end)

-- Virtual ---------------------------------------------------------------------

function Application:getDefaultSize()
    return Config.render.defaultResX, Config.render.defaultResY
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
        --print("[" .. tostring(Render.ToString(eventData:getRender())) .. "]")
        --print("- Tunnel Id: " .. tostring(eventData:tunnelId()))

        EventTunnels[eventData:tunnelId()](eventData, payload)
        eventData, payload = EventBus:nextEvent()
    end
end

-- Application Template --------------------------------------------------------

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

    self:onInit()
    self:onResize(self.resX, self.resY)

    self.profilerFont = Font.Load('NovaMono', 10)
    self.lastUpdate = TimeStamp.Now() -- TODO: was TimeStamp.GetFuture(-1.0 / 60.0)

    if Config.jit.dumpasm then Jit.StartDump() end
    if Config.jit.profile and not Config.jit.profileInit then Jit.StartProfile() end
    if Config.jit.verbose then Jit.StartVerbose() end

    Window:cursor():setGrabMode(CursorGrabMode.Confined)
    Window:setCursorPosition(Vec2f(self.resX / 2, self.resY / 2))
    Window:cursor():setGrabMode(CursorGrabMode.None)

    self.profiling = false
    self.toggleProfiler = false
    self.showBackgroundModeHints = true
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

    --* system & canvas should probably subscribe to onPreRender themselves
    if GameState.player.humanPlayer and GameState.player.humanPlayer:getRoot().update then
        GameState.player.humanPlayer:getRoot():update(timeScaledDt)
        GameState.render.uiCanvas:update(timeScaledDt)
    end

    do
        Profiler.SetValue('gcmem', GC.GetMemory())
        Profiler.Begin('App.onResize')
        local size = Window:size()
        Window:cursor():setGrabMode(CursorGrabMode.None)
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

    Window:beginDraw()

    --* should they subscribe to onRender themselves?
    if GameState.render.uiCanvas ~= nil then
        GameState.render.uiCanvas:draw(self.resX, self.resY)
        Gui:draw()
    end

    Profiler.End()

    UI.DrawEx.TextAdditive(
        'Unageo-Medium',
        "WORK IN PROGRESS",
        20,
        self.resX / 2 - 20, 50, 40, 20,
        0.75, 0.75, 0.75, 0.75,
        0.5, 0.5
    )

    if GameState:GetCurrentState() ~= Enums.GameStates.MainMenu then
        --if GameState.paused then
        --    UI.DrawEx.TextAdditive(
        --        'NovaRound',
        --        "[PAUSED]",
        --        24,
        --        0, 0, self.resX, self.resY,
        --        1, 1, 1, 1,
        --        0.5, 0.99
        --    )
        --end

        --if GameState.player.currentShip and GameState.player.currentShip:isDestroyed() then
        --    --TODO: replace this with a general "is alive" game state here and in LTR,
        --    -- the whole process needs to be improved
        --    if MainMenu and not MainMenu.dialogDisplayed and
        --        not MainMenu.seedDialogDisplayed and
        --        not MainMenu.settingsScreenDisplayed then
        --        do
        --            UI.DrawEx.TextAdditive(
        --                'NovaRound',
        --                "[GAME OVER]",
        --                32,
        --                0, 0, self.resX, self.resY,
        --                1, 1, 1, 1,
        --                0.5, 0.5
        --            )
        --        end
        --    end
        --end
    end

    -- Take screenshot AFTER on-screen text is shown but BEFORE metrics are displayed
    if self.doScreenshot then
        -- Settings.set('render.superSample', 2) -- turn on mild supersampling
        ScreenCap()
        if self.prevSS then
            -- Settings.set('render.superSample', self.prevSS) -- restore previous supersampling setting
            self.prevSS = nil
        end
    end

    do -- Metrics display
        if GameState.debug.metricsEnabled then
            local dt = data:deltaTime()

            local s = string.format(
                '%.2f ms / %.0f fps / %.2f MB / %.1f K tris / %d draws / %d imms / %d swaps',
                1000.0 * dt,
                1.0 / dt,
                GC.GetMemory() / 1000.0,
                Metric.Get(Metric.TrisDrawn) / 1000,
                Metric.Get(Metric.DrawCalls),
                Metric.Get(Metric.Immediate),
                Metric.Get(Metric.FBOSwap))
            RenderState.PushBlendMode(BlendMode.Alpha)
            UI.DrawEx.SimpleRect(0, self.resY - 20, self.resX, self.resY, Color(0.1, 0.1, 0.1, 0.5))
            self.profilerFont:draw(s, 10, self.resY - 5, Color(1, 1, 1, 1))

            local y = self.resY - 5
            if self.profiling then
                self.profilerFont:draw('>> PROFILER ACTIVE <<', self.resX - 128, y, Color(1, 0, 0.15, 1))
                y = y - 12
            end
            RenderState.PopBlendMode()
        end
    end
    Profiler.End()
    Profiler.LoopMarker()
end

function Application:onPostRender(data)
    do -- End Draw
        Profiler.SetValue('gcmem', GC.GetMemory())
        Profiler.Begin('App.onPostRender')
        Window:endDraw()
        Profiler.End()
    end
end

function Application:onPreInput(data) end

function Application:onInput(data)
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onInput')

    -- Immediately quit game without saving
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
        Window:setFullscreen(GameState.render.fullscreen, GameState.render.fullscreenExclusive);
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

    -- Preserving this in case we need to be able to automatically pause on window exit again
    -- TODO: Re-enable this and connect it to a Settings option for players who want this mode
    -- if Input:isPressed(Button.System.WindowLeave) and Config.getGameMode() ~= 1 then
    --     GameState.paused = true
    -- end

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

    --! why is this needed for the game to render and update lol
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

    do -- Exit
        self:onExit()
        -- Window:free()
    end
end

return Application
