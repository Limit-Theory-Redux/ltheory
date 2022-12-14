
local State = class(function (self) end)

--[[
State Variables

-- Static State Type Enum
StateType = {
    TEST        = 0, -- Currently in a Test State
    MENU        = 1, -- General Menu State. I.E. Main Menu, Overlay Menu, etc.
    FLIGHT      = 2, -- Player is Flying in Space
    COMMAND     = 3, -- Player is in Command View
}

-- Core Variables
StateMachine    stateMachine        Reference to StateMachine.
StateType       stateType           Type of Current State.
string          stateName           Name of Current State. (Potentially redundant.)
table           stateSettings       Nested Table Used for all Table Settings. 
    -- Format is State Dependent. Please Define Settings Format in StateFile.
table           allControls         Reference to all Controllers added to the scene and if they are active.
    -- Format: [string "ControlName": [Control controlVar, bool active] ]
]]--

--[[
State Functions

-- Core Functions
onInit          ()                          return nil
    -- On State Initialization. Calls onInitialized().
onInitialized   ()                          return nil
    -- Called when State is Initialized.
onExit          ()                          return nil
    -- Called when State is Exited.
onEnable        ()                          return nil
    -- Called when State is Enabled.
onDisable       ()                          return nil
    -- Called when State is Disabled.
toggleControls  (table controls)            return ()
    -- controls Format: {[string "ControlName", bool setActive], [string "ControlName2", bool setActive]}
    -- Toggles all Controls in 'controls' on or off depending on their related setActive.
-- 

-- State Machine Run Functions
onInput         ()                          return nil
    -- Called Every Frame. Get User Input
onUpdate        (TimeStamp deltaTime)       return nil
    -- Called Every Frame. General Update
onLateUpdate    (TimeStamp deltaTime)       return nil
    -- Called Every Frame after Update(). 
onFixedUpdate   ()                          return nil
    -- Called Every x Frames. Used for Updates that are not time sensitive.
onDraw          ()                          return nil
    -- Called Every Frame. Render State Scene.
]]--