-- Audio2 ----------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local Audio2

do -- C Definitions
    ffi.cdef [[
        void    Audio2_Free            (Audio2*);
        Audio2* Audio2_Create          ();
        void    Audio2_Play            (Audio2*, Sound2* sound);
        void    Audio2_SetListenerPos  (Audio2*, Vec3f const* pos, Quat const* rot);
        uint64  Audio2_GetLoadedCount  (Audio2 const*);
        uint64  Audio2_GetTotalCount   (Audio2 const*);
        uint64  Audio2_GetPlayingCount (Audio2 const*);
    ]]
end

do -- Global Symbol Table
    Audio2 = {
        Free            = libphx.Audio2_Free,
        Create          = libphx.Audio2_Create,
        Play            = libphx.Audio2_Play,
        SetListenerPos  = libphx.Audio2_SetListenerPos,
        GetLoadedCount  = libphx.Audio2_GetLoadedCount,
        GetTotalCount   = libphx.Audio2_GetTotalCount,
        GetPlayingCount = libphx.Audio2_GetPlayingCount,
    }

    if onDef_Audio2 then onDef_Audio2(Audio2, mt) end
    Audio2 = setmetatable(Audio2, mt)
end

do -- Metatype for class instances
    local t  = ffi.typeof('Audio2')
    local mt = {
        __index = {
            managed         = function(self) return ffi.gc(self, libphx.Audio2_Free) end,
            free            = libphx.Audio2_Free,
            play            = libphx.Audio2_Play,
            setListenerPos  = libphx.Audio2_SetListenerPos,
            getLoadedCount  = libphx.Audio2_GetLoadedCount,
            getTotalCount   = libphx.Audio2_GetTotalCount,
            getPlayingCount = libphx.Audio2_GetPlayingCount,
        },
    }

    if onDef_Audio2_t then onDef_Audio2_t(t, mt) end
    Audio2_t = ffi.metatype(t, mt)
end

return Audio2
