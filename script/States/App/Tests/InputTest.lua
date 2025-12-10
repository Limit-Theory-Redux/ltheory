local Application         = require('States.Application')

local InputTest           = Subclass("InputTest", Application)

--- /////////// Imports /////////// ---
-- core/modules
local GC                  = require("Core.Util.GC")
local Cache               = require("Render.Cache")
local RenderCoreSystem    = require("Modules.Rendering.Systems.RenderCoreSystem")
local DeltaTimer          = require("Shared.Tools.DeltaTimer")
local DrawEx              = require("UI.DrawEx")

function InputTest:onInit()

    -- window setup
    Window:setFullscreen(false, true)

    -- start fps timer
    self.timer = DeltaTimer("InputTest")
    self.timer:start("fps", 0.1)
    self.frameCount = 0
    self.smoothFPS = 0
    self.fpsText = "FPS: 0"
    self.time = 0

    
    
    -- subscribe to app hooks
    EventBus:subscribe(Event.PreRender, self, self.onStatePreRender)
    EventBus:subscribe(Event.Input, self, self.onStateInput)
end

function InputTest:onPreRender(data)
    
end

function InputTest:onRender(data)
    RenderCoreSystem:render(data)
    


end

function InputTest:onStateInput(data)

end


return InputTest