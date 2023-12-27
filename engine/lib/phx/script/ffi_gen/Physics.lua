-- Physics ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Physics {} Physics;
    ]]

    return 1, 'Physics'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Physics

    do -- C Definitions
        ffi.cdef [[
            void     Physics_Free                   (Physics*);
            Physics* Physics_Create                 ();
            void     Physics_AddRigidBody           (Physics*, RigidBody* rigidBody);
            void     Physics_RemoveRigidBody        (Physics*, RigidBody* rigidBody);
            void     Physics_AddTrigger             (Physics*, Trigger* trigger);
            void     Physics_RemoveTrigger          (Physics*, Trigger* trigger);
            void     Physics_Update                 (Physics*, float dt);
            bool     Physics_GetNextCollision       (Physics const*, Collision* c);
            void     Physics_RayCast                (Physics const*, Ray const* ray, RayCastResult* out);
            void     Physics_SphereCast             (Physics*, Sphere const* sphere, ShapeCastResult* out);
            void     Physics_BoxCast                (Physics*, Vec3f const* pos, Quat const* rot, Vec3f const* halfExtents, ShapeCastResult* out);
            bool     Physics_SphereOverlap          (Physics*, Sphere const* sphere);
            bool     Physics_BoxOverlap             (Physics*, Vec3f const* pos, Quat const* rot, Vec3f const* halfExtents);
            void     Physics_PrintProfiling         (Physics*);
            void     Physics_DrawBoundingBoxesLocal (Physics*);
            void     Physics_DrawBoundingBoxesWorld (Physics*);
            void     Physics_DrawTriggers           (Physics*);
            void     Physics_DrawWireframes         (Physics*);
        ]]
    end

    do -- Global Symbol Table
        Physics = {
            Free                   = libphx.Physics_Free,
            Create                 = libphx.Physics_Create,
            AddRigidBody           = libphx.Physics_AddRigidBody,
            RemoveRigidBody        = libphx.Physics_RemoveRigidBody,
            AddTrigger             = libphx.Physics_AddTrigger,
            RemoveTrigger          = libphx.Physics_RemoveTrigger,
            Update                 = libphx.Physics_Update,
            GetNextCollision       = libphx.Physics_GetNextCollision,
            RayCast                = libphx.Physics_RayCast,
            SphereCast             = libphx.Physics_SphereCast,
            BoxCast                = libphx.Physics_BoxCast,
            SphereOverlap          = libphx.Physics_SphereOverlap,
            BoxOverlap             = libphx.Physics_BoxOverlap,
            PrintProfiling         = libphx.Physics_PrintProfiling,
            DrawBoundingBoxesLocal = libphx.Physics_DrawBoundingBoxesLocal,
            DrawBoundingBoxesWorld = libphx.Physics_DrawBoundingBoxesWorld,
            DrawTriggers           = libphx.Physics_DrawTriggers,
            DrawWireframes         = libphx.Physics_DrawWireframes,
        }

        if onDef_Physics then onDef_Physics(Physics, mt) end
        Physics = setmetatable(Physics, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Physics')
        local mt = {
            __index = {
                managed                = function(self) return ffi.gc(self, libphx.Physics_Free) end,
                free                   = libphx.Physics_Free,
                addRigidBody           = libphx.Physics_AddRigidBody,
                removeRigidBody        = libphx.Physics_RemoveRigidBody,
                addTrigger             = libphx.Physics_AddTrigger,
                removeTrigger          = libphx.Physics_RemoveTrigger,
                update                 = libphx.Physics_Update,
                getNextCollision       = libphx.Physics_GetNextCollision,
                rayCast                = libphx.Physics_RayCast,
                sphereCast             = libphx.Physics_SphereCast,
                boxCast                = libphx.Physics_BoxCast,
                sphereOverlap          = libphx.Physics_SphereOverlap,
                boxOverlap             = libphx.Physics_BoxOverlap,
                printProfiling         = libphx.Physics_PrintProfiling,
                drawBoundingBoxesLocal = libphx.Physics_DrawBoundingBoxesLocal,
                drawBoundingBoxesWorld = libphx.Physics_DrawBoundingBoxesWorld,
                drawTriggers           = libphx.Physics_DrawTriggers,
                drawWireframes         = libphx.Physics_DrawWireframes,
            },
        }

        if onDef_Physics_t then onDef_Physics_t(t, mt) end
        Physics_t = ffi.metatype(t, mt)
    end

    return Physics
end

return Loader
