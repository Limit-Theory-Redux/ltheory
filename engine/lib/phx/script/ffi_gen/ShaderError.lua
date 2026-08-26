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
            int    ShaderError_GetCount           (Renderer const* r);
            bool   ShaderError_HasNewErrors       (Renderer const* r);
            void   ShaderError_AcknowledgeErrors  (Renderer* r);
            cstr   ShaderError_GetShaderKey       (Renderer const* r, int index);
            cstr   ShaderError_GetErrorType       (Renderer const* r, int index);
            cstr   ShaderError_GetMessage         (Renderer const* r, int index);
            uint64 ShaderError_GetTimestamp       (Renderer const* r, int index);
            void   ShaderError_Clear              (Renderer* r);
            void   ShaderError_ClearAt            (Renderer* r, int index);
            void   ShaderError_ClearForShader     (Renderer* r, cstr shaderKey);
            void   ShaderError_Update             (Renderer* r);
            cstr   ShaderError_GetLatestMessage   (Renderer const* r);
            cstr   ShaderError_GetLatestShaderKey (Renderer const* r);
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
