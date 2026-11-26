-- AUTO GENERATED. DO NOT MODIFY!
---@meta

---@class Audio
Audio = {}

-- Opens the default output device and creates the listener for 3D sounds.
--
-- Panics if the output device cannot be opened.
---@return Audio
function Audio.Create() end

---@param sound Sound
---@param initVolume number
---@param fadeMillis integer
---@return SoundInstance
function Audio:play(sound, initVolume, fadeMillis) end

-- Plays the sound at a position on a spatial track of its own, panned and
-- attenuated for the listener.
--
-- The volume falls off linearly in decibels, from full volume at the
-- minimum distance to silence at the maximum distance.
---@param sound Sound
---@param initVolume number
---@param fadeMillis integer
---@param initPos Position
---@param minDistance number
---@param maxDistance number
---@return SoundInstance
function Audio:play3D(sound, initVolume, fadeMillis, initPos, minDistance, maxDistance) end

---@param pos Position
function Audio:setListenerPos(pos) end

---@return Position
function Audio:listenerPos() end

---@param rot Quat
function Audio:setListenerRot(rot) end

---@return Quat
function Audio:listenerRot() end

-- Updates the origin in Kira's coordinate system.
--
-- As Kira maintains a 32-bit coordinate system, if the listener strays too far away from the origin, we will start to have difficulty with 32-bit precision.
-- If this function is called, the listener and all new sounds will have their position calculated from the new origin in Kira's coordinate system.
---@param origin Position
function Audio:setOriginPos(origin) end

---@return Position
function Audio:originPos() end

-- Number of sounds alive: sounds on the main track plus one per spatial
-- track of a 3D sound.
---@return integer
function Audio:getLoadedCount() end

-- Maximum number of sounds: the main track's sound capacity plus the
-- maximum number of spatial tracks for 3D sounds.
---@return integer
function Audio:getTotalCount() end
