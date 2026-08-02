-- AUTO GENERATED. DO NOT MODIFY!
-- RenderTarget ----------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'RenderTarget'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local RenderTarget

    do -- C Definitions
        ffi.cdef [[
            void RenderTarget_Push             (Renderer* r, int sx, int sy);
            void RenderTarget_Pop              (Renderer* r);
            void RenderTarget_BindTex2D        (Renderer* r, Tex2D const* tex);
            void RenderTarget_BindTex2DLevel   (Renderer* r, Tex2D const* tex, int level);
            void RenderTarget_BindTex3D        (Renderer* r, Tex3D const* tex, int layer);
            void RenderTarget_BindTex3DLevel   (Renderer* r, Tex3D const* tex, int layer, int level);
            void RenderTarget_BindTexCube      (Renderer* r, TexCube const* tex, CubeFace face);
            void RenderTarget_BindTexCubeLevel (Renderer* r, TexCube const* tex, CubeFace face, int level);
            void RenderTarget_PushTex2D        (Renderer* r, Tex2D const* tex);
            void RenderTarget_PushTex2DLevel   (Renderer* r, Tex2D const* tex, int level);
            void RenderTarget_PushTex3D        (Renderer* r, Tex3D const* tex, int layer);
            void RenderTarget_PushTex3DLevel   (Renderer* r, Tex3D const* tex, int layer, int level);
        ]]
    end

    do -- Global Symbol Table
        RenderTarget = {
            Push             = libphx.RenderTarget_Push,
            Pop              = libphx.RenderTarget_Pop,
            BindTex2D        = libphx.RenderTarget_BindTex2D,
            BindTex2DLevel   = libphx.RenderTarget_BindTex2DLevel,
            BindTex3D        = libphx.RenderTarget_BindTex3D,
            BindTex3DLevel   = libphx.RenderTarget_BindTex3DLevel,
            BindTexCube      = libphx.RenderTarget_BindTexCube,
            BindTexCubeLevel = libphx.RenderTarget_BindTexCubeLevel,
            PushTex2D        = libphx.RenderTarget_PushTex2D,
            PushTex2DLevel   = libphx.RenderTarget_PushTex2DLevel,
            PushTex3D        = libphx.RenderTarget_PushTex3D,
            PushTex3DLevel   = libphx.RenderTarget_PushTex3DLevel,
        }

        if onDef_RenderTarget then onDef_RenderTarget(RenderTarget, mt) end
        RenderTarget = setmetatable(RenderTarget, mt)
    end

    return RenderTarget
end

return Loader
