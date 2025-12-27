-- AUTO GENERATED. DO NOT MODIFY!
-- RenderBatchApi --------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    return 0, 'RenderBatchApi'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local RenderBatchApi

    do -- C Definitions
        ffi.cdef [[
            void   RenderBatchApi_Begin                ();
            void   RenderBatchApi_SetCamera            (float view00, float view01, float view02, float view03, float view10, float view11, float view12, float view13, float view20, float view21, float view22, float view23, float view30, float view31, float view32, float view33, float proj00, float proj01, float proj02, float proj03, float proj10, float proj11, float proj12, float proj13, float proj20, float proj21, float proj22, float proj23, float proj30, float proj31, float proj32, float proj33, float eyeX, float eyeY, float eyeZ);
            void   RenderBatchApi_AddEntity            (float t00, float t01, float t02, float t03, float t10, float t11, float t12, float t13, float t20, float t21, float t22, float t23, float t30, float t31, float t32, float t33, float boundsX, float boundsY, float boundsZ, float boundsRadius, uint32 meshVao, int indexCount, uint32 shaderHandle, uint32 sortKey);
            void   RenderBatchApi_Flush                ();
            uint32 RenderBatchApi_GetEntityCount       ();
            uint32 RenderBatchApi_GetEntitiesSubmitted ();
            uint32 RenderBatchApi_GetEntitiesVisible   ();
            uint32 RenderBatchApi_GetEntitiesCulled    ();
            uint32 RenderBatchApi_GetCommandsGenerated ();
        ]]
    end

    do -- Global Symbol Table
        RenderBatchApi = {
            Begin                = libphx.RenderBatchApi_Begin,
            SetCamera            = libphx.RenderBatchApi_SetCamera,
            AddEntity            = libphx.RenderBatchApi_AddEntity,
            Flush                = libphx.RenderBatchApi_Flush,
            GetEntityCount       = libphx.RenderBatchApi_GetEntityCount,
            GetEntitiesSubmitted = libphx.RenderBatchApi_GetEntitiesSubmitted,
            GetEntitiesVisible   = libphx.RenderBatchApi_GetEntitiesVisible,
            GetEntitiesCulled    = libphx.RenderBatchApi_GetEntitiesCulled,
            GetCommandsGenerated = libphx.RenderBatchApi_GetCommandsGenerated,
        }

        if onDef_RenderBatchApi then onDef_RenderBatchApi(RenderBatchApi, mt) end
        RenderBatchApi = setmetatable(RenderBatchApi, mt)
    end

    return RenderBatchApi
end

return Loader
