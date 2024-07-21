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
    EngineInstance:exit()
end

function Application:eventLoop()
    if not self.eventsRegistered then
        self:registerEvents()
        self.eventsRegistered = true
    end

    local nextEvent = EventBusInstance:getNextEvent()
    while nextEvent ~= nil do
        --print("[" .. tostring(Render.ToString(nextEvent:getRender())) .. "]")
        --print("- Tunnel Id: " .. tostring(nextEvent:getTunnelId()))

        EventTunnels[nextEvent:getTunnelId()](nextEvent)
        nextEvent = EventBusInstance:getNextEvent()
    end
end

-- Application Template --------------------------------------------------------

function Application:appInit()
    self.eventsRegistered = false
    self.resX, self.resY = self:getDefaultSize()

    WindowInstance:setTitle(self:getTitle())
    WindowInstance:setCenteredPosition()
    WindowInstance:setSize(self.resX, self.resY)

    self.audio = Audio.Create()
    GameState.audio.manager = self.audio

    GameState.render.gameWindow = WindowInstance

    WindowInstance:setPresentMode(GameState.render.presentMode)

    if Config.jit.profile and Config.jit.profileInit then Jit.StartProfile() end

    Preload.Run()

    self:onInit()
    self:onResize(self.resX, self.resY)

    self.profilerFont = Font.Load('NovaMono', 10)
    self.lastUpdate = TimeStamp.Now() -- TODO: was TimeStamp.GetFuture(-1.0 / 60.0)

    if Config.jit.dumpasm then Jit.StartDump() end
    if Config.jit.profile and not Config.jit.profileInit then Jit.StartProfile() end
    if Config.jit.verbose then Jit.StartVerbose() end

    WindowInstance:cursor():setGrabMode(CursorGrabMode.Confined)
    WindowInstance:setCursorPosition(Vec2f(self.resX / 2, self.resY / 2))
    WindowInstance:cursor():setGrabMode(CursorGrabMode.None)

    self.profiling = false
    self.toggleProfiler = false
    self.showBackgroundModeHints = true
end

function Application:registerEvents()
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PreSim), self, self.onPreSim)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.Sim), self, self.onSim)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PostSim), self, self.onPostSim)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PreRender), self, self.onPreRender)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.Render), self, self.onRender)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PostRender), self, self.onPostRender)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PreInput), self, self.onPreInput)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.Input), self, self.onInput)
    EventBusInstance:subscribe(FrameStage.ToString(FrameStage.PostInput), self, self.onPostInput)

    local fakeEntity = { getGuid = function() return 0 end }
    EventBusInstance:register("MyCustomEvent", EventPriority.Medium, FrameStage.PreRender, false)
    EventBusInstance:subscribe("MyCustomEvent", fakeEntity,
        function() Log.Debug("\x1b[31mGot my event\x1b[0m") end)
    EventBusInstance:send("MyCustomEvent", fakeEntity)
end

function Application:onPreSim() end
function Application:onSim() end
function Application:onPostSim() end

function Application:onPreRender()
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

    if InputInstance:isDown(Bindings.TimeAccel) then
        self.timeScale = GameState.debug.timeAccelFactor
    end

    local now = TimeStamp.Now()
    self.dt = self.lastUpdate:getDifference(now)
    self.lastUpdate = now
    local timeScaledDt = self.timeScale * self.dt

    --* system & canvas should probably subscribe to onPreRender themselves
    if GameState.player.humanPlayer and GameState.player.humanPlayer:getRoot().update then
        GameState.player.humanPlayer:getRoot():update(timeScaledDt)
        GameState.render.uiCanvas:update(timeScaledDt)
    end

    do
        Profiler.SetValue('gcmem', GC.GetMemory())
        Profiler.Begin('App.onResize')
        local size = WindowInstance:size()
        WindowInstance:cursor():setGrabMode(CursorGrabMode.None)
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

