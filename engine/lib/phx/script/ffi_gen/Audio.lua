-- Audio -----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Audio {} Audio;
    ]]

    return 1, 'Audio'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Audio

    do -- C Definitions
        ffi.cdef [[
            void           Audio_Free           (Audio*);
            Audio*         Audio_Create         ();
            SoundInstance* Audio_Play           (Audio*, Sound* sound, double initVolume, uint64 fadeMillis);
            void           Audio_SetListenerPos (Audio*, Vec3f const* pos, Quat const* rot);
            uint64         Audio_GetLoadedCount (Audio const*);
            uint64         Audio_GetTotalCount  (Audio const*);
        ]]
    end

    do -- Global Symbol Table
        Audio = {
            Create         = function(...)
                local instance = libphx.Audio_Create(...)
                ffi.gc(instance, libphx.Audio_Free)
                return instance
            end,
        }

        if onDef_Audio then onDef_Audio(Audio, mt) end
        Audio = setmetatable(Audio, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Audio')
        local mt = {
            __index = {
                play           = function(...)
                    local instance = libphx.Audio_Play(...)
                    ffi.gc(instance, libphx.SoundInstance_Free)
                    return instance
                end,
                setListenerPos = libphx.Audio_SetListenerPos,
                getLoadedCount = libphx.Audio_GetLoadedCount,
                getTotalCount  = libphx.Audio_GetTotalCount,
            },
        }

        if onDef_Audio_t then onDef_Audio_t(t, mt) end
        Audio_t = ffi.metatype(t, mt)
    end

    return Audio
end

return Loader
