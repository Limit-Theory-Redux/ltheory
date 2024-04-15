-- Audio -----------------------------------------------------------------------

---@class Audio
---@field Create fun(): Audio
---@field Play fun(self, sound: Sound, init_volume: number, fade_millis: integer): SoundInstance
---@field Play3D fun(self, sound: Sound, init_volume: number, fade_millis: integer, init_pos: Vec3, min_distance: number, max_distance: number): SoundInstance
---@field SetListenerPos fun(self, pos: Vec3)
---@field ListenerPos fun(self): Vec3
---@field SetListenerRot fun(self, rot: Quat)
---@field ListenerRot fun(self): Quat
---@field GetLoadedCount fun(self): integer
---@field GetTotalCount fun(self): integer

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
            SoundInstance* Audio_Play3D         (Audio*, Sound* sound, double initVolume, uint64 fadeMillis, Vec3f initPos, float minDistance, float maxDistance);
            void           Audio_SetListenerPos (Audio*, Vec3f const* pos);
            Vec3f          Audio_ListenerPos    (Audio const*);
            void           Audio_SetListenerRot (Audio*, Quat const* rot);
            Quat*          Audio_ListenerRot    (Audio const*);
            uint64         Audio_GetLoadedCount (Audio const*);
            uint64         Audio_GetTotalCount  (Audio const*);
        ]]
    end

    do -- Global Symbol Table
        Audio = {
            ---@return Audio
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
                ---@param sound Sound
                ---@param init_volume number
                ---@param fade_millis integer
                ---@return SoundInstance
                play           = function(...)
                    local instance = libphx.Audio_Play(...)
                    return Core.ManagedObject(instance, libphx.SoundInstance_Free)
                end,
                ---@param sound Sound
                ---@param init_volume number
                ---@param fade_millis integer
                ---@param init_pos Vec3
                ---@param min_distance number
                ---@param max_distance number
                ---@return SoundInstance
                play3D         = function(...)
                    local instance = libphx.Audio_Play3D(...)
                    return Core.ManagedObject(instance, libphx.SoundInstance_Free)
                end,
                ---@param pos Vec3
                setListenerPos = libphx.Audio_SetListenerPos,
                ---@return Vec3
                listenerPos    = libphx.Audio_ListenerPos,
                ---@param rot Quat
                setListenerRot = libphx.Audio_SetListenerRot,
                ---@return Quat
                listenerRot    = function(...)
                    local instance = libphx.Audio_ListenerRot(...)
                    return Core.ManagedObject(instance, libphx.Quat_Free)
                end,
                ---@return integer
                getLoadedCount = libphx.Audio_GetLoadedCount,
                ---@return integer
                getTotalCount  = libphx.Audio_GetTotalCount,
            },
        }

        if onDef_Audio_t then onDef_Audio_t(t, mt) end
        Audio_t = ffi.metatype(t, mt)
    end

    return Audio
end

return Loader
