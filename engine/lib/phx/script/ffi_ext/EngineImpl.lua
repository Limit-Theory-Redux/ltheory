local libphx = require('libphx').lib

function onDef_EngineImpl(t, mt)
    t.GetVersion = function()
        return ffi.string(libphx.EngineImpl_GetVersion())
    end
end
