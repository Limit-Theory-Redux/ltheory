-- AUTO GENERATED. DO NOT MODIFY!
-- SamplerDesc -----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct SamplerDesc {} SamplerDesc;
    ]]

    return 1, 'SamplerDesc'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local SamplerDesc

    do -- C Definitions
        ffi.cdef [[
            void         SamplerDesc_Free       (SamplerDesc*);
            SamplerDesc* SamplerDesc_Create     ();
            void         SamplerDesc_Min        (SamplerDesc*, SamplerFilter filter);
            void         SamplerDesc_Mag        (SamplerDesc*, SamplerFilter filter);
            void         SamplerDesc_Mip        (SamplerDesc*, MipFilter filter);
            void         SamplerDesc_Wrap       (SamplerDesc*, TexWrapMode mode);
            void         SamplerDesc_WrapAxes   (SamplerDesc*, TexWrapMode s, TexWrapMode t, TexWrapMode r);
            void         SamplerDesc_Anisotropy (SamplerDesc*, int max);
            void         SamplerDesc_LodRange   (SamplerDesc*, int min, int max);
            void         SamplerDesc_Compare    (SamplerDesc*, CompareFn func);
        ]]
    end

    do -- Global Symbol Table
        SamplerDesc = {
            Create     = function()
                local _instance = libphx.SamplerDesc_Create()
                return Core.ManagedObject(_instance, libphx.SamplerDesc_Free)
            end,
        }

        if onDef_SamplerDesc then onDef_SamplerDesc(SamplerDesc, mt) end
        SamplerDesc = setmetatable(SamplerDesc, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('SamplerDesc')
        local mt = {
            __index = {
                min        = libphx.SamplerDesc_Min,
                mag        = libphx.SamplerDesc_Mag,
                mip        = libphx.SamplerDesc_Mip,
                wrap       = libphx.SamplerDesc_Wrap,
                wrapAxes   = libphx.SamplerDesc_WrapAxes,
                anisotropy = libphx.SamplerDesc_Anisotropy,
                lodRange   = libphx.SamplerDesc_LodRange,
                compare    = libphx.SamplerDesc_Compare,
            },
        }

        if onDef_SamplerDesc_t then onDef_SamplerDesc_t(t, mt) end
        SamplerDesc_t = ffi.metatype(t, mt)
    end

    return SamplerDesc
end

return Loader
