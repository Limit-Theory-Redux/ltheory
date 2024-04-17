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
---@param fade_millis integer
function SoundInstance:setVolume(volume, fade_millis) end

---@param fade_millis integer
function SoundInstance:pause(fade_millis) end

---@param fade_millis integer
function SoundInstance:resume(fade_millis) end

---@param fade_millis integer
function SoundInstance:stop(fade_millis) end

function SoundInstance:freeEmitter() end

---@param position number
function SoundInstance:setPlayPos(position) end

---@param offset number
function SoundInstance:movePlayPos(offset) end

---@param position Vec3
function SoundInstance:setEmitterPos(position) end

---@return Vec3
function SoundInstance:emitterPos() end

---@param listener_pos Vec3
---@return number
function SoundInstance:emitterDistance(listener_pos) end

