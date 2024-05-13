local SoundManager = require "Systems.SFX.SoundManager"

---@class MusicObject
local MusicObject = {}
MusicObject.__index = MusicObject

function MusicObject:Create(arg)
    if not arg.name or not arg.path or not arg.volume then
        Log.Warn("Cannot create MusicObject")
        return
    end
    -- Log.Debug("MusicObject: create new sound: " .. arg.name)

    local object = {}
    object.name = arg.name
    object.path = arg.path
    object.sound = Sound.Load(arg.path, arg.isLooping)
    object.volume = arg.volume
    object.instance = nil
    setmetatable(object, MusicObject)
    return object
end

function MusicObject:Play(volume, fadeInMS)
    local vol = volume or self.volume

    local soundGroup = Enums.SoundGroups.Music

    -- clear existing instance
    if self.instance then
        self.instance:stop()
        self.instance = nil
    end

    if SoundManager:canSoundPlay(soundGroup) then
        -- Log.Debug("[MusicObject:Play] " .. self.name)
        self.instance = GameState.audio.manager:play(self.sound, vol, fadeInMS)
        SoundManager:addInstance(self.instance, soundGroup)
    end
end

function MusicObject:Stop()
    if self.instance then
        self.instance:stop()
        self.instance = nil
    end
end

function MusicObject:Pause()
    if self.instance then
        self.instance:pause(0)
    end
end

function MusicObject:Rewind()
    if self.instance then
        self.instance:setPlayPos(0)
    end
end

function MusicObject:SetVolume(volume, fadeMS)
    self.volume = volume
    if self.instance then
        self.instance:setVolume(volume, fadeMS)
    end
end

function MusicObject:IsPlaying()
    if self.instance then
        return self.instance:isPlaying()
    end
end

return MusicObject