function Application:onRender()
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onRender')

    WindowInstance:beginDraw()

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
            local s = string.format(
                '%.2f ms / %.0f fps / %.2f MB / %.1f K tris / %d draws / %d imms / %d swaps',
                1000.0 * self.dt,
                1.0 / self.dt,
                GC.GetMemory() / 1000.0,
                Metric.Get(Metric.TrisDrawn) / 1000,
                Metric.Get(Metric.DrawCalls),
                Metric.Get(Metric.Immediate),
                Metric.Get(Metric.FBOSwap))
            BlendMode.Push(BlendMode.Alpha)
            UI.DrawEx.SimpleRect(0, self.resY - 20, self.resX, self.resY, Color(0.1, 0.1, 0.1, 0.5))
            self.profilerFont:draw(s, 10, self.resY - 5, Color(1, 1, 1, 1))

            local y = self.resY - 5
            if self.profiling then
                self.profilerFont:draw('>> PROFILER ACTIVE <<', self.resX - 128, y, Color(1, 0, 0.15, 1))
                y = y - 12
            end
            BlendMode.Pop()
        end
    end
    Profiler.End()
    Profiler.LoopMarker()
end

function Application:onPostRender()
    do -- End Draw
        Profiler.SetValue('gcmem', GC.GetMemory())
        Profiler.Begin('App.onPostRender')
        WindowInstance:endDraw()
        Profiler.End()
    end
end

function Application:onPreInput() end

function Application:onInput()
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onInput')

    -- Immediately quit game without saving
    if InputInstance:isKeyboardAltPressed() and InputInstance:isPressed(Button.KeyboardQ) then self:quit() end
    if InputInstance:isPressed(Bindings.Exit) then self:quit() end

    if InputInstance:isPressed(Bindings.ToggleProfiler) then
        self.toggleProfiler = true
    end

    if InputInstance:isPressed(Bindings.Screenshot) then
        self.doScreenshot = true
        if Settings.exists('render.superSample') then
            self.prevSS = Settings.get('render.superSample')
        end
    end

    if InputInstance:isPressed(Bindings.ToggleFullscreen) then
        GameState.render.fullscreen = not GameState.render.fullscreen
        WindowInstance:setFullscreen(GameState.render.fullscreen, GameState.render.fullscreenExclusive);
    end

    if InputInstance:isPressed(Bindings.Reload) then
        Profiler.Begin('Engine.Reload')
        Cache.Clear()
        SendEvent('Engine.Reload')
        Preload.Run()
        Profiler.End()
    end

    if InputInstance:isPressed(Bindings.Pause) and GameState:GetCurrentState() == Enums.GameStates.InGame then
        if GameState.paused then
            GameState.paused = false
            if not GameState.panelActive and not GameState.debug.instantJobs then
                InputInstance:setCursorVisible(false)
            end
        else
            GameState.paused = true
            InputInstance:setCursorVisible(true)
        end
    end

    -- Preserving this in case we need to be able to automatically pause on window exit again
    -- TODO: Re-enable this and connect it to a Settings option for players who want this mode
    -- if InputInstance:isPressed(Button.System.WindowLeave) and Config.getGameMode() ~= 1 then
    --     GameState.paused = true
    -- end

    if not Gui:hasActiveInput() then
        if InputInstance:isPressed(Bindings.ToggleWireframe) then
            GameState.debug.physics.drawWireframe = not GameState.debug.physics.drawWireframe
        end

        if InputInstance:isPressed(Bindings.ToggleMetrics) then
            GameState.debug.metricsEnabled = not GameState.debug.metricsEnabled
        end

        if MainMenu.inBackgroundMode and InputInstance:isPressed(Bindings.ToggleHUD) then
            self.showBackgroundModeHints = not self.showBackgroundModeHints
        end
    end

    --! why is this needed for the game to render and update lol
    if GameState.render.uiCanvas ~= nil then
        GameState.render.uiCanvas:input()
    end

    Profiler.End()
end

function Application:onPostInput() end

function Application:doExit()
    if self.profiling then Profiler.Disable() end

    if Config.jit.dumpasm then Jit.StopDump() end
    if Config.jit.profile then Jit.StopProfile() end
    if Config.jit.verbose then Jit.StopVerbose() end

    do -- Exit
        self:onExit()
        -- WindowInstance:free()
    end
end

return Application
