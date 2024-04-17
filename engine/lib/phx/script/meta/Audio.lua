---@meta

---@class Audio
Audio = {}

---@return Audio
function Audio.Create() end

---@param sound Sound
---@param init_volume number
---@param fade_millis integer
---@return SoundInstance
function Audio:play(sound, init_volume, fade_millis) end

---@param sound Sound
---@param init_volume number
---@param fade_millis integer
---@param init_pos Vec3
---@param min_distance number
---@param max_distance number
---@return SoundInstance
function Audio:play3D(sound, init_volume, fade_millis, init_pos, min_distance, max_distance) end

---@param pos Vec3
function Audio:setListenerPos(pos) end

---@return Vec3
function Audio:listenerPos() end

---@param rot Quat
function Audio:setListenerRot(rot) end

---@return Quat
function Audio:listenerRot() end

---@return integer
function Audio:getLoadedCount() end

---@return integer
function Audio:getTotalCount() end

