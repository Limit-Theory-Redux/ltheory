-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class SoundInstance
SoundInstance = {}

---@return boolean
function SoundInstance:isPlaying() end

---@return boolean
function SoundInstance:isPaused() end

---@return boolean
function SoundInstance:isStopped() end

---@return number
function SoundInstance:getVolume() end

-- Sets the volume as a linear amplitude, where `1.0` plays the sound unchanged.
---@param volume number
---@param fadeMillis integer
function SoundInstance:setVolume(volume, fadeMillis) end

---@param fadeMillis integer
function SoundInstance:pause(fadeMillis) end

---@param fadeMillis integer
function SoundInstance:resume(fadeMillis) end

---@param fadeMillis integer
function SoundInstance:stop(fadeMillis) end

-- Releases the spatial track of a 3D sound, which stops the sound.
function SoundInstance:freeEmitter() end

---@param position number
function SoundInstance:setPlayPos(position) end

---@param offset number
function SoundInstance:movePlayPos(offset) end

---@param position Position
function SoundInstance:setEmitterPos(position) end

---@param origin Position
function SoundInstance:setEmitterOriginPos(origin) end

---@return Position
function SoundInstance:emitterPos() end

---@param listenerPos Position
---@return number
function SoundInstance:emitterDistance(listenerPos) end
