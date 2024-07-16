--[[----------------------------------------------------------------------------
                  Message logging functions.
----------------------------------------------------------------------------]]
--
local Log = {}

-- Lowlevel tracing information
function Log.Trace(...)
    Logger.Trace(format("[lua] %s", format(...)))
end

-- General information
function Log.Debug(...)
    Logger.Debug(format("[lua] %s", format(...)))
end

-- General information
function Log.Info(...)
    Logger.Info(format("[lua] %s", format(...)))
end

-- Recoverable issue
function Log.Warn(...)
    Logger.Warn(format("[lua] %s", format(...)))
end

-- Hard error (non-recoverable)
function Log.Error(...)
    local pre = '\x1B[43m \x1B[0m \x1B[33;1mError: '
    local app = '\x1B[0m'
    local msg = format(...)

    Logger.Error(format("[lua] %s", msg))

    error(pre .. msg .. app)
end

return Log
