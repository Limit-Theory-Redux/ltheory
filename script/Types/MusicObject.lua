local MusicObject = {}
MusicObject.__index = MusicObject

function MusicObject:Create(arg)
    if not arg.name or not arg.path or not arg.volume then
        Log.Warn("Cannot create MusicObject")
        return
    end
    Log.Debug("MusicObject: create new sound: " .. arg.name)

    local object = {}
    object.name = arg.name
    object.path = arg.path
    object.sound = Sound.Load(arg.path, arg.isLooping)
    object.volume = arg.volume
    setmetatable(object, MusicObject)
    return object
end

function MusicObject:Play(volume)
    local vol = volume or self.volume
    self.sound:setVolume(vol)
    GameState.audio.musicManager:play(self.sound)
end

function MusicObject:Pause()
    self.sound:pause(0)
end

function MusicObject:Rewind()
    self.sound:setPlayPos(0)
end

function MusicObject:SetVolume(volume, fadeMS)
    self.sound:setVolume(volume, fadeMS)
end

function MusicObject:IsPlaying()
    return self.sound:isPlaying()
end

return MusicObject
