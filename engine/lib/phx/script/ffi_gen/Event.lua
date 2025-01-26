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
            Event Event_PreSim;
            Event Event_Sim;
            Event Event_PostSim;
            Event Event_PreRender;
            Event Event_Render;
            Event Event_PostRender;
            Event Event_PreInput;
            Event Event_Input;
            Event Event_PostInput;
            Event Event_EngineEventsCount;

            cstr  Event_ToString(Event);
        ]]
    end

    do -- Global Symbol Table
        Event = {
            PreSim            = libphx.Event_PreSim,
            Sim               = libphx.Event_Sim,
            PostSim           = libphx.Event_PostSim,
            PreRender         = libphx.Event_PreRender,
            Render            = libphx.Event_Render,
            PostRender        = libphx.Event_PostRender,
            PreInput          = libphx.Event_PreInput,
            Input             = libphx.Event_Input,
            PostInput         = libphx.Event_PostInput,
            EngineEventsCount = libphx.Event_EngineEventsCount,

            ToString          = libphx.Event_ToString,
        }

        if onDef_Event then onDef_Event(Event, mt) end
        Event = setmetatable(Event, mt)
    end

    return Event
end

return Loader
