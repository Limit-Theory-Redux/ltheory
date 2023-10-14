-- Logger ----------------------------------------------------------------------

function declareType()
    return 0, 'Logger'
end

function defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Logger

    do -- C Definitions
        ffi.cdef [[
            void Logger_Trace (cstr msg);
            void Logger_Debug (cstr msg);
            void Logger_Info  (cstr msg);
            void Logger_Warn  (cstr msg);
            void Logger_Error (cstr msg);
        ]]
    end

    do -- Global Symbol Table
        Logger = {
            Trace = libphx.Logger_Trace,
            Debug = libphx.Logger_Debug,
            Info  = libphx.Logger_Info,
            Warn  = libphx.Logger_Warn,
            Error = libphx.Logger_Error,
        }

        if onDef_Logger then onDef_Logger(Logger, mt) end
        Logger = setmetatable(Logger, mt)
    end

    return Logger
end

