local SFXObject = {}
SFXObject.__index = SFXObject

function SFXObject:Create(arg)
    if not arg.name or not arg.path or not arg.volume then
        Log.Warn("Cannot create SFXObject")
        return
    end
    Log.Debug("SFXObject: create new sound: " .. arg.name)

    local object = {}
    object.name = arg.name
    object.path = Config.paths.soundEffects .. arg.path
    object.sound = Sound.Load(object.path, arg.isLooping)
    object.volume = arg.volume
    setmetatable(object, SFXObject)
    return object
end

function SFXObject:Play(volume)
    local vol = volume or self.volume
    self.sound:setVolume(vol)
    GameState.audio.fxManager:play(self.sound)
end

function SFXObject:Stop()
    self.sound:stop(0)
end

function SFXObject:Pause()
    self.sound:pause(0)
end

function SFXObject:Resume()
    self.sound:resume(0)
end

function SFXObject:Rewind()
    self.sound:setPlayPos(0)
end

function SFXObject:SetPlayPos(pos)
    self.sound:setPlayPos(pos % self.sound:getDuration())
end

function SFXObject:SetVolume(volume, fadeMS)
    self.sound:setVolume(volume, fadeMS)
end

function SFXObject:IsPlaying()
    return self.sound:isPlaying()
end

function SFXObject:IsPaused()
    return self.sound:isPaused()
end

return SFXObject
