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
    object.instances = {}
    object.last_created = 0
    setmetatable(object, SFXObject)
    return object
end

function SFXObject:Play(volume)
    local time = EngineInstance:getTime()
    if (time - self.last_created) > 0.05 or self.last_created == 0 then
        self.last_created = time
        local vol = volume or self.volume
        local instance = GameState.audio.manager:play(self.sound, SoundGroup.Effects, vol)
        table.insert(self.instances, instance)
        return instance
    end
end

return SFXObject
