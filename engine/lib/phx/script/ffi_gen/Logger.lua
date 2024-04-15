-- Logger ----------------------------------------------------------------------

---@class Logger
---@field Trace fun(msg: string)
---@field Debug fun(msg: string)
---@field Info fun(msg: string)
---@field Warn fun(msg: string)
---@field Error fun(msg: string)

local Loader = {}

function Loader.declareType()
    return 0, 'Logger'
end

function Loader.defineType()
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
            ---@param msg string
            Trace = libphx.Logger_Trace,
            ---@param msg string
            Debug = libphx.Logger_Debug,
            ---@param msg string
            Info  = libphx.Logger_Info,
            ---@param msg string
            Warn  = libphx.Logger_Warn,
            ---@param msg string
            Error = libphx.Logger_Error,
        }

        if onDef_Logger then onDef_Logger(Logger, mt) end
        Logger = setmetatable(Logger, mt)
    end

    return Logger
end

return Loader
