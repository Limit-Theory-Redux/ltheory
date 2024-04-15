-- RigidBody -------------------------------------------------------------------

---@class RigidBody
---@field CreateBox fun(): RigidBody
---@field CreateBoxFromMesh fun(mesh: Mesh): RigidBody
---@field CreateSphere fun(): RigidBody
---@field CreateSphereFromMesh fun(mesh: Mesh): RigidBody
---@field CreateConvexHullFromMesh fun(mesh: Mesh): RigidBody
---@field CreateConvexDecompositionFromMesh fun(mesh: Mesh): RigidBody
---@field CreateTrimeshFromMesh fun(mesh: Mesh): RigidBody
---@field GetParentBody fun(self): RigidBody
---@field ApplyForce fun(self, force: Vec3)
---@field ApplyTorque fun(self, torque: Vec3)
---@field Attach fun(self, child: RigidBody, pos: Vec3, rot: Quat)
---@field Detach fun(self, child: RigidBody)
---@field GetBoundingBox fun(self, result: Box3)
---@field GetBoundingBoxCompound fun(self, result: Box3)
---@field GetBoundingBoxLocal fun(self, result: Box3)
---@field GetBoundingBoxLocalCompound fun(self, result: Box3)
---@field GetBoundingRadius fun(self): number
---@field GetBoundingRadiusCompound fun(self): number
---@field GetSpeed fun(self): number
---@field GetToWorldMatrix fun(self): Matrix
---@field GetToLocalMatrix fun(self): Matrix
---@field GetVelocity fun(self, result: Vec3)
---@field GetVelocityA fun(self, result: Vec3)
---@field SetCollidable fun(self, collidable: boolean)
---@field SetCollisionGroup fun(self, group: integer)
---@field SetCollisionMask fun(self, mask: integer)
---@field SetDrag fun(self, linear: number, angular: number)
---@field SetFriction fun(self, friction: number)
---@field SetKinematic fun(self, kinematic: boolean)
---@field SetRestitution fun(self, restitution: number)
---@field SetSleepThreshold fun(self, linear: number, angular: number)
---@field GetMass fun(self): number
---@field SetMass fun(self, mass: number)
---@field GetPos fun(self, result: Vec3)
---@field GetPosLocal fun(self, result: Vec3)
---@field SetPos fun(self, pos: Vec3)
---@field SetPosLocal fun(self, pos: Vec3)
---@field GetRot fun(self, result: Quat)
---@field GetRotLocal fun(self, result: Quat)
---@field SetRot fun(self, rot: Quat)
---@field SetRotLocal fun(self, rot: Quat)
---@field GetScale fun(self): number
---@field SetScale fun(self, scale: number)
---@field DistanceTo fun(self, target: RigidBody): number
---@field IsSleeping fun(self): boolean

