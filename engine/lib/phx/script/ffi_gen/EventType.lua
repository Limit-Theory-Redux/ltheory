-- EventType -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 EventType;
    ]]

    return 2, 'EventType'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local EventType

    do -- C Definitions
        ffi.cdef [[
            EventType EventType_PreSim;
            EventType EventType_Sim;
            EventType EventType_PostSim;
            EventType EventType_PreRender;
            EventType EventType_Render;
            EventType EventType_PostRender;
            EventType EventType_PreInput;
            EventType EventType_Input;
            EventType EventType_PostInput;
            EventType EventType_EngineEventTypesCount;

            cstr      EventType_ToString(EventType);
        ]]
    end

    do -- Global Symbol Table
        EventType = {
            PreSim                = libphx.EventType_PreSim,
            Sim                   = libphx.EventType_Sim,
            PostSim               = libphx.EventType_PostSim,
            PreRender             = libphx.EventType_PreRender,
            Render                = libphx.EventType_Render,
            PostRender            = libphx.EventType_PostRender,
            PreInput              = libphx.EventType_PreInput,
            Input                 = libphx.EventType_Input,
            PostInput             = libphx.EventType_PostInput,
            EngineEventTypesCount = libphx.EventType_EngineEventTypesCount,

            ToString              = libphx.EventType_ToString,
        }

        if onDef_EventType then onDef_EventType(EventType, mt) end
        EventType = setmetatable(EventType, mt)
    end

    return EventType
end

return Loader
