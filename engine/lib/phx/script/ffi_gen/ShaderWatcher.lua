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
            bool ShaderWatcher_Init         (Renderer* r);
            void ShaderWatcher_Shutdown     (Renderer* r);
            bool ShaderWatcher_IsActive     (Renderer const* r);
            void ShaderWatcher_Register     (Renderer* r, cstr shaderKey, cstr vsPath, cstr fsPath);
            int  ShaderWatcher_Poll         (Renderer* r);
            cstr ShaderWatcher_GetChanged   (Renderer const* r, int index);
            void ShaderWatcher_ClearChanged (Renderer* r);
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
