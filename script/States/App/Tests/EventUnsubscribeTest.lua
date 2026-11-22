--[[
Event unsubscribe test: a handler that unsubscribes other handlers in the
middle of a frame. EventBus:unsubscribe drops the Lua handler at once, while
the bus still delivers the rest of that frame's events to the subscriber; the
event loop must skip those events instead of calling a handler that is gone.

On its first call, a PreRender handler unsubscribes two others: one later in
the same stage and one in a later stage (Render). It then subscribes a new
Render handler in their place, the way a system that is disabled and enabled
again in one frame does. The removed handlers must never run, the new one must
run once per frame from the next frame on, and the frames after that must run
as usual.

Run it from the repository root:

    cargo run -p ltr -- EventUnsubscribeTest

Exit code 0 and "EventUnsubscribeTest: passed" = passed. A failure stops the
game with its error and traceback and exit code 1 (`Core.Call`).
]]

local Application = require('States.Application')

---@class EventUnsubscribeTest: Application
local EventUnsubscribeTest = Subclass("EventUnsubscribeTest", Application)

-- Frames to run after the one that unsubscribes
local FRAMES_AFTER = 5

function EventUnsubscribeTest:onInit()
    self.frame = 0
    self.newHandlerCalls = 0
    self.passed = false

    -- The bus calls the subscribers of a stage in the order they subscribed:
    -- the unsubscribing handler runs before the other PreRender handler.
    EventBus:subscribe(Event.PreRender, nil, function() self:onUnsubscriberPreRender() end)
    self.sameStageTunnel = EventBus:subscribe(Event.PreRender, nil, function() self:removedHandlerCalled('PreRender') end)
    self.laterStageTunnel = EventBus:subscribe(Event.Render, nil, function() self:removedHandlerCalled('Render') end)
end

function EventUnsubscribeTest:onUnsubscriberPreRender()
    -- The engine still runs the frame after the one that quits
    if self.passed then return end
    self.frame = self.frame + 1

    if self.frame == 1 then
        EventBus:unsubscribe(self.sameStageTunnel)
        EventBus:unsubscribe(self.laterStageTunnel)
        -- The bus adds a subscriber at the start of the next frame
        EventBus:subscribe(Event.Render, nil, function() self.newHandlerCalls = self.newHandlerCalls + 1 end)
        return
    end

    -- The new handler runs in Render, after this stage: once in each frame
    -- from the second up to the previous one
    local expectedCalls = self.frame - 2
    if self.newHandlerCalls ~= expectedCalls then
        error(string.format('EventUnsubscribeTest: the new Render handler ran %d times by frame %d, expected %d',
            self.newHandlerCalls, self.frame, expectedCalls))
    end

    if self.frame > FRAMES_AFTER then
        self.passed = true
        Log.Info('EventUnsubscribeTest: passed')
        self:quit()
    end
end

---@param stage string
function EventUnsubscribeTest:removedHandlerCalled(stage)
    error(string.format('EventUnsubscribeTest: the %s handler ran after it unsubscribed', stage))
end

return EventUnsubscribeTest
