local libphx = require('libphx').lib

function onDef_SoundInstance_t(t, mt)
    mt.__index.setVolume = function(self, volume, fadeMS)
        local fadeTime = fadeMS or 0 -- set fade time or default to instant
        libphx.SoundInstance_SetVolume(self, volume, fadeTime)
    end

    mt.__index.stop = function(self, fadeMS)
        local fadeTime = fadeMS or 0 -- set fade time or default to instant
        libphx.SoundInstance_Stop(self, fadeTime)
    end
end
