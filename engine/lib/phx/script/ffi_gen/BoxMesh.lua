-- BoxMesh ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct BoxMesh {} BoxMesh;
    ]]

    return 1, 'BoxMesh'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local BoxMesh

    do -- C Definitions
        ffi.cdef [[
            void     BoxMesh_Free    (BoxMesh*);
            BoxMesh* BoxMesh_Create  ();
            void     BoxMesh_Add     (BoxMesh*, Vec3f const* p, Vec3f const* s, Vec3f const* r, Vec3f const* b);
            Mesh*    BoxMesh_GetMesh (BoxMesh const*, int res);
        ]]
    end

    do -- Global Symbol Table
        BoxMesh = {
            Create  = function()
                local _instance = libphx.BoxMesh_Create()
                return Core.ManagedObject(_instance, libphx.BoxMesh_Free)
            end,
        }

        if onDef_BoxMesh then onDef_BoxMesh(BoxMesh, mt) end
        BoxMesh = setmetatable(BoxMesh, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('BoxMesh')
        local mt = {
            __index = {
                add     = libphx.BoxMesh_Add,
                getMesh = function(self, res)
                    local _instance = libphx.BoxMesh_GetMesh(self, res)
                    return Core.ManagedObject(_instance, libphx.Mesh_Free)
                end,
            },
        }

        if onDef_BoxMesh_t then onDef_BoxMesh_t(t, mt) end
        BoxMesh_t = ffi.metatype(t, mt)
    end

    return BoxMesh
end

return Loader
