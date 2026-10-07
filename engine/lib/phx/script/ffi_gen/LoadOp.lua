-- AUTO GENERATED. DO NOT MODIFY!
-- LoadOp ----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef uint32 LoadOp;
    ]]

    return 2, 'LoadOp'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local LoadOp

    do -- C Definitions
        ffi.cdef [[
            cstr   LoadOp_ToString(LoadOp);
        ]]
    end

    do -- Global Symbol Table
        LoadOp = {
            Load     = 0,
            Clear    = 1,
            DontCare = 2,

            ToString = libphx.LoadOp_ToString,
        }

        if onDef_LoadOp then onDef_LoadOp(LoadOp, mt) end
        LoadOp = setmetatable(LoadOp, mt)
    end

    return LoadOp
end

return Loader
