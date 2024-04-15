-- Physics ---------------------------------------------------------------------

---@class Physics
---@field Create fun(): Physics
---@field AddRigidBody fun(self, rigid_body: RigidBody)
---@field RemoveRigidBody fun(self, rigid_body: RigidBody)
---@field AddTrigger fun(self, trigger: Trigger)
---@field RemoveTrigger fun(self, trigger: Trigger)
---@field Update fun(self, dt: number)
---@field GetNextCollision fun(self, iterator: Collision): boolean
---@field RayCast fun(self, ray: Ray, result: RayCastResult)
---@field SphereCast fun(self, sphere: Sphere, result: ShapeCastResult)
---@field BoxCast fun(self, pos: Vec3, rot: Quat, half_extents: Vec3, result: ShapeCastResult)
---@field SphereOverlap fun(self, sphere: Sphere): boolean
---@field BoxOverlap fun(self, pos: Vec3, rot: Quat, half_extents: Vec3): boolean
---@field DrawBoundingBoxesLocal fun(self)
---@field DrawBoundingBoxesWorld fun(self)
---@field DrawWireframes fun(self)

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
            void     Physics_DrawWireframes         (Physics*);
        ]]
    end

    do -- Global Symbol Table
        Physics = {
            ---@return Physics
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
                -- Adds this rigid body to this physics world if it doesn't exist, otherwise do nothing.
                --
                -- Automatically adds all attached Triggers. Automatically adds all
                -- attached children and their Triggers.
                ---@param rigid_body RigidBody
                addRigidBody           = libphx.Physics_AddRigidBody,
                -- Removes this rigid body from this physics world if it's added, otherwise do nothing.
                --
                -- Automatically removes all attached Triggers. Automatically removes all
                -- attached children and their Triggers.
                ---@param rigid_body RigidBody
                removeRigidBody        = libphx.Physics_RemoveRigidBody,
                ---@param trigger Trigger
                addTrigger             = libphx.Physics_AddTrigger,
                ---@param trigger Trigger
                removeTrigger          = libphx.Physics_RemoveTrigger,
                ---@param dt number
                update                 = libphx.Physics_Update,
                -- This will fill the collision object with the collision information.
                --
                -- Will include results for both child and parent RigidBodys that are
                -- colliding. Will not include Triggers.
                ---@param iterator Collision
                ---@return boolean
                getNextCollision       = libphx.Physics_GetNextCollision,
                ---@param ray Ray
                ---@param [out] RayCastResult
                rayCast                = libphx.Physics_RayCast,
                -- Results are unsorted and will include child objects.
                --
                -- The array stored inside ShapeCastResult is valid until the next call to sphere_cast.
                ---@param sphere Sphere
                ---@param [out] ShapeCastResult
                sphereCast             = libphx.Physics_SphereCast,
                -- Results are unsorted and will include child objects.
                --
                -- The array stored inside ShapeCastResult is valid until the next call to box_cast.
                ---@param pos Vec3
                ---@param rot Quat
                ---@param half_extents Vec3
                ---@param [out] ShapeCastResult
                boxCast                = libphx.Physics_BoxCast,
                ---@param sphere Sphere
                ---@return boolean
                sphereOverlap          = libphx.Physics_SphereOverlap,
                ---@param pos Vec3
                ---@param rot Quat
                ---@param half_extents Vec3
                ---@return boolean
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
