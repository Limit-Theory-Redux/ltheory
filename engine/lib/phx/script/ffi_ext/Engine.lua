local libphx = require('libphx').lib

function onDef_Engine(t, mt)
    t.GetVersion = function()
        return ffi.string(libphx.Engine_GetVersion())
    end
end
