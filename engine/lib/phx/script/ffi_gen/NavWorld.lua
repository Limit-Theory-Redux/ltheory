-- AUTO GENERATED. DO NOT MODIFY!
-- NavWorld --------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct NavWorld {} NavWorld;
    ]]

    return 1, 'NavWorld'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local NavWorld

    do -- C Definitions
        ffi.cdef [[
            void      NavWorld_Free               (NavWorld*);
            NavWorld* NavWorld_Create             (Physics const* physics);
            uint32    NavWorld_AxisCount          ();
            uint32    NavWorld_AgentCount         (NavWorld const*);
            uint32    NavWorld_AddAgent           (NavWorld*, RigidBody const* body, double maxSpeed, double linearDrag);
            void      NavWorld_AddObstacle        (NavWorld*, RigidBody const* body);
            void      NavWorld_SetGoal            (NavWorld*, uint32 id, Position const* target, double arrivalRadius);
            void      NavWorld_ClearGoal          (NavWorld*, uint32 id);
            void      NavWorld_SetAvoidanceTuning (NavWorld*, double horizonS, double clearanceRadii, double aimFactor);
            void      NavWorld_Step               (NavWorld*);
            void      NavWorld_CopyAxes           (NavWorld const*, double* out, uint64 out_size);
            void      NavWorld_CopyArrived        (NavWorld const*, bool* out, uint64 out_size);
        ]]
    end

    do -- Global Symbol Table
        NavWorld = {
            Create             = function(physics)
                local _instance = libphx.NavWorld_Create(physics)
                return Core.ManagedObject(_instance, libphx.NavWorld_Free)
            end,
            AxisCount          = libphx.NavWorld_AxisCount,
        }

        if onDef_NavWorld then onDef_NavWorld(NavWorld, mt) end
        NavWorld = setmetatable(NavWorld, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('NavWorld')
        local mt = {
            __index = {
                agentCount         = libphx.NavWorld_AgentCount,
                addAgent           = libphx.NavWorld_AddAgent,
                addObstacle        = libphx.NavWorld_AddObstacle,
                setGoal            = libphx.NavWorld_SetGoal,
                clearGoal          = libphx.NavWorld_ClearGoal,
                setAvoidanceTuning = libphx.NavWorld_SetAvoidanceTuning,
                step               = libphx.NavWorld_Step,
                copyAxes           = libphx.NavWorld_CopyAxes,
                copyArrived        = libphx.NavWorld_CopyArrived,
            },
        }

        if onDef_NavWorld_t then onDef_NavWorld_t(t, mt) end
        NavWorld_t = ffi.metatype(t, mt)
    end

    return NavWorld
end

return Loader
