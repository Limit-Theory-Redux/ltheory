local libphx = require('libphx').lib

function onDef_Audio_t(t, mt)
    mt.__index.play = function(self, sound, initialVolume, fadeMS)
        local vol = initialVolume or 1.0
        local fadeTime = fadeMS or 0 -- set fade time or default to instant
        return Core.ManagedObject(libphx.Audio_Play(self, sound, vol, fadeTime), libphx.SoundInstance_Free)
    end

    mt.__index.play3d = function(self, sound, initialVolume, fadeMS, initPos, minDistance, maxDistance)
        local vol = initialVolume or 1.0
        local fadeTime = fadeMS or 0 -- set fade time or default to instant
        local pos = initPos or Position.Identity()
        local min = minDistance or 0
        local max = maxDistance or 2000
        return Core.ManagedObject(libphx.Audio_Play3D(self, sound, vol, fadeTime, pos, min, max),
            libphx.SoundInstance_Free)
    end
end
