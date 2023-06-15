-- Sound2 ----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local Sound2

do -- C Definitions
    ffi.cdef [[
        void    Sound2_Free          (Sound2*);
        Sound2* Sound2_Load          (cstr path, bool isLooping);
        void    Sound2_SetVolume     (Sound2*, double volume);
        void    Sound2_Pause         (Sound2*, uint64 duration);
        void    Sound2_Resume        (Sound2*, uint64 duration);
        void    Sound2_Stop          (Sound2*, uint64 duration);
        void    Sound2_SetPlayPos    (Sound2*, double position);
        void    Sound2_MovePlayPos   (Sound2*, double offset);
        void    Sound2_SetEmitterPos (Sound2*, Vec3f const* pos);
    ]]
end

do -- Global Symbol Table
    Sound2 = {
        Free          = libphx.Sound2_Free,
        Load          = libphx.Sound2_Load,
        SetVolume     = libphx.Sound2_SetVolume,
        Pause         = libphx.Sound2_Pause,
        Resume        = libphx.Sound2_Resume,
        Stop          = libphx.Sound2_Stop,
        SetPlayPos    = libphx.Sound2_SetPlayPos,
        MovePlayPos   = libphx.Sound2_MovePlayPos,
        SetEmitterPos = libphx.Sound2_SetEmitterPos,
    }

    if onDef_Sound2 then onDef_Sound2(Sound2, mt) end
    Sound2 = setmetatable(Sound2, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Sound2')
    local mt = {
        __index = {
            managed       = function(self) return ffi.gc(self, libphx.Sound2_Free) end,
            free          = libphx.Sound2_Free,
            setVolume     = libphx.Sound2_SetVolume,
            pause         = libphx.Sound2_Pause,
            resume        = libphx.Sound2_Resume,
            stop          = libphx.Sound2_Stop,
            setPlayPos    = libphx.Sound2_SetPlayPos,
            movePlayPos   = libphx.Sound2_MovePlayPos,
            setEmitterPos = libphx.Sound2_SetEmitterPos,
        },
    }

    if onDef_Sound2_t then onDef_Sound2_t(t, mt) end
    Sound2_t = ffi.metatype(t, mt)
end

return Sound2
