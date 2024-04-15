-- SoundInstance ---------------------------------------------------------------

---@class SoundInstance
---@field IsPlaying fun(self): boolean
---@field IsPaused fun(self): boolean
---@field IsStopped fun(self): boolean
---@field GetVolume fun(self): number
---@field SetVolume fun(self, volume: number, fade_millis: integer)
---@field Pause fun(self, fade_millis: integer)
---@field Resume fun(self, fade_millis: integer)
---@field Stop fun(self, fade_millis: integer)
---@field FreeEmitter fun(self)
---@field SetPlayPos fun(self, position: number)
---@field MovePlayPos fun(self, offset: number)
---@field SetEmitterPos fun(self, position: Vec3)
---@field EmitterPos fun(self): Vec3
---@field EmitterDistance fun(self, listener_pos: Vec3): number

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
                ---@return boolean
                isPlaying       = libphx.SoundInstance_IsPlaying,
                ---@return boolean
                isPaused        = libphx.SoundInstance_IsPaused,
                ---@return boolean
                isStopped       = libphx.SoundInstance_IsStopped,
                ---@return number
                getVolume       = libphx.SoundInstance_GetVolume,
                ---@param volume number
                ---@param fade_millis integer
                setVolume       = libphx.SoundInstance_SetVolume,
                ---@param fade_millis integer
                pause           = libphx.SoundInstance_Pause,
                ---@param fade_millis integer
                resume          = libphx.SoundInstance_Resume,
                ---@param fade_millis integer
                stop            = libphx.SoundInstance_Stop,
                freeEmitter     = libphx.SoundInstance_FreeEmitter,
                ---@param position number
                setPlayPos      = libphx.SoundInstance_SetPlayPos,
                ---@param offset number
                movePlayPos     = libphx.SoundInstance_MovePlayPos,
                ---@param position Vec3
                setEmitterPos   = libphx.SoundInstance_SetEmitterPos,
                ---@return Vec3
                emitterPos      = libphx.SoundInstance_EmitterPos,
                ---@param listener_pos Vec3
                ---@return number
                emitterDistance = libphx.SoundInstance_EmitterDistance,
            },
        }

        if onDef_SoundInstance_t then onDef_SoundInstance_t(t, mt) end
        SoundInstance_t = ffi.metatype(t, mt)
    end

    return SoundInstance
end

return Loader
