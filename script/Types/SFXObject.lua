local SoundManager = require('Legacy.Systems.SFX.SoundManager')

---@class SFXObject
local SFXObject = {}
SFXObject.__index = SFXObject

function SFXObject:create(arg)
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

---Play the sound, scaled by the Effects Volume and Master Volume settings.
---@param volume number|nil default: the volume it was created with
---@param pos Position|nil plays in 3D at this position
function SFXObject:play(volume, pos)
    if self.last_created:getElapsed() > 0.05 then
        local vol = (volume or self.volume)
            * (GameState.audio.effectsVolume or 1) * (GameState.audio.masterVolume or 1)

        local soundGroup = Enums.SoundGroups.Effects

        if SoundManager:canSoundPlay(soundGroup) then
            local instance
            if pos then
                instance = GameState.audio.manager:play3d(self.sound, vol, 0, pos)
            else
                instance = GameState.audio.manager:play(self.sound, vol, 0)
            end
            SoundManager:addInstance(instance, soundGroup)
            self.last_created = TimeStamp.Now()
            return instance
        end
    end
end

return SFXObject
