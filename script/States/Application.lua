local Bindings = require('States.ApplicationBindings')

local Application = class(function (self) end)

-- Virtual ---------------------------------------------------------------------

function Application:getDefaultSize ()
--  return 1600, 900
  return Config.render.defaultResX, Config.render.defaultResY
end

function Application:getTitle () return
  Config.gameTitle
end

function Application:getWindowMode ()
  return Bit.Or32(WindowMode.Shown, WindowMode.Resizable)
end

function Application:onInit         ()       end
function Application:onDraw         ()       end
function Application:onResize       (sx, sy) self.window:setMousePosition(self.resX / 2, self.resY / 2) end
function Application:onUpdate       (dt)     end
function Application:onExit         ()       end
function Application:onInput        ()       end

function Application:quit ()
  self.exit = true
end

-- Application Template --------------------------------------------------------

function Application:run ()
  self.resX, self.resY = self:getDefaultSize()
  self.window = Window.Create(
    self:getTitle(),
    WindowPos.Default,
    WindowPos.Default,
    self.resX,
    self.resY,
    self:getWindowMode())

  GameState.render.gameWindow = self.window

  self.exit = false

  self.window:setVsync(GameState.render.vsync)

  if Config.jit.profile and Config.jit.profileInit then Jit.StartProfile() end

  Preload.Run()

  Input.LoadGamepadDatabase('gamecontrollerdb_205.txt');
  self:onInit()
  self:onResize(self.resX, self.resY)

  local font = Font.Load('NovaMono', 10)
  self.lastUpdate = TimeStamp.GetFuture(-1.0 / 60.0)

  if Config.jit.dumpasm then Jit.StartDump() end
  if Config.jit.profile and not Config.jit.profileInit then Jit.StartProfile() end
  if Config.jit.verbose then Jit.StartVerbose() end

  self.window:setWindowGrab(true)
  self.window:setMousePosition(self.resX / 2, self.resY / 2)
  self.window:setWindowGrab(false)

  local profiling = false
  local toggleProfiler = false
  while not self.exit do
    if toggleProfiler then
      toggleProfiler = false
      profiling = not profiling
      if profiling then Profiler.Enable() else Profiler.Disable() end
    end

    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('Frame')
    Engine.Update()

    do
      Profiler.SetValue('gcmem', GC.GetMemory())
      Profiler.Begin('App.onResize')
      local size = self.window:getSize()
      self.window:setWindowGrab(false)
      if size.x ~= self.resX or size.y ~= self.resY then
        self.resX = size.x
        self.resY = size.y
        if not GameState.render.fullscreen then
          GameState.render.resX = self.resX
          GameState.render.resY = self.resY
        end
        self:onResize(self.resX, self.resY)
      end
      Profiler.End()
    end

    local timeScale = 1.0
    local doScreenshot = false

    do
      Profiler.SetValue('gcmem', GC.GetMemory())
      Profiler.Begin('App.onInput')

      -- Immediately quit game without saving
      if Input.GetKeyboardCtrl() and Input.GetPressed(Button.Keyboard.W) then self:quit() end
      if Input.GetKeyboardAlt()  and Input.GetPressed(Button.Keyboard.Q) then self:quit() end
      if Input.GetPressed(Bindings.Exit) then self:quit() end

      if Input.GetPressed(Bindings.ToggleProfiler) then
        toggleProfiler = true
      end

      if Input.GetPressed(Bindings.Screenshot) then
        doScreenshot = true
        if Settings.exists('render.superSample') then
          self.prevSS = Settings.get('render.superSample')
