-- AUTO GENERATED. DO NOT MODIFY!
-- ShaderError -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'ShaderError'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local ShaderError

    do -- C Definitions
        ffi.cdef [[
            int    ShaderError_GetCount           ();
            bool   ShaderError_HasNewErrors       ();
            void   ShaderError_AcknowledgeErrors  ();
            cstr   ShaderError_GetShaderKey       (int index);
            cstr   ShaderError_GetErrorType       (int index);
            cstr   ShaderError_GetMessage         (int index);
            uint64 ShaderError_GetTimestamp       (int index);
            void   ShaderError_Clear              ();
            void   ShaderError_ClearAt            (int index);
            void   ShaderError_ClearForShader     (cstr shaderKey);
            void   ShaderError_Update             ();
            cstr   ShaderError_GetLatestMessage   ();
            cstr   ShaderError_GetLatestShaderKey ();
        ]]
    end

    do -- Global Symbol Table
        ShaderError = {
            GetCount           = libphx.ShaderError_GetCount,
            HasNewErrors       = libphx.ShaderError_HasNewErrors,
            AcknowledgeErrors  = libphx.ShaderError_AcknowledgeErrors,
            GetShaderKey       = libphx.ShaderError_GetShaderKey,
            GetErrorType       = libphx.ShaderError_GetErrorType,
            GetMessage         = libphx.ShaderError_GetMessage,
            GetTimestamp       = libphx.ShaderError_GetTimestamp,
            Clear              = libphx.ShaderError_Clear,
            ClearAt            = libphx.ShaderError_ClearAt,
            ClearForShader     = libphx.ShaderError_ClearForShader,
            Update             = libphx.ShaderError_Update,
            GetLatestMessage   = libphx.ShaderError_GetLatestMessage,
            GetLatestShaderKey = libphx.ShaderError_GetLatestShaderKey,
        }

        if onDef_ShaderError then onDef_ShaderError(ShaderError, mt) end
        ShaderError = setmetatable(ShaderError, mt)
    end

    return ShaderError
end

return Loader
