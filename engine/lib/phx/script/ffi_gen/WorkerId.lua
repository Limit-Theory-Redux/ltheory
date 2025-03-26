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
            cstr     WorkerId_ToString(WorkerId);
        ]]
    end

    do -- Global Symbol Table
        WorkerId = {
            Echo               = 0,
            EngineWorkersCount = 1,

            ToString           = libphx.WorkerId_ToString,
        }

        if onDef_WorkerId then onDef_WorkerId(WorkerId, mt) end
        WorkerId = setmetatable(WorkerId, mt)
    end

    return WorkerId
end

return Loader
