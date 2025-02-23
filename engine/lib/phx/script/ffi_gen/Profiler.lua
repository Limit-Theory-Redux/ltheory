-- AUTO GENERATED. DO NOT MODIFY!
-- Profiler --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Profiler'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Profiler

    do -- C Definitions
        ffi.cdef [[
            void Profiler_Enable     ();
            void Profiler_Disable    ();
            void Profiler_Begin      (cstr name);
            void Profiler_End        ();
            void Profiler_SetValue   (cstr name, int value);
            void Profiler_LoopMarker ();
            void Profiler_Backtrace  ();
        ]]
    end

    do -- Global Symbol Table
        Profiler = {
            Enable     = libphx.Profiler_Enable,
            Disable    = libphx.Profiler_Disable,
            Begin      = libphx.Profiler_Begin,
            End        = libphx.Profiler_End,
            SetValue   = libphx.Profiler_SetValue,
            LoopMarker = libphx.Profiler_LoopMarker,
            Backtrace  = libphx.Profiler_Backtrace,
        }

        if onDef_Profiler then onDef_Profiler(Profiler, mt) end
        Profiler = setmetatable(Profiler, mt)
    end

    return Profiler
end

return Loader
