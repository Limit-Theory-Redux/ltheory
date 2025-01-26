-- AUTO GENERATED. DO NOT MODIFY!
-- Renamed_Struct --------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'Renamed_Struct'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Renamed_Struct

    do -- C Definitions
        ffi.cdef [[
        ]]
    end

    do -- Global Symbol Table
        Renamed_Struct = {}

        if onDef_Renamed_Struct then onDef_Renamed_Struct(Renamed_Struct, mt) end
        Renamed_Struct = setmetatable(Renamed_Struct, mt)
    end

    return Renamed_Struct
end

return Loader
