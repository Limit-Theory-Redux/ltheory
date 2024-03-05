local libphx = require('libphx').lib

function onDef_Audio_t(t, mt)
    mt.__index.play = function(self, sound, initialVolume, fadeMS)
        local vol = initialVolume or 1.0
        local fadeTime = fadeMS or 0 -- set fade time or default to instant
        return libphx.Audio_Play(self, sound, vol, fadeTime)
    end
end
