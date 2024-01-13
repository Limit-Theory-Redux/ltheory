local libphx = require('libphx').lib

function onDef_Sound_t(t, mt)
    mt.__index.setVolume = function(self, volume, fadeMS)
        local fadeTime = fadeMS or 0 -- set fade time or default to instant
        libphx.Sound_SetVolume(self, volume, fadeTime)
    end
end
