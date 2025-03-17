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

    do -- C Definitions
        ffi.cdef [[
            void             ShapeCastResult_Free   (ShapeCastResult*);
            ShapeCastResult* ShapeCastResult_Create ();
        ]]
    end

    do -- Global Symbol Table
        ShapeCastResult = {
            Create = function()
                local _instance = libphx.ShapeCastResult_Create()
                return Core.ManagedObject(_instance, libphx.ShapeCastResult_Free)
            end,
        }

        if onDef_ShapeCastResult then onDef_ShapeCastResult(ShapeCastResult, mt) end
        ShapeCastResult = setmetatable(ShapeCastResult, mt)
    end

    return ShapeCastResult
end

return Loader
