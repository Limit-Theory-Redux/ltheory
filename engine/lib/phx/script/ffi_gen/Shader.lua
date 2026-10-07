-- AUTO GENERATED. DO NOT MODIFY!
-- Shader ----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Shader {} Shader;
    ]]

    return 1, 'Shader'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Shader

    do -- C Definitions
        ffi.cdef [[
            void    Shader_Free       (Shader*);
            Shader* Shader_Create     (Renderer* r, cstr vs, cstr fs);
            Shader* Shader_Load       (Renderer* r, cstr vsName, cstr fsName);
            bool    Shader_Reload     (Shader*, Renderer* r);
            cstr    Shader_Name       (Shader const*);
            cstr    Shader_BlockDecl  (Shader const*, cstr name);
            uint32  Shader_BlockSize  (Shader const*, cstr name);
            uint32  Shader_Generation (Shader const*);
            uint64  Shader_ResourceId (Shader const*);
            Shader* Shader_Clone      (Shader const*);
        ]]
    end

    do -- Global Symbol Table
        Shader = {
            Create     = function(r, vs, fs)
                local _instance = libphx.Shader_Create(r, vs, fs)
                return Core.ManagedObject(_instance, libphx.Shader_Free)
            end,
            Load       = function(r, vsName, fsName)
                local _instance = libphx.Shader_Load(r, vsName, fsName)
                return Core.ManagedObject(_instance, libphx.Shader_Free)
            end,
        }

        if onDef_Shader then onDef_Shader(Shader, mt) end
        Shader = setmetatable(Shader, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Shader')
        local mt = {
            __index = {
                reload     = libphx.Shader_Reload,
                name       = libphx.Shader_Name,
                blockDecl  = libphx.Shader_BlockDecl,
                blockSize  = libphx.Shader_BlockSize,
                generation = libphx.Shader_Generation,
                resourceId = libphx.Shader_ResourceId,
                clone      = function(self)
                    local _instance = libphx.Shader_Clone(self)
                    return Core.ManagedObject(_instance, libphx.Shader_Free)
                end,
            },
        }

        if onDef_Shader_t then onDef_Shader_t(t, mt) end
        Shader_t = ffi.metatype(t, mt)
    end

    return Shader
end

return Loader
