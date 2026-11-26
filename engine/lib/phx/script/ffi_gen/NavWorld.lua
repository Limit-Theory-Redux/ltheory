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
            uint32    NavWorld_SlotCount          (NavWorld const*);
            uint32    NavWorld_AddAgent           (NavWorld*, RigidBody const* body, double linearDrag, double angularDrag, double thrustForward, double thrustRight, double thrustUp, double torquePitch, double torqueYaw, double torqueRoll);
            void      NavWorld_RemoveAgent        (NavWorld*, uint32 id);
            void      NavWorld_SetSensing         (NavWorld*, uint32 mask);
            void      NavWorld_SetArrive          (NavWorld*, uint32 id, Position const* target, double arrivalRadius);
            void      NavWorld_SetArriveRelative  (NavWorld*, uint32 id, RigidBody const* body, Vec3d const* offset, double arrivalRadius);
            void      NavWorld_SetFrameVelocity   (NavWorld*, uint32 id, Vec3d const* velocity);
            void      NavWorld_SetFacing          (NavWorld*, uint32 id, Position const* point);
            void      NavWorld_ClearFacing        (NavWorld*, uint32 id);
            void      NavWorld_SetSeek            (NavWorld*, uint32 id, Position const* target, double arrivalRadius, double passSpeed);
            void      NavWorld_SetSpeedCap        (NavWorld*, uint32 id, double cap);
            void      NavWorld_SetHold            (NavWorld*, uint32 id, double arrivalRadius);
            void      NavWorld_SetPursue          (NavWorld*, uint32 id, RigidBody const* target, double standOff, double arrivalRadius);
            void      NavWorld_SetFollow          (NavWorld*, uint32 id, RigidBody const* leader, double right, double up, double forward, double arrivalRadius);
            void      NavWorld_SetOrbit           (NavWorld*, uint32 id, RigidBody const* centre, double radius, Vec3d const* normal, double arrivalRadius);
            void      NavWorld_ClearBehaviour     (NavWorld*, uint32 id);
            void      NavWorld_SetAvoidanceTuning (NavWorld*, double horizonS, double clearanceRadii, double aimFactor);
            void      NavWorld_SetAvoidancePeriod (NavWorld*, uint32 id, double seconds);
            void      NavWorld_Refresh            (NavWorld*, uint32 id);
            double    NavWorld_SensingRange       (NavWorld const*, uint32 id);
            void      NavWorld_Step               (NavWorld*, double dt);
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
                slotCount          = libphx.NavWorld_SlotCount,
                addAgent           = libphx.NavWorld_AddAgent,
                removeAgent        = libphx.NavWorld_RemoveAgent,
                setSensing         = libphx.NavWorld_SetSensing,
                setArrive          = libphx.NavWorld_SetArrive,
                setArriveRelative  = libphx.NavWorld_SetArriveRelative,
                setFrameVelocity   = libphx.NavWorld_SetFrameVelocity,
                setFacing          = libphx.NavWorld_SetFacing,
                clearFacing        = libphx.NavWorld_ClearFacing,
                setSeek            = libphx.NavWorld_SetSeek,
                setSpeedCap        = libphx.NavWorld_SetSpeedCap,
                setHold            = libphx.NavWorld_SetHold,
                setPursue          = libphx.NavWorld_SetPursue,
                setFollow          = libphx.NavWorld_SetFollow,
                setOrbit           = libphx.NavWorld_SetOrbit,
                clearBehaviour     = libphx.NavWorld_ClearBehaviour,
                setAvoidanceTuning = libphx.NavWorld_SetAvoidanceTuning,
                setAvoidancePeriod = libphx.NavWorld_SetAvoidancePeriod,
                refresh            = libphx.NavWorld_Refresh,
                sensingRange       = libphx.NavWorld_SensingRange,
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
