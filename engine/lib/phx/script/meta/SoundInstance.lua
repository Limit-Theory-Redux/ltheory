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

---@param volume number
---@param fadeMillis integer
function SoundInstance:setVolume(volume, fadeMillis) end

---@param fadeMillis integer
function SoundInstance:pause(fadeMillis) end

---@param fadeMillis integer
function SoundInstance:resume(fadeMillis) end

---@param fadeMillis integer
function SoundInstance:stop(fadeMillis) end

function SoundInstance:freeEmitter() end

---@param position number
function SoundInstance:setPlayPos(position) end

---@param offset number
function SoundInstance:movePlayPos(offset) end

---@param position Vec3f
function SoundInstance:setEmitterPos(position) end

---@return Vec3f
function SoundInstance:emitterPos() end

---@param listenerPos Vec3f
---@return number
function SoundInstance:emitterDistance(listenerPos) end

