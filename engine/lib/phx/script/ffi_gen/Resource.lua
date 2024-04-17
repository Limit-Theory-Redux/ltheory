-- Resource --------------------------------------------------------------------

local Loader = {}

function Loader.declareType()
    return 0, 'Resource'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Resource

    do -- C Definitions
        ffi.cdef [[
            bool   Resource_Exists     (ResourceType ty, cstr name);
            cstr   Resource_GetPath    (ResourceType ty, cstr name);
            Bytes* Resource_LoadBytes  (ResourceType ty, cstr name);
            cstr   Resource_LoadString (ResourceType ty, cstr name);
        ]]
    end

    do -- Global Symbol Table
        Resource = {
            Exists     = libphx.Resource_Exists,
            GetPath    = libphx.Resource_GetPath,
            LoadBytes  = function(...)
                local instance = libphx.Resource_LoadBytes(...)
                return Core.ManagedObject(instance, libphx.Bytes_Free)
            end,
            LoadString = libphx.Resource_LoadString,
        }

        if onDef_Resource then onDef_Resource(Resource, mt) end
        Resource = setmetatable(Resource, mt)
    end

    return Resource
end

return Loader
