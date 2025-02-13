-- AUTO GENERATED. DO NOT MODIFY!
-- ShapeCastResult -------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct ShapeCastResult {
            RigidBody** hits;
            uint32      hits_len;
        } ShapeCastResult;
    ]]

    return 1, 'ShapeCastResult'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local ShapeCastResult

    do -- Global Symbol Table
        ShapeCastResult = {}

        if onDef_ShapeCastResult then onDef_ShapeCastResult(ShapeCastResult, mt) end
        ShapeCastResult = setmetatable(ShapeCastResult, mt)
    end

    return ShapeCastResult
end

return Loader
