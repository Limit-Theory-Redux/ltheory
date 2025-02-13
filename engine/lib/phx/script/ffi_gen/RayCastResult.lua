-- AUTO GENERATED. DO NOT MODIFY!
-- RayCastResult ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct RayCastResult {
            RigidBody* body;
            float      normx;
            float      normy;
            float      normz;
            double     posx;
            double     posy;
            double     posz;
            float      t;
        } RayCastResult;
    ]]

    return 1, 'RayCastResult'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local RayCastResult

    do -- Global Symbol Table
        RayCastResult = {}

        if onDef_RayCastResult then onDef_RayCastResult(RayCastResult, mt) end
        RayCastResult = setmetatable(RayCastResult, mt)
    end

    return RayCastResult
end

return Loader
