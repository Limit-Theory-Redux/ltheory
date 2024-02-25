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
            bool       RigidBody_IsSleeping                        (RigidBody const*);
        ]]
    end

    do -- Global Symbol Table
        RigidBody = {
            CreateBox                         = function(...)
                local instance = libphx.RigidBody_CreateBox(...)
                ffi.gc(instance, libphx.RigidBody_Free)
                return instance
            end,
            CreateBoxFromMesh                 = function(...)
                local instance = libphx.RigidBody_CreateBoxFromMesh(...)
                ffi.gc(instance, libphx.RigidBody_Free)
                return instance
            end,
            CreateSphere                      = function(...)
                local instance = libphx.RigidBody_CreateSphere(...)
                ffi.gc(instance, libphx.RigidBody_Free)
                return instance
            end,
            CreateSphereFromMesh              = function(...)
                local instance = libphx.RigidBody_CreateSphereFromMesh(...)
                ffi.gc(instance, libphx.RigidBody_Free)
                return instance
            end,
            CreateConvexHullFromMesh          = function(...)
                local instance = libphx.RigidBody_CreateConvexHullFromMesh(...)
                ffi.gc(instance, libphx.RigidBody_Free)
                return instance
            end,
            CreateConvexDecompositionFromMesh = function(...)
                local instance = libphx.RigidBody_CreateConvexDecompositionFromMesh(...)
                ffi.gc(instance, libphx.RigidBody_Free)
                return instance
            end,
            CreateTrimeshFromMesh             = function(...)
                local instance = libphx.RigidBody_CreateTrimeshFromMesh(...)
                ffi.gc(instance, libphx.RigidBody_Free)
                return instance
            end,
        }

        if onDef_RigidBody then onDef_RigidBody(RigidBody, mt) end
        RigidBody = setmetatable(RigidBody, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('RigidBody')
        local mt = {
            __index = {
                getParentBody               = libphx.RigidBody_GetParentBody,
                applyForce                  = libphx.RigidBody_ApplyForce,
                applyTorque                 = libphx.RigidBody_ApplyTorque,
                attach                      = libphx.RigidBody_Attach,
                detach                      = libphx.RigidBody_Detach,
                getBoundingBox              = libphx.RigidBody_GetBoundingBox,
                getBoundingBoxCompound      = libphx.RigidBody_GetBoundingBoxCompound,
                getBoundingBoxLocal         = libphx.RigidBody_GetBoundingBoxLocal,
                getBoundingBoxLocalCompound = libphx.RigidBody_GetBoundingBoxLocalCompound,
                getBoundingRadius           = libphx.RigidBody_GetBoundingRadius,
                getBoundingRadiusCompound   = libphx.RigidBody_GetBoundingRadiusCompound,
                getSpeed                    = libphx.RigidBody_GetSpeed,
                getToWorldMatrix            = function(...)
                    local instance = libphx.RigidBody_GetToWorldMatrix(...)
                    ffi.gc(instance, libphx.Matrix_Free)
                    return instance
                end,
                getToLocalMatrix            = function(...)
                    local instance = libphx.RigidBody_GetToLocalMatrix(...)
                    ffi.gc(instance, libphx.Matrix_Free)
                    return instance
                end,
                getVelocity                 = libphx.RigidBody_GetVelocity,
                getVelocityA                = libphx.RigidBody_GetVelocityA,
                setCollidable               = libphx.RigidBody_SetCollidable,
                setCollisionGroup           = libphx.RigidBody_SetCollisionGroup,
                setCollisionMask            = libphx.RigidBody_SetCollisionMask,
                setDrag                     = libphx.RigidBody_SetDrag,
                setFriction                 = libphx.RigidBody_SetFriction,
                setKinematic                = libphx.RigidBody_SetKinematic,
                setRestitution              = libphx.RigidBody_SetRestitution,
                setSleepThreshold           = libphx.RigidBody_SetSleepThreshold,
                getMass                     = libphx.RigidBody_GetMass,
                setMass                     = libphx.RigidBody_SetMass,
                getPos                      = libphx.RigidBody_GetPos,
                getPosLocal                 = libphx.RigidBody_GetPosLocal,
                setPos                      = libphx.RigidBody_SetPos,
                setPosLocal                 = libphx.RigidBody_SetPosLocal,
                getRot                      = libphx.RigidBody_GetRot,
                getRotLocal                 = libphx.RigidBody_GetRotLocal,
                setRot                      = libphx.RigidBody_SetRot,
                setRotLocal                 = libphx.RigidBody_SetRotLocal,
                getScale                    = libphx.RigidBody_GetScale,
                setScale                    = libphx.RigidBody_SetScale,
                isSleeping                  = libphx.RigidBody_IsSleeping,
            },
        }

        if onDef_RigidBody_t then onDef_RigidBody_t(t, mt) end
        RigidBody_t = ffi.metatype(t, mt)
    end

    return RigidBody
end

return Loader