local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct RigidBody {} RigidBody;
    ]]

    return 1, 'RigidBody'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local RigidBody

    do -- C Definitions
        ffi.cdef [[
            void       RigidBody_Free                              (RigidBody*);
            RigidBody* RigidBody_CreateBox                         ();
            RigidBody* RigidBody_CreateBoxFromMesh                 (Mesh* mesh);
            RigidBody* RigidBody_CreateSphere                      ();
            RigidBody* RigidBody_CreateSphereFromMesh              (Mesh* mesh);
            RigidBody* RigidBody_CreateConvexHullFromMesh          (Mesh const* mesh);
            RigidBody* RigidBody_CreateConvexDecompositionFromMesh (Mesh const* mesh);
            RigidBody* RigidBody_CreateTrimeshFromMesh             (Mesh const* mesh);
            RigidBody* RigidBody_GetParentBody                     (RigidBody const*);
            void       RigidBody_ApplyForce                        (RigidBody*, Vec3f const* force);
            void       RigidBody_ApplyTorque                       (RigidBody*, Vec3f const* torque);
            void       RigidBody_Attach                            (RigidBody*, RigidBody* child, Vec3f const* pos, Quat const* rot);
            void       RigidBody_Detach                            (RigidBody*, RigidBody* child);
            void       RigidBody_GetBoundingBox                    (RigidBody const*, Box3f* out);
            void       RigidBody_GetBoundingBoxCompound            (RigidBody const*, Box3f* out);
            void       RigidBody_GetBoundingBoxLocal               (RigidBody const*, Box3f* out);
            void       RigidBody_GetBoundingBoxLocalCompound       (RigidBody const*, Box3f* out);
            float      RigidBody_GetBoundingRadius                 (RigidBody const*);
            float      RigidBody_GetBoundingRadiusCompound         (RigidBody const*);
            float      RigidBody_GetSpeed                          (RigidBody const*);
            Matrix*    RigidBody_GetToWorldMatrix                  (RigidBody const*);
            Matrix*    RigidBody_GetToLocalMatrix                  (RigidBody const*);
            void       RigidBody_GetVelocity                       (RigidBody const*, Vec3f* out);
            void       RigidBody_GetVelocityA                      (RigidBody const*, Vec3f* out);
            void       RigidBody_SetCollidable                     (RigidBody*, bool collidable);
            void       RigidBody_SetCollisionGroup                 (RigidBody*, uint32 group);
            void       RigidBody_SetCollisionMask                  (RigidBody*, uint32 mask);
            void       RigidBody_SetDrag                           (RigidBody*, float linear, float angular);
            void       RigidBody_SetFriction                       (RigidBody*, float friction);
            void       RigidBody_SetKinematic                      (RigidBody*, bool kinematic);
            void       RigidBody_SetRestitution                    (RigidBody*, float restitution);
            void       RigidBody_SetSleepThreshold                 (RigidBody*, float linear, float angular);
            float      RigidBody_GetMass                           (RigidBody const*);
            void       RigidBody_SetMass                           (RigidBody*, float mass);
            void       RigidBody_GetPos                            (RigidBody const*, Vec3f* out);
            void       RigidBody_GetPosLocal                       (RigidBody const*, Vec3f* out);
            void       RigidBody_SetPos                            (RigidBody*, Vec3f const* pos);
            void       RigidBody_SetPosLocal                       (RigidBody*, Vec3f const* pos);
            void       RigidBody_GetRot                            (RigidBody const*, Quat* out);
            void       RigidBody_GetRotLocal                       (RigidBody const*, Quat* out);
            void       RigidBody_SetRot                            (RigidBody*, Quat* rot);
            void       RigidBody_SetRotLocal                       (RigidBody*, Quat const* rot);
            float      RigidBody_GetScale                          (RigidBody const*);
            void       RigidBody_SetScale                          (RigidBody*, float scale);
            float      RigidBody_DistanceTo                        (RigidBody const*, RigidBody const* target);
            bool       RigidBody_IsSleeping                        (RigidBody const*);
        ]]
    end

    do -- Global Symbol Table
        RigidBody = {
            ---@return RigidBody
            CreateBox                         = function(...)
                local instance = libphx.RigidBody_CreateBox(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
            ---@param mesh Mesh
            ---@return RigidBody
            CreateBoxFromMesh                 = function(...)
                local instance = libphx.RigidBody_CreateBoxFromMesh(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
            ---@return RigidBody
            CreateSphere                      = function(...)
                local instance = libphx.RigidBody_CreateSphere(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
            ---@param mesh Mesh
            ---@return RigidBody
            CreateSphereFromMesh              = function(...)
                local instance = libphx.RigidBody_CreateSphereFromMesh(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
            ---@param mesh Mesh
            ---@return RigidBody
            CreateConvexHullFromMesh          = function(...)
                local instance = libphx.RigidBody_CreateConvexHullFromMesh(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
            ---@param mesh Mesh
            ---@return RigidBody
            CreateConvexDecompositionFromMesh = function(...)
                local instance = libphx.RigidBody_CreateConvexDecompositionFromMesh(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
            ---@param mesh Mesh
            ---@return RigidBody
            CreateTrimeshFromMesh             = function(...)
                local instance = libphx.RigidBody_CreateTrimeshFromMesh(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
        }

        if onDef_RigidBody then onDef_RigidBody(RigidBody, mt) end
        RigidBody = setmetatable(RigidBody, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('RigidBody')
        local mt = {
            __index = {
                -- Return a reference to the parent rigid body, that we can guarantee
                -- has a lifetime as long as self.
                ---@return RigidBody
                getParentBody               = libphx.RigidBody_GetParentBody,
                ---@param force Vec3
                applyForce                  = libphx.RigidBody_ApplyForce,
                ---@param torque Vec3
                applyTorque                 = libphx.RigidBody_ApplyTorque,
                -- Adds another rigid body as a child of this rigid body. This means that
                -- the child's position will be controlled by `self`.
                --
                -- Only a single level of attachment is supported. Child objects do not
                -- affect the mass or inertia of the parent. Position is relative to the
                -- unscaled parent. i.e. it will be multiplied by the current scale. This
                -- function is O(1). Warning: if one object is attached to another and a
                -- third object happens to be between them this may trap the third object.
                -- The same issue may occur when spawning one compound inside another.
                --
                -- This function expects that the child is not already in the physics
                -- world, as it will add it if the parent is already in the world.
                --
                -- This function assumes that `self` is not already a child.
                ---@param child RigidBody
                ---@param pos Vec3
                ---@param rot Quat
                attach                      = libphx.RigidBody_Attach,
                -- Removes a rigid body as a child of this rigid body. This means that
                -- the child's will be under control of it's own position.
                --
                -- This function will result in a child that is not in the world anymore,
                -- so it will need to be re-added with physics.add_rigid_body(...).
                --
                -- This function assumes that `self` is not already a child.
                ---@param child RigidBody
                detach                      = libphx.RigidBody_Detach,
                -- Calculates the bounding box.
                ---@param [out] Box3
                getBoundingBox              = libphx.RigidBody_GetBoundingBox,
                -- Calculates the compound bounding box.
                ---@param [out] Box3
                getBoundingBoxCompound      = libphx.RigidBody_GetBoundingBoxCompound,
                -- Calculates the local bounding box.
                ---@param [out] Box3
                getBoundingBoxLocal         = libphx.RigidBody_GetBoundingBoxLocal,
                -- Calculates the local compound bounding box.
                ---@param [out] Box3
                getBoundingBoxLocalCompound = libphx.RigidBody_GetBoundingBoxLocalCompound,
                ---@return number
                getBoundingRadius           = libphx.RigidBody_GetBoundingRadius,
                ---@return number
                getBoundingRadiusCompound   = libphx.RigidBody_GetBoundingRadiusCompound,
                ---@return number
                getSpeed                    = libphx.RigidBody_GetSpeed,
                -- Returns the local -> world matrix for this rigid body.
                ---@return Matrix
                getToWorldMatrix            = function(...)
                    local instance = libphx.RigidBody_GetToWorldMatrix(...)
                    return Core.ManagedObject(instance, libphx.Matrix_Free)
                end,
                -- Returns the world -> local matrix for this rigid body.
                ---@return Matrix
                getToLocalMatrix            = function(...)
                    local instance = libphx.RigidBody_GetToLocalMatrix(...)
                    return Core.ManagedObject(instance, libphx.Matrix_Free)
                end,
                ---@param [out] Vec3
                getVelocity                 = libphx.RigidBody_GetVelocity,
                ---@param [out] Vec3
                getVelocityA                = libphx.RigidBody_GetVelocityA,
                -- When disabled, the object will pass through others without colliding
                -- and will not be returned from ray or shape casts.
                ---@param collidable boolean
                setCollidable               = libphx.RigidBody_SetCollidable,
                ---@param group integer
                setCollisionGroup           = libphx.RigidBody_SetCollisionGroup,
                ---@param mask integer
                setCollisionMask            = libphx.RigidBody_SetCollisionMask,
                ---@param linear number
                ---@param angular number
                setDrag                     = libphx.RigidBody_SetDrag,
                ---@param friction number
                setFriction                 = libphx.RigidBody_SetFriction,
                ---@param kinematic boolean
                setKinematic                = libphx.RigidBody_SetKinematic,
                ---@param restitution number
                setRestitution              = libphx.RigidBody_SetRestitution,
                ---@param linear number
                ---@param angular number
                setSleepThreshold           = libphx.RigidBody_SetSleepThreshold,
                ---@return number
                getMass                     = libphx.RigidBody_GetMass,
                -- The mass of child objects does not affect the mass or inertia of the parent
                ---@param mass number
                setMass                     = libphx.RigidBody_SetMass,
                -- Children return the parent position.
                ---@param [out] Vec3
                getPos                      = libphx.RigidBody_GetPos,
                -- Local coordinates are relative to the parent *before* scaling.
                ---@param [out] Vec3
                getPosLocal                 = libphx.RigidBody_GetPosLocal,
                ---@param pos Vec3
                setPos                      = libphx.RigidBody_SetPos,
                -- Local coordinates are relative to the parent *before* scaling. The
                -- given position will be multiplied by the parent's scale.
                ---@param pos Vec3
                setPosLocal                 = libphx.RigidBody_SetPosLocal,
                ---@param [out] Quat
                getRot                      = libphx.RigidBody_GetRot,
                ---@param [out] Quat
                getRotLocal                 = libphx.RigidBody_GetRotLocal,
                ---@param rot Quat
                setRot                      = libphx.RigidBody_SetRot,
                ---@param rot Quat
                setRotLocal                 = libphx.RigidBody_SetRotLocal,
                ---@return number
                getScale                    = libphx.RigidBody_GetScale,
                -- When called on a parent object the positions of all children will be
                -- multiplied such that they retain the same relative position. Child
                -- scale is not affected by parent scale (i.e. it is not inherited).
                ---@param scale number
                setScale                    = libphx.RigidBody_SetScale,
                ---@param target RigidBody
                ---@return number
                distanceTo                  = libphx.RigidBody_DistanceTo,
                ---@return boolean
                isSleeping                  = libphx.RigidBody_IsSleeping,
            },
        }

        if onDef_RigidBody_t then onDef_RigidBody_t(t, mt) end
        RigidBody_t = ffi.metatype(t, mt)
    end

    return RigidBody
end

return Loader
