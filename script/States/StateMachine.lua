
local StateMachine = class(function (self) end)

--[[
State Machine Variables

-- Core Variables
state[]         stateStack          Stack of all current states. Top is current.
bool            exit                Default: false. If exiting program.


-- Window Variables
Window          window              Window information
int             resolutionX         Default: getDefaultSize()[0]. X-axis size of Resolution
int             resolutionY         Default: getDefaultSize()[1]. Y-axis size of Resolution
Size            size                Size of Window. Derived from window:getSize()

-- Time Variables
TimeStamp       lastUpdate          Default: TimeStamp.GetFuture(-1.0 / 60.0). Time of last update.
TimeStamp       deltaTime           Difference between lastUpdate and current Time
int             timeScale           Used for speeding up and slowing down time.

-- Profiler Variables
bool            profiling           Default: false. If currently Profiling.
bool            toggleProfiler      Default: false. Toggle Profiler On and Off.

-- Other Variables
Font            font                Current Font.
superSample     prevScreenShot      render.superSample to get Screenshot of Window

]]--

--[[
State Machine Functions

-- Core Functions
init            (state initialState)        return nil
    -- Initialize Variable Defaults. Create Window. Configure JIT. Run Preload. Call run.
run             ()                          return nil
    -- Function that Loops until exited. Every Loop is a Frame. Calls exit() when exit is true.

    input           ()                          return nil
        -- Called Every Frame. Get User Input

    update          (TimeStamp deltaTime)       return nil
        -- Called Every Frame. General use Update function

    lateUpdate      (TimeStamp deltaTime)       return nil
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

exit            ()                          return nil
    -- Stop Profiler, Stop JIT, do State Exit, window:free()

quit            ()                          return nil
    -- set exit to true.
    
-- Stack Functions
pushState       (newState)                  return nil
popState        ()                          return (state prevState)
peekState       ()                          return (state currentState)

-- Window Functions
getDefaultSize  ()                          return (int x, int y)
getTitle        ()                          return (string title)
getWindowMode   ()                          return (Bit windowMode)
resize          (int SizeX, int SizeY)      return nil

]]--