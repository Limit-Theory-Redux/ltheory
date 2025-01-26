-- AUTO GENERATED. DO NOT MODIFY!
-- WorkerId --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint16 WorkerId;
    ]]

    return 2, 'WorkerId'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local WorkerId

    do -- C Definitions
        ffi.cdef [[
            WorkerId WorkerId_Echo;
            WorkerId WorkerId_EngineWorkersCount;

            cstr     WorkerId_ToString(WorkerId);
        ]]
    end

    do -- Global Symbol Table
        WorkerId = {
            Echo               = libphx.WorkerId_Echo,
            EngineWorkersCount = libphx.WorkerId_EngineWorkersCount,

            ToString           = libphx.WorkerId_ToString,
        }

        if onDef_WorkerId then onDef_WorkerId(WorkerId, mt) end
        WorkerId = setmetatable(WorkerId, mt)
    end

    return WorkerId
end

return Loader
