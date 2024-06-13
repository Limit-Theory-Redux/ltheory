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
            SoundInstance* Audio_Play3D         (Audio*, Sound* sound, double initVolume, uint64 fadeMillis, Position initPos, float minDistance, float maxDistance);
            void           Audio_SetListenerPos (Audio*, Position const* pos);
            Position       Audio_ListenerPos    (Audio const*);
            void           Audio_SetListenerRot (Audio*, Quat const* rot);
            Quat*          Audio_ListenerRot    (Audio const*);
            void           Audio_SetOriginPos   (Audio*, Position const* origin);
            Position       Audio_OriginPos      (Audio const*);
            uint64         Audio_GetLoadedCount (Audio const*);
            uint64         Audio_GetTotalCount  (Audio const*);
        ]]
    end

    do -- Global Symbol Table
        Audio = {
            Create         = function(...)
                local instance = libphx.Audio_Create(...)
                return Core.ManagedObject(instance, libphx.Audio_Free)
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
                    return Core.ManagedObject(instance, libphx.SoundInstance_Free)
                end,
                play3D         = function(...)
                    local instance = libphx.Audio_Play3D(...)
                    return Core.ManagedObject(instance, libphx.SoundInstance_Free)
                end,
                setListenerPos = libphx.Audio_SetListenerPos,
                listenerPos    = libphx.Audio_ListenerPos,
                setListenerRot = libphx.Audio_SetListenerRot,
                listenerRot    = function(...)
                    local instance = libphx.Audio_ListenerRot(...)
                    return Core.ManagedObject(instance, libphx.Quat_Free)
                end,
                setOriginPos   = libphx.Audio_SetOriginPos,
                originPos      = libphx.Audio_OriginPos,
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
