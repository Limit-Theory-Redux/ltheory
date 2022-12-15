local UniversalBindings = require('Systems.Controls.Bindings.UniversalBindings')
local StateMachine = class(function (self) end)

--[[
State Machine Variables

-- Core Variables
state[]         stateStack          Stack of all current states. Top is current.
state           loadingState        State that runs inbetween other states for Loading purposes.
bool            exit                Default: false. If exiting program.


-- Window Variables
Window          window              Window information
int             resolutionX         Default: getDefaultSize()[0]. X-axis size of Resolution
int             resolutionY         Default: getDefaultSize()[1]. Y-axis size of Resolution

-- Time Variables
TimeStamp       lastUpdate          Default: TimeStamp.GetFuture(-1.0 / 60.0). Time of last update.
TimeStamp       deltaTime           Difference between lastUpdate and current Time
int             timeScale           Default: 1.0 Used for speeding up and slowing down time.

-- Profiler Variables
bool            profiling           Default: false. If currently Profiling.
bool            toggleProfiler      Default: false. Toggle Profiler On and Off.

-- Other Variables
Font            font                Current Font.
superSample     prevScreenShot      render.superSample to get Screenshot of Window
bool            doScreenshot        Default: false. Triggers a Screenshot.    

]]--

--[[
State Machine Functions

-- Core Functions
init            (string initialStateName)   return nil
    -- Initialize Variable Defaults. Create Window. Configure JIT. Run Preload. Call run.
run             ()                          return nil
    -- Function that Loops until exited. Every Loop is a Frame. Calls exit() when exit is true.

    resize          ()                          return nil
        -- Called Every Frame. Detects Resize --? Nothing is ever done with this.
    input           ()                          return nil
        -- Called Every Frame. Get User Input

    update          ()                          return nil
        -- Called Every Frame. General use Update function

    lateUpdate      ()                          return nil
        -- Called Every Frame after Update(). 

    fixedUpdate     ()                          return nil
        -- Called Every x Frames. Used for Update that are not time sensitive.

    draw            ()                          return nil
        -- Called Every Frame. Render scene.

    printScreenshot ()                          return nil
        -- Called if toggled. Print Screenshot to settings. (Functionality unclear)

    drawMetrics     ()                          return nil
        -- Called Every Frame if Toggled. Renders Metrics to bottom of Screen

    endDraw         ()                          return nil
        -- Called Every Frame. Window Buffer Swap

quit            ()                          return nil
    -- set exit to true.

-- StateStack Functions
pushState       (State newState)            return nil
popState        ()                          return (State prevState)
peekState       ()                          return (State currentState)
findState       (string stateName)          return State
    -- FindState Based on stateName 
findState       (stateName, enum stateType) return State
    -- FindState Based on stateName and stateType

-- Window Functions
getDefaultSize  ()                          return (int x, int y)
getTitle        ()                          return (string title)
getWindowMode   ()                          return (Bit windowMode)

]]--

-- Window Functions
--TODO: Allow for Modifiable Size
function StateMachine:getDefaultSize ()
    return 1600, 900
end
--TODO: Allow for Modifiable Title
function StateMachine:getTitle ()
    return 'Pheonix Engine Application'
end

function StateMachine:getWindowMode ()
    return Bit.Or32(WindowMode.Shown, WindowMode.Resizable)
end

