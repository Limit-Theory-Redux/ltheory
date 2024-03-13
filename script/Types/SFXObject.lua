local SoundManager = require "Systems.SFX.SoundManager"

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
    object.last_created = TimeStamp.Now()
    setmetatable(object, SFXObject)
    return object
end

function SFXObject:Play(pos, volume)
    if self.last_created:getElapsed() > 0.05 then
        local vol = volume or self.volume

        local soundGroup = Enums.SoundGroups.Effects

        if SoundManager:canSoundPlay(soundGroup) then
            local instance
            if pos then
                instance = GameState.audio.manager:play3d(self.sound, vol, 50, pos)
            else
                instance = GameState.audio.manager:play(self.sound, vol, 50)
            end
            SoundManager:addInstance(instance, soundGroup)
            self.last_created = TimeStamp.Now()
            return instance
        end
    end
end

return SFXObject
