-- AUTO GENERATED. DO NOT MODIFY!
-- Engine ----------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Engine {} Engine;
    ]]

    return 1, 'Engine'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Engine

    do -- C Definitions
        ffi.cdef [[
            void       Engine_Free                               (Engine*);
            Window*    Engine_Window                             (Engine*);
            Input*     Engine_Input                              (Engine*);
            EventBus*  Engine_EventBus                           (Engine*);
            TaskQueue* Engine_TaskQueue                          (Engine*);
            HmGui*     Engine_HmGui                              (Engine*);
            void       Engine_Abort                              ();
            int        Engine_GetBits                            ();
            double     Engine_GetTime                            (Engine const*);
            cstr       Engine_GetVersion                         ();
            void       Engine_Exit                               (Engine*);
            void       Engine_Terminate                          ();
            void       Engine_Update                             ();
            bool       Engine_StartRenderThread                  (Engine*);
            void       Engine_StopRenderThread                   (Engine*);
            bool       Engine_IsRenderThreadActive               (Engine const*);
            uint64     Engine_GetRenderThreadCommands            (Engine const*);
            uint64     Engine_GetRenderThreadDrawCalls           (Engine const*);
            uint64     Engine_GetRenderThreadStateChanges        (Engine const*);
            uint64     Engine_GetRenderThreadFrameCount          (Engine const*);
            double     Engine_GetRenderThreadFrameTimeMs         (Engine const*);
            uint64     Engine_GetRenderThreadCommandsPerFrame    (Engine const*);
            uint64     Engine_GetRenderThreadDrawCallsPerFrame   (Engine const*);
            uint64     Engine_GetRenderThreadTextureBindsSkipped (Engine const*);
            double     Engine_GetMainThreadWaitTimeMs            (Engine const*);
            uint64     Engine_GetFramesInFlight                  (Engine const*);
            uint32     Engine_GetCpuCount                        (Engine const*);
            uint32     Engine_GetWorkerThreadCount               (Engine const*);
            bool       Engine_IsWorkerPoolActive                 (Engine const*);
            uint32     Engine_GetActiveWorkerCount               (Engine const*);
            uint32     Engine_FlushRenderBatch                   (Engine const*);
            void       Engine_CreateCameraUBO                    (Engine const*);
            void       Engine_UpdateCameraUBO                    (Engine const*, Matrix const* mView, Matrix const* mViewInv, Matrix const* mProj, float eyeX, float eyeY, float eyeZ, float starDirX, float starDirY, float starDirZ);
            void       Engine_CreateLightUBO                     (Engine const*);
            void       Engine_UpdateLightUBO                     (Engine const*, float posX, float posY, float posZ, float radius, float r, float g, float b, float intensity);
            bool       Engine_ReloadShaderOnRenderThread         (Engine const*, cstr shaderKey, cstr vsName, cstr fsName);
        ]]
    end

    do -- Global Symbol Table
        Engine = {
            Abort                              = libphx.Engine_Abort,
            GetBits                            = libphx.Engine_GetBits,
            GetVersion                         = libphx.Engine_GetVersion,
            Terminate                          = libphx.Engine_Terminate,
            Update                             = libphx.Engine_Update,
        }

        if onDef_Engine then onDef_Engine(Engine, mt) end
        Engine = setmetatable(Engine, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Engine')
        local mt = {
            __index = {
                window                             = libphx.Engine_Window,
                input                              = libphx.Engine_Input,
                eventBus                           = libphx.Engine_EventBus,
                taskQueue                          = libphx.Engine_TaskQueue,
                hmGui                              = libphx.Engine_HmGui,
                getTime                            = libphx.Engine_GetTime,
                exit                               = libphx.Engine_Exit,
                startRenderThread                  = libphx.Engine_StartRenderThread,
                stopRenderThread                   = libphx.Engine_StopRenderThread,
                isRenderThreadActive               = libphx.Engine_IsRenderThreadActive,
                getRenderThreadCommands            = libphx.Engine_GetRenderThreadCommands,
                getRenderThreadDrawCalls           = libphx.Engine_GetRenderThreadDrawCalls,
                getRenderThreadStateChanges        = libphx.Engine_GetRenderThreadStateChanges,
                getRenderThreadFrameCount          = libphx.Engine_GetRenderThreadFrameCount,
                getRenderThreadFrameTimeMs         = libphx.Engine_GetRenderThreadFrameTimeMs,
                getRenderThreadCommandsPerFrame    = libphx.Engine_GetRenderThreadCommandsPerFrame,
                getRenderThreadDrawCallsPerFrame   = libphx.Engine_GetRenderThreadDrawCallsPerFrame,
                getRenderThreadTextureBindsSkipped = libphx.Engine_GetRenderThreadTextureBindsSkipped,
                getMainThreadWaitTimeMs            = libphx.Engine_GetMainThreadWaitTimeMs,
                getFramesInFlight                  = libphx.Engine_GetFramesInFlight,
                getCpuCount                        = libphx.Engine_GetCpuCount,
                getWorkerThreadCount               = libphx.Engine_GetWorkerThreadCount,
                isWorkerPoolActive                 = libphx.Engine_IsWorkerPoolActive,
                getActiveWorkerCount               = libphx.Engine_GetActiveWorkerCount,
                flushRenderBatch                   = libphx.Engine_FlushRenderBatch,
                createCameraUBO                    = libphx.Engine_CreateCameraUBO,
                updateCameraUBO                    = libphx.Engine_UpdateCameraUBO,
                createLightUBO                     = libphx.Engine_CreateLightUBO,
                updateLightUBO                     = libphx.Engine_UpdateLightUBO,
                reloadShaderOnRenderThread         = libphx.Engine_ReloadShaderOnRenderThread,
            },
        }

        if onDef_Engine_t then onDef_Engine_t(t, mt) end
        Engine_t = ffi.metatype(t, mt)
    end

    return Engine
end

return Loader
