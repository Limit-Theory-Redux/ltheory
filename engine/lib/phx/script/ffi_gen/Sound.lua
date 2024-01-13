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
            bool   Sound_IsPlaying     (Sound const*);
            bool   Sound_IsPaused      (Sound const*);
            bool   Sound_IsStopped     (Sound const*);
            void   Sound_SetVolume     (Sound*, double volume, uint64 fadeMillis);
            void   Sound_Pause         (Sound*, uint64 fadeMillis);
            void   Sound_Resume        (Sound*, uint64 fadeMillis);
            void   Sound_Stop          (Sound*, uint64 fadeMillis);
            void   Sound_SetPlayPos    (Sound*, double position);
            void   Sound_MovePlayPos   (Sound*, double offset);
            void   Sound_SetEmitterPos (Sound*, Vec3f const* position);
        ]]
    end

    do -- Global Symbol Table
        Sound = {
            Free          = libphx.Sound_Free,
            Load          = libphx.Sound_Load,
            GetDuration   = libphx.Sound_GetDuration,
            GetPath       = libphx.Sound_GetPath,
            IsPlaying     = libphx.Sound_IsPlaying,
            IsPaused      = libphx.Sound_IsPaused,
            IsStopped     = libphx.Sound_IsStopped,
            SetVolume     = libphx.Sound_SetVolume,
            Pause         = libphx.Sound_Pause,
            Resume        = libphx.Sound_Resume,
            Stop          = libphx.Sound_Stop,
            SetPlayPos    = libphx.Sound_SetPlayPos,
            MovePlayPos   = libphx.Sound_MovePlayPos,
            SetEmitterPos = libphx.Sound_SetEmitterPos,
        }

        if onDef_Sound then onDef_Sound(Sound, mt) end
        Sound = setmetatable(Sound, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Sound')
        local mt = {
            __index = {
                managed       = function(self) return ffi.gc(self, libphx.Sound_Free) end,
                free          = libphx.Sound_Free,
                getDuration   = libphx.Sound_GetDuration,
                getPath       = libphx.Sound_GetPath,
                isPlaying     = libphx.Sound_IsPlaying,
                isPaused      = libphx.Sound_IsPaused,
                isStopped     = libphx.Sound_IsStopped,
                setVolume     = libphx.Sound_SetVolume,
                pause         = libphx.Sound_Pause,
                resume        = libphx.Sound_Resume,
                stop          = libphx.Sound_Stop,
                setPlayPos    = libphx.Sound_SetPlayPos,
                movePlayPos   = libphx.Sound_MovePlayPos,
                setEmitterPos = libphx.Sound_SetEmitterPos,
            },
        }

        if onDef_Sound_t then onDef_Sound_t(t, mt) end
        Sound_t = ffi.metatype(t, mt)
    end

    return Sound
end

return Loader
