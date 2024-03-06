-- Sound -----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Sound {} Sound;
    ]]

    return 1, 'Sound'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Sound

    do -- C Definitions
        ffi.cdef [[
            void   Sound_Free          (Sound*);
            Sound* Sound_Load          (cstr path, bool isLooping);
            float  Sound_GetDuration   (Sound const*);
            cstr   Sound_GetPath       (Sound const*);
            void   Sound_SetEmitterPos (Sound*, Vec3f const* position);
        ]]
    end

    do -- Global Symbol Table
        Sound = {
            Load          = function(...)
                local instance = libphx.Sound_Load(...)
                ffi.gc(instance, libphx.Sound_Free)
                return instance
            end,
        }

        if onDef_Sound then onDef_Sound(Sound, mt) end
        Sound = setmetatable(Sound, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Sound')
        local mt = {
            __index = {
                getDuration   = libphx.Sound_GetDuration,
                getPath       = libphx.Sound_GetPath,
                setEmitterPos = libphx.Sound_SetEmitterPos,
            },
        }

        if onDef_Sound_t then onDef_Sound_t(t, mt) end
        Sound_t = ffi.metatype(t, mt)
    end

    return Sound
end

return Loader
