-- Worker ----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint8 Worker;
    ]]

    return 2, 'Worker'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Worker

    do -- C Definitions
        ffi.cdef [[
            Worker Worker_Echo;
            Worker Worker_EngineWorkersCount;

            cstr   Worker_ToString(Worker);
        ]]
    end

    do -- Global Symbol Table
        Worker = {
            Echo               = libphx.Worker_Echo,
            EngineWorkersCount = libphx.Worker_EngineWorkersCount,

            ToString           = libphx.Worker_ToString,
        }

        if onDef_Worker then onDef_Worker(Worker, mt) end
        Worker = setmetatable(Worker, mt)
    end

    return Worker
end

return Loader
