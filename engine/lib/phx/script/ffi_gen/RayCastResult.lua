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

    do -- C Definitions
        ffi.cdef [[
            void           RayCastResult_Free   (RayCastResult*);
            RayCastResult* RayCastResult_Create ();
        ]]
    end

    do -- Global Symbol Table
        RayCastResult = {
            Create = function()
                local _instance = libphx.RayCastResult_Create()
                return Core.ManagedObject(_instance, libphx.RayCastResult_Free)
            end,
        }

        if onDef_RayCastResult then onDef_RayCastResult(RayCastResult, mt) end
        RayCastResult = setmetatable(RayCastResult, mt)
    end

    return RayCastResult
end

return Loader
