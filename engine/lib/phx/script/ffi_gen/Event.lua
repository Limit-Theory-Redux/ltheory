-- AUTO GENERATED. DO NOT MODIFY!
-- Event -----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 Event;
    ]]

    return 2, 'Event'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Event

    do -- C Definitions
        ffi.cdef [[
            cstr  Event_ToString(Event);
        ]]
    end

    do -- Global Symbol Table
        Event = {
            PreSim            = 0,
            Sim               = 1,
            PostSim           = 2,
            PreRender         = 3,
            Render            = 4,
            PostRender        = 5,
            PreInput          = 6,
            Input             = 7,
            PostInput         = 8,
            EngineEventsCount = 9,

            ToString          = libphx.Event_ToString,
        }

        if onDef_Event then onDef_Event(Event, mt) end
        Event = setmetatable(Event, mt)
    end

    return Event
end

return Loader
