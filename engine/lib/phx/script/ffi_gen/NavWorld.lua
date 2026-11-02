-- AUTO GENERATED. DO NOT MODIFY!
-- NavWorld --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct NavWorld {} NavWorld;
    ]]

    return 1, 'NavWorld'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local NavWorld

    do -- C Definitions
        ffi.cdef [[
            void      NavWorld_Free   (NavWorld*);
            NavWorld* NavWorld_Create ();
        ]]
    end

    do -- Global Symbol Table
        NavWorld = {
            Create = function()
                local _instance = libphx.NavWorld_Create()
                return Core.ManagedObject(_instance, libphx.NavWorld_Free)
            end,
        }

        if onDef_NavWorld then onDef_NavWorld(NavWorld, mt) end
        NavWorld = setmetatable(NavWorld, mt)
    end

    return NavWorld
end

return Loader
