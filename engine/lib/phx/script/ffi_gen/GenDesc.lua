-- AUTO GENERATED. DO NOT MODIFY!
-- GenDesc ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct GenDesc {} GenDesc;
    ]]

    return 1, 'GenDesc'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local GenDesc

    do -- C Definitions
        ffi.cdef [[
            void     GenDesc_Free   (GenDesc*);
            GenDesc* GenDesc_Create (Shader const* shader);
            void     GenDesc_Label  (GenDesc*, cstr label);
            void     GenDesc_Params (GenDesc*, uint8 const* bytes, uint64 bytes_size);
            void     GenDesc_Input  (GenDesc*, int slot, TexView const* view, uint32 sampler);
        ]]
    end

    do -- Global Symbol Table
        GenDesc = {
            Create = function(shader)
                local _instance = libphx.GenDesc_Create(shader)
                return Core.ManagedObject(_instance, libphx.GenDesc_Free)
            end,
        }

        if onDef_GenDesc then onDef_GenDesc(GenDesc, mt) end
        GenDesc = setmetatable(GenDesc, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('GenDesc')
        local mt = {
            __index = {
                label  = libphx.GenDesc_Label,
                params = libphx.GenDesc_Params,
                input  = libphx.GenDesc_Input,
            },
        }

        if onDef_GenDesc_t then onDef_GenDesc_t(t, mt) end
        GenDesc_t = ffi.metatype(t, mt)
    end

    return GenDesc
end

return Loader