-- StateStack Functions
function StateMachine:pushState(nextState)
    self:peekState().onDisable()            -- Disable Previous State
    --TODO: Condition for loadingState if defined.
    self.stateStack[#self.stateStack+1] = nextState
    nextState.onInit()                      -- Initialize Next State
end
--? Maybe utilize an exitFinished flag or function before enabling the next state on the stack
function StateMachine:popState()
    if #self.stateStack > 0 then
        local poppedState = table.remove(self.stateStack, #self.stateStack)
        poppedState.onExit()                    -- Exit PoppedState
        self:peekState().onEnable()             -- Enable Next State on Stack
    else
        self:quit()
end
function StateMachine:peekState()
    return self.stateStack[#self.stateStack]
end
--TODO: Implement findState
function StateMachine:findState(stateName)
end
function StateMachine:findState(stateName, stateType)
end

-- Core Functions

function StateMachine:init(initialStateName)
    -- Creating Window
    self.resolutionX, self.resolutionY = self:getDefaultSize()
    self.window = Window.Create(
        self:getTitle(),
        WindowPos.Default,
        WindowPos.Default,
        self.resolutionX,
        self.resolutionY,
        self:getWindowMode())
    self.window:setVsync(Config.render.vsync)

    -- Start JIT profile
    if Config.jit.profile and Config.jit.profileInit then Jit.StartProfile() end

    Preload.Run()

    -- Configuring JIT Settings
    if Config.jit.dumpasm then Jit.StartDump() end
    if Config.jit.profile and not Config.jit.profileInit then Jit.StartProfile() end
    if Config.jit.verbose then Jit.StartVerbose() end

    -- Initializing Default Variables
    self.profiling = false
    self.toggleProfiler = false
    self.exit = false
    Input.LoadGamepadDatabase('gamecontrollerdb_205.txt'); --TODO: Research what this is doing.
    self.font = Font.Load('NovaMono', 10) --TODO: Allow for Font Modifications in Config
    self.lastUpdate = TimeStamp.GetFuture(-1.0 / 60.0)
    self.timeScale = 1.0
    self.doScreenshot = false
    
    -- Initialize stateStack, findState, and push it onto the stack.
    self.stateStack = {}
    local initialState = findState(initialStateName)
    self:pushState(initialState)

    -- Start Run Loop.
    self:run()
end

function StateMachine:run()
    while not self.exit do
        --? Is there a smarter way to do this?
        if toggleProfiler then
            toggleProfiler = false
            profiling = not profiling
            if profiling then Profiler.Enable() else Profiler.Disable() end 
        end

        Profiler.SetValue('gcmem', GC.GetMemory())
        Profiler.Begin('Frame')
        Engine.Update()

        self:resize()

        self:input()

        self:update()

        self:lateUpdate()

        --TODO: Implement fixedUpdate, Find the right amount of time between fixedUpdates.

        self:draw()

        self:printScreenshot()

        self:drawMetrics()

        self:endDraw()

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

function StateMachine:resize() 
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onResize')
    local size = self.window:getSize()
    if size.x ~= self.resolutionX or size.y ~= self.resolutionY then
        self.resolutionX = size.x
        self.resolutionY = size.y
        self:peekState().onResize(self.resolutionX, self.resolutionY)
    end
    Profiler.End()
end

function StateMachine:input()
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onInput')

    --TODO: Fix Application Bindings to be actual Controls not manually defined bindings
    -- Exit Game Binding
    if UniversalBindings.Exit:get() > 0 then self:quit() end

    if UniversalBindings.ProfilerToggle:get() > 0 then 
        toggleProfiler = true
    end

    if UniversalBindings.Screenshot:get() > 0 then
        self.doScreenshot = true
        if Settings.exists('render.superSample') then
            self.prevScreenShot = Settings.get('render.superSample')
            Settings.set('render.superSample', 2)
        end
    end

    if UniversalBindings.ToggleFullscreen:get() > 0 then
        self.window:toggleFullscreen()
    end

    if UniversalBindings.Reload:get() > 0 then
        Profiler.Begin('Engine.Reload')
        Cache.Clear()
        SendEvent('Engine.Reload')
        Preload.Run()
        Profiler.End()
    end

    if UniversalBindings.TimeAccel then
        timeScale = Config.debug.timeAccelFactor
    end

    if UniversalBindings.ToggleWireframe then
        Settings.set('render.wireframe', not Settings.get('render.wireframe'))
    end

    self:peekState().onInput()
    Profiler.End()
end

function StateMachine:update()
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onUpdate')
    local now = TimeStamp.Get()
    self.deltaTime = TimeStamp.GetDifference(self.lastUpdate, now)
    self.lastUpdate = now
    self:peekState().onUpdate(self.timeScale * self.deltaTime)
    Profiler.End()
end

function StateMachine:lateUpdate()
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onUpdate')
    local now = TimeStamp.Get()
    self.deltaTime = TimeStamp.GetDifference(self.lastUpdate, now)
    self.lastUpdate = now
    self:peekState().onLateUpdate(self.timeScale * self.deltaTime)
    Profiler.End()
end

function StateMachine:draw()
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.onDraw')
    self.window:beginDraw()
    self:onDraw()
    Profiler.End()
end

function StateMachine:printScreenshot()
    ScreenCap()
    if self.prevScreenShot then
        Settings.set('render.superSample', self.prevScreenShot)
        self.prevScreenShot = nil
    end
end

function StateMachine:drawMetrics()
    if Config.debug.metrics then -- Metrics Display
        local str = string.format(
            '%.2f ms / %.0f fps / %.2f MB / %.1f K tris / %d draws / %d imms / %d swaps',
            1000.0 * self.deltaTime,
            1.0 / self.deltaTime,
            GC.GetMemory() / 1000.0,
            Metric.Get(Metric.TrisDrawn) / 1000,
            Metric.Get(Metric.DrawCalls),
            Metric.Get(Metric.Immediate),
            Metric.Get(Metric.FBOSwap))
        BlendMode.Push(BlendMode.Alpha)
        Draw.Color(0.1, 0.1, 0.1, 0.5)
        Draw.Rect(0, self.resolutionY - 20, self.resolutionX, self.resolutionY)
        self.font:draw(str, 10, self.resolutionY - 5, 1, 1, 1, 1)

        local y = self.resolutionY - 5
        if profiling then
            self.font:draw('>> PROFILER ACTIVE <<', self.resolutionX - 128, y, 1, 0, 0.15, 1)
            y = y - 12
        end
        BlendMode.Pop()
    end
end

function StateMachine:endDraw()
    Profiler.SetValue('gcmem', GC.GetMemory())
    Profiler.Begin('App.SwapBuffers')
    self.window:endDraw()
    Profiler.End()
end

return StateMachine