--          Settings.set('render.superSample', 2)
        end
      end

      if Input.GetPressed(Bindings.ToggleFullscreen) then
        self.window:toggleFullscreen()
        GameState.render.fullscreen = not GameState.render.fullscreen
      end

      if Input.GetPressed(Bindings.Reload) then
        Profiler.Begin('Engine.Reload')
        Cache.Clear()
        SendEvent('Engine.Reload')
        Preload.Run()
        Profiler.End()
      end

      if Input.GetPressed(Bindings.Pause) and GameState:GetCurrentState() == Enums.GameStates.InGame then
        if GameState.paused then
          GameState.paused = false
          if not GameState.panelActive then
            Input.SetMouseVisible(false)
          end
        else
          GameState.paused = true
          Input.SetMouseVisible(true)
        end
      end

      -- Preserving this in case we need to be able to automatically pause on window exit again
      -- TODO: Re-enable this and connect it to a Settings option for players who want this mode
--      if Input.GetPressed(Button.System.WindowLeave) and Config.getGameMode() ~= 1 then
--        Config.game.gamePaused = true
--      end

      if GameState.paused then
        timeScale = 0.0
      else
        timeScale = 1.0
      end

      if Input.GetDown(Bindings.TimeAccel) then
        timeScale = GameState.debug.timeAccelFactor
      end

      if Input.GetPressed(Bindings.ToggleWireframe) then
        Settings.set('render.wireframe', not Settings.get('render.wireframe'))
      end

      if Input.GetPressed(Bindings.ToggleMetrics) then
        GameState.debug.metricsEnabled = not GameState.debug.metricsEnabled
      end

      self:onInput()
      Profiler.End()
    end

    do
      Profiler.SetValue('gcmem', GC.GetMemory())
      Profiler.Begin('App.onUpdate')
      local now = TimeStamp.Get()
      self.dt = TimeStamp.GetDifference(self.lastUpdate, now)
      self.lastUpdate = now
      self:onUpdate(timeScale * self.dt)
      Profiler.End()
    end

    do
      Profiler.SetValue('gcmem', GC.GetMemory())
      Profiler.Begin('App.onDraw')
      self.window:beginDraw()
      self:onDraw()
      Profiler.End()
    end

    if GameState:GetCurrentState() ~= Enums.GameStates.MainMenu then
      UI.DrawEx.TextAdditive(
        'NovaRound',
        "EXPERIMENTAL BUILD - NOT FINAL!",
        20,
        self.resX / 2 - 24, 62, 40, 20,
        1, 1, 1, 1,
        0.5, 0.5
      )

      if GameState.paused then
        UI.DrawEx.TextAdditive(
          'NovaRound',
          "[PAUSED]",
          24,
          0, 0, self.resX, self.resY,
          1, 1, 1, 1,
          0.5, 0.99
        )
      end
    end

    -- Take screenshot AFTER on-screen text is shown but BEFORE metrics are displayed
    if doScreenshot then
      ScreenCap()
      if self.prevSS then
--        Settings.set('render.superSample', self.prevSS)
        self.prevSS = nil
      end
    end

    do -- Metrics display
      if GameState.debug.metricsEnabled then -- Metrics Display
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
        Draw.Color(0.1, 0.1, 0.1, 0.5)
        Draw.Rect(0, self.resY - 20, self.resX, self.resY)
        font:draw(s, 10, self.resY - 5, 1, 1, 1, 1)

        local y = self.resY - 5
        if profiling then
          font:draw('>> PROFILER ACTIVE <<', self.resX - 128, y, 1, 0, 0.15, 1)
          y = y - 12
        end
        BlendMode.Pop()
      end
    end

    do -- End Draw
      Profiler.SetValue('gcmem', GC.GetMemory())
      Profiler.Begin('App.SwapBuffers')
      self.window:endDraw()
      Profiler.End()
    end
    Profiler.End()
    Profiler.LoopMarker()
  end

  if profiling then Profiler.Disable() end

  if Config.jit.dumpasm then Jit.StopDump() end
  if Config.jit.profile then Jit.StopProfile() end
  if Config.jit.verbose then Jit.StopVerbose() end

  do -- Exit
    self:onExit()
    self.window:free()
  end
end

return Application
