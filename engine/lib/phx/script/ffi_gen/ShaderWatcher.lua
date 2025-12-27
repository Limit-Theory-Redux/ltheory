-- AUTO GENERATED. DO NOT MODIFY!
-- ShaderWatcher ---------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'ShaderWatcher'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local ShaderWatcher

    do -- C Definitions
        ffi.cdef [[
            bool ShaderWatcher_Init         ();
            void ShaderWatcher_Shutdown     ();
            bool ShaderWatcher_IsActive     ();
            void ShaderWatcher_Register     (cstr shaderKey, cstr vsPath, cstr fsPath);
            int  ShaderWatcher_Poll         ();
            cstr ShaderWatcher_GetChanged   (int index);
            void ShaderWatcher_ClearChanged ();
        ]]
    end

    do -- Global Symbol Table
        ShaderWatcher = {
            Init         = libphx.ShaderWatcher_Init,
            Shutdown     = libphx.ShaderWatcher_Shutdown,
            IsActive     = libphx.ShaderWatcher_IsActive,
            Register     = libphx.ShaderWatcher_Register,
            Poll         = libphx.ShaderWatcher_Poll,
            GetChanged   = libphx.ShaderWatcher_GetChanged,
            ClearChanged = libphx.ShaderWatcher_ClearChanged,
        }

        if onDef_ShaderWatcher then onDef_ShaderWatcher(ShaderWatcher, mt) end
        ShaderWatcher = setmetatable(ShaderWatcher, mt)
    end

    return ShaderWatcher
end

return Loader
