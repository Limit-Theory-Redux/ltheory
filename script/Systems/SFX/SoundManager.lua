local SoundManager = class(function(self) end)

local CLEAN_EVERY_S = 2

function SoundManager:init()
    self.lastClean = 0
    self.groups = {}

    for _, soundGroup in pairs(Enums.SoundGroups) do
        self.groups[soundGroup] = {}
    end
end

function SoundManager:canSoundPlay(soundGroup)
    if self.groups[soundGroup] and #self.groups[soundGroup] < Enums.SoundGroupLimits[soundGroup] then
        return true
    end

    -- remove sound with lowest volume
    local lowestVolume = 1.0
    local soundIndexToRemove = nil

    for index, soundInstance in ipairs(self.groups[soundGroup]) do
        local soundVolume = soundInstance:getVolume()
        if soundVolume < lowestVolume then
            lowestVolume = soundInstance:getVolume()
            soundIndexToRemove = index
        end
    end

    if soundIndexToRemove then
        self.groups[soundGroup][soundIndexToRemove]:stop()
        table.remove(self.groups[soundGroup], soundIndexToRemove)

        if self.groups[soundGroup] and #self.groups[soundGroup] < Enums.SoundGroupLimits[soundGroup] then
            return true
        end
    end
    return false
end

function SoundManager:addInstance(instance, soundGroup)
    if self.groups[soundGroup] and #self.groups[soundGroup] > Enums.SoundGroupLimits[soundGroup] then
        Log.Warn("Cannot play sound as it would exceed group limits")
        return
    end

    table.insert(self.groups[soundGroup], instance)
end

function SoundManager:clean(dt)
    local time = EngineInstance:getTime()

    if time >= self.lastClean + CLEAN_EVERY_S then
        for _, soundGroup in ipairs(self.groups) do
            for index, soundInstance in ipairs(soundGroup) do
                if not soundInstance:isPlaying() then
                    table.remove(soundGroup, index)
                end
            end
        end

        self.lastClean = time
    end
end

return SoundManager
