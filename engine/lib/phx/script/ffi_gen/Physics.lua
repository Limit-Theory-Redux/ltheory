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
            bool     Physics_GetNextCollision       (Physics const*, Collision* iterator);
            void     Physics_RayCast                (Physics const*, Ray const* ray, RayCastResult* out);
            void     Physics_SphereCast             (Physics const*, Sphere const* sphere, ShapeCastResult* out);
            void     Physics_BoxCast                (Physics const*, Vec3f const* pos, Quat const* rot, Vec3f const* halfExtents, ShapeCastResult* out);
            bool     Physics_SphereOverlap          (Physics const*, Sphere const* sphere);
            bool     Physics_BoxOverlap             (Physics const*, Vec3f const* pos, Quat const* rot, Vec3f const* halfExtents);
            void     Physics_DrawBoundingBoxesLocal (Physics const*);
            void     Physics_DrawBoundingBoxesWorld (Physics const*);
            void     Physics_DrawWireframes         (Physics*, Shader* shader, Position const* eye);
        ]]
    end

    do -- Global Symbol Table
        Physics = {
            Create                 = function(...)
                local instance = libphx.Physics_Create(...)
                return Core.ManagedObject(instance, libphx.Physics_Free)
            end,
        }

        if onDef_Physics then onDef_Physics(Physics, mt) end
        Physics = setmetatable(Physics, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Physics')
        local mt = {
            __index = {
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
                drawBoundingBoxesLocal = libphx.Physics_DrawBoundingBoxesLocal,
                drawBoundingBoxesWorld = libphx.Physics_DrawBoundingBoxesWorld,
                drawWireframes         = libphx.Physics_DrawWireframes,
            },
        }

        if onDef_Physics_t then onDef_Physics_t(t, mt) end
        Physics_t = ffi.metatype(t, mt)
    end

    return Physics
end

return Loader
