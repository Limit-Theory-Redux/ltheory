---@meta

SoundInstance = SoundInstance

---@return boolean
function SoundInstance:isPlaying(self) end

---@return boolean
function SoundInstance:isPaused(self) end

---@return boolean
function SoundInstance:isStopped(self) end

---@return number
function SoundInstance:getVolume(self) end

---@param volume number
---@param fade_millis integer
function SoundInstance:setVolume(self, volume, fade_millis) end

---@param fade_millis integer
function SoundInstance:pause(self, fade_millis) end

---@param fade_millis integer
function SoundInstance:resume(self, fade_millis) end

---@param fade_millis integer
function SoundInstance:stop(self, fade_millis) end

function SoundInstance:freeEmitter(self) end

---@param position number
function SoundInstance:setPlayPos(self, position) end

---@param offset number
function SoundInstance:movePlayPos(self, offset) end

---@param position Vec3
function SoundInstance:setEmitterPos(self, position) end

---@return Vec3
function SoundInstance:emitterPos(self) end

---@param listener_pos Vec3
---@return number
function SoundInstance:emitterDistance(self, listener_pos) end

