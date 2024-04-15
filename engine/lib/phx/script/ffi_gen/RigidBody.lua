-- RigidBody -------------------------------------------------------------------
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
            ---@return RigidBody*
            CreateBox                         = function(...)
                local instance = libphx.RigidBody_CreateBox(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
            ---@param mesh Mesh*
            ---@return RigidBody*
            CreateBoxFromMesh                 = function(...)
                local instance = libphx.RigidBody_CreateBoxFromMesh(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
            ---@return RigidBody*
            CreateSphere                      = function(...)
                local instance = libphx.RigidBody_CreateSphere(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
            ---@param mesh Mesh*
            ---@return RigidBody*
            CreateSphereFromMesh              = function(...)
                local instance = libphx.RigidBody_CreateSphereFromMesh(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
            ---@param mesh Mesh const*
            ---@return RigidBody*
            CreateConvexHullFromMesh          = function(...)
                local instance = libphx.RigidBody_CreateConvexHullFromMesh(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
            ---@param mesh Mesh const*
            ---@return RigidBody*
            CreateConvexDecompositionFromMesh = function(...)
                local instance = libphx.RigidBody_CreateConvexDecompositionFromMesh(...)
                return Core.ManagedObject(instance, libphx.RigidBody_Free)
            end,
            ---@param mesh Mesh const*
            ---@return RigidBody*
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
                ---@return RigidBody*
                getParentBody               = libphx.RigidBody_GetParentBody,
                ---@param force Vec3f const*
                applyForce                  = libphx.RigidBody_ApplyForce,
                ---@param torque Vec3f const*
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
                ---@param child RigidBody*
                ---@param pos Vec3f const*
                ---@param rot Quat const*
                attach                      = libphx.RigidBody_Attach,
                -- Removes a rigid body as a child of this rigid body. This means that
                -- the child's will be under control of it's own position.
                --
                -- This function will result in a child that is not in the world anymore,
                -- so it will need to be re-added with physics.add_rigid_body(...).
                --
                -- This function assumes that `self` is not already a child.
                ---@param child RigidBody*
                detach                      = libphx.RigidBody_Detach,
                -- Calculates the bounding box.
                ---@param [out] Box3f
                getBoundingBox              = libphx.RigidBody_GetBoundingBox,
                -- Calculates the compound bounding box.
                ---@param [out] Box3f
                getBoundingBoxCompound      = libphx.RigidBody_GetBoundingBoxCompound,
                -- Calculates the local bounding box.
                ---@param [out] Box3f
                getBoundingBoxLocal         = libphx.RigidBody_GetBoundingBoxLocal,
                -- Calculates the local compound bounding box.
                ---@param [out] Box3f
                getBoundingBoxLocalCompound = libphx.RigidBody_GetBoundingBoxLocalCompound,
                ---@return float
                getBoundingRadius           = libphx.RigidBody_GetBoundingRadius,
                ---@return float
                getBoundingRadiusCompound   = libphx.RigidBody_GetBoundingRadiusCompound,
                ---@return float
                getSpeed                    = libphx.RigidBody_GetSpeed,
                -- Returns the local -> world matrix for this rigid body.
                ---@return Matrix*
                getToWorldMatrix            = function(...)
                    local instance = libphx.RigidBody_GetToWorldMatrix(...)
                    return Core.ManagedObject(instance, libphx.Matrix_Free)
                end,
                -- Returns the world -> local matrix for this rigid body.
                ---@return Matrix*
                getToLocalMatrix            = function(...)
                    local instance = libphx.RigidBody_GetToLocalMatrix(...)
                    return Core.ManagedObject(instance, libphx.Matrix_Free)
                end,
                ---@param [out] Vec3f
                getVelocity                 = libphx.RigidBody_GetVelocity,
                ---@param [out] Vec3f
                getVelocityA                = libphx.RigidBody_GetVelocityA,
                -- When disabled, the object will pass through others without colliding
                -- and will not be returned from ray or shape casts.
                ---@param collidable bool
                setCollidable               = libphx.RigidBody_SetCollidable,
                ---@param group uint32
                setCollisionGroup           = libphx.RigidBody_SetCollisionGroup,
                ---@param mask uint32
                setCollisionMask            = libphx.RigidBody_SetCollisionMask,
                ---@param linear float
                ---@param angular float
                setDrag                     = libphx.RigidBody_SetDrag,
                ---@param friction float
                setFriction                 = libphx.RigidBody_SetFriction,
                ---@param kinematic bool
                setKinematic                = libphx.RigidBody_SetKinematic,
                ---@param restitution float
                setRestitution              = libphx.RigidBody_SetRestitution,
                ---@param linear float
                ---@param angular float
                setSleepThreshold           = libphx.RigidBody_SetSleepThreshold,
                ---@return float
                getMass                     = libphx.RigidBody_GetMass,
                -- The mass of child objects does not affect the mass or inertia of the parent
                ---@param mass float
                setMass                     = libphx.RigidBody_SetMass,
                -- Children return the parent position.
                ---@param [out] Vec3f
                getPos                      = libphx.RigidBody_GetPos,
                -- Local coordinates are relative to the parent *before* scaling.
                ---@param [out] Vec3f
                getPosLocal                 = libphx.RigidBody_GetPosLocal,
                ---@param pos Vec3f const*
                setPos                      = libphx.RigidBody_SetPos,
                -- Local coordinates are relative to the parent *before* scaling. The
                -- given position will be multiplied by the parent's scale.
                ---@param pos Vec3f const*
                setPosLocal                 = libphx.RigidBody_SetPosLocal,
                ---@param [out] Quat*
                getRot                      = libphx.RigidBody_GetRot,
                ---@param [out] Quat*
                getRotLocal                 = libphx.RigidBody_GetRotLocal,
                ---@param rot Quat*
                setRot                      = libphx.RigidBody_SetRot,
                ---@param rot Quat const*
                setRotLocal                 = libphx.RigidBody_SetRotLocal,
                ---@return float
                getScale                    = libphx.RigidBody_GetScale,
                -- When called on a parent object the positions of all children will be
                -- multiplied such that they retain the same relative position. Child
                -- scale is not affected by parent scale (i.e. it is not inherited).
                ---@param scale float
                setScale                    = libphx.RigidBody_SetScale,
                ---@param target RigidBody const*
                ---@return float
                distanceTo                  = libphx.RigidBody_DistanceTo,
                ---@return bool
                isSleeping                  = libphx.RigidBody_IsSleeping,
            },
        }

        if onDef_RigidBody_t then onDef_RigidBody_t(t, mt) end
        RigidBody_t = ffi.metatype(t, mt)
    end

    return RigidBody
end

return Loader
