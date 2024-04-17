-- SoundInstance ---------------------------------------------------------------

local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct SoundInstance {} SoundInstance;
    ]]

    return 1, 'SoundInstance'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local SoundInstance

    do -- C Definitions
        ffi.cdef [[
            void   SoundInstance_Free            (SoundInstance*);
            bool   SoundInstance_IsPlaying       (SoundInstance const*);
            bool   SoundInstance_IsPaused        (SoundInstance const*);
            bool   SoundInstance_IsStopped       (SoundInstance const*);
            double SoundInstance_GetVolume       (SoundInstance const*);
            void   SoundInstance_SetVolume       (SoundInstance*, double volume, uint64 fadeMillis);
            void   SoundInstance_Pause           (SoundInstance*, uint64 fadeMillis);
            void   SoundInstance_Resume          (SoundInstance*, uint64 fadeMillis);
            void   SoundInstance_Stop            (SoundInstance*, uint64 fadeMillis);
            void   SoundInstance_FreeEmitter     (SoundInstance*);
            void   SoundInstance_SetPlayPos      (SoundInstance*, double position);
            void   SoundInstance_MovePlayPos     (SoundInstance*, double offset);
            void   SoundInstance_SetEmitterPos   (SoundInstance*, Vec3f const* position);
            Vec3f  SoundInstance_EmitterPos      (SoundInstance const*);
            float  SoundInstance_EmitterDistance (SoundInstance const*, Vec3f const* listenerPos);
        ]]
    end

    do -- Global Symbol Table
        SoundInstance = {}

        if onDef_SoundInstance then onDef_SoundInstance(SoundInstance, mt) end
        SoundInstance = setmetatable(SoundInstance, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('SoundInstance')
        local mt = {
            __index = {
                isPlaying       = libphx.SoundInstance_IsPlaying,
                isPaused        = libphx.SoundInstance_IsPaused,
                isStopped       = libphx.SoundInstance_IsStopped,
                getVolume       = libphx.SoundInstance_GetVolume,
                setVolume       = libphx.SoundInstance_SetVolume,
                pause           = libphx.SoundInstance_Pause,
                resume          = libphx.SoundInstance_Resume,
                stop            = libphx.SoundInstance_Stop,
                freeEmitter     = libphx.SoundInstance_FreeEmitter,
                setPlayPos      = libphx.SoundInstance_SetPlayPos,
                movePlayPos     = libphx.SoundInstance_MovePlayPos,
                setEmitterPos   = libphx.SoundInstance_SetEmitterPos,
                emitterPos      = libphx.SoundInstance_EmitterPos,
                emitterDistance = libphx.SoundInstance_EmitterDistance,
            },
        }

        if onDef_SoundInstance_t then onDef_SoundInstance_t(t, mt) end
        SoundInstance_t = ffi.metatype(t, mt)
    end

    return SoundInstance
end

return Loader
