local SoundManager = class(function(self) end)

local CLEAN_EVERY_S = 2
local UPDATE_DIST = 100000.0

function SoundManager:init()
    self.lastClean = TimeStamp.Now()
    self.groups = {}

    for _, soundGroup in pairs(Enums.SoundGroups) do
        self.groups[soundGroup] = {}
    end

    self:registerEvents()
end

function SoundManager:registerEvents()
    EventBus:subscribe(EventType.PostRender, self, self.onPostRender)
end

function SoundManager:canSoundPlay(soundGroup)
    if (not self.groups[soundGroup] or
            #self.groups[soundGroup] < Enums.SoundGroupLimits[soundGroup]) and
        GameState.audio.soundEnabled then
        return true
    end

    -- remove sound with lowest volume
    local lowestVolume = 1.0
    local soundIndexToRemove = 1

    for index, soundInstance in ipairs(self.groups[soundGroup]) do
        local soundVolume = soundInstance:getVolume()
        if soundVolume < lowestVolume then
            lowestVolume = soundVolume
            soundIndexToRemove = index
        end
    end

    local instanceToRemove = self.groups[soundGroup][soundIndexToRemove]

    if instanceToRemove then
        instanceToRemove:stop() -- a stopped sound will get dropped
        instanceToRemove:freeEmitter()
        table.remove(self.groups[soundGroup], soundIndexToRemove)

        if not self.groups[soundGroup] or #self.groups[soundGroup] < Enums.SoundGroupLimits[soundGroup] then
            return true
        end
    end
    -- Log.Warn("Sound would exceed group limits. Cancel play.")

    return false
end

function SoundManager:addInstance(instance, soundGroup)
    if not instance then
        Log.Error("Sound instance is nil")
    end

    if self.groups[soundGroup] and #self.groups[soundGroup] > Enums.SoundGroupLimits[soundGroup] then
        instance:stop()
        instance:freeEmitter()
        Log.Warn("Cannot play sound as it would exceed group limits")
        return
    end

    table.insert(self.groups[soundGroup], instance)
end

function SoundManager:getSoundsPlaying(soundGroup)
    if self.groups[soundGroup] and #self.groups[soundGroup] < Enums.SoundGroupLimits[soundGroup] then
        return #self.groups[soundGroup]
    end
    return nil
end

function SoundManager:onPostRender(dt)
    -- Clean up finished sounds.
    if self.lastClean:getElapsed() > CLEAN_EVERY_S then
        local instanceCount = 0
        for _, soundGroup in ipairs(self.groups) do
            for index, soundInstance in ipairs(soundGroup) do
                if not soundInstance:isPlaying() then
                    instanceCount = instanceCount + 1
                    soundInstance:freeEmitter()
                    table.remove(soundGroup, index)
                end
            end
        end

        -- if instanceCount > 0 then
        --     Log.Debug("[SoundManager] Cleaned " .. instanceCount .. " SoundInstance")
        -- end

        self.lastClean = TimeStamp.Now()
    end

    -- If the listener has strayed too far from the origin, update it.
    local audio = GameState.audio.manager
    if audio:listenerPos():distanceSquared(audio:originPos()) > (UPDATE_DIST * UPDATE_DIST) then
        local newOrigin = audio:listenerPos()
        audio:setOriginPos(newOrigin)

        -- Update all playing sounds.
        for _, soundGroup in ipairs(self.groups) do
            for _, soundInstance in ipairs(soundGroup) do
                soundInstance:setEmitterOriginPos(newOrigin)
            end
        end
    end
end

return SoundManager
