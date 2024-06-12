-- Trigger ---------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct Trigger {} Trigger;
    ]]

    return 1, 'Trigger'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Trigger

    do -- C Definitions
        ffi.cdef [[
            void       Trigger_Free             (Trigger*);
            Trigger*   Trigger_CreateBox        (Vec3f const* halfExtents);
            void       Trigger_Attach           (Trigger*, RigidBody* parent, Vec3f const* offset);
            void       Trigger_Detach           (Trigger*, RigidBody* parent);
            void       Trigger_GetBoundingBox   (Trigger const*, Box3f* out);
            int        Trigger_GetContentsCount (Trigger*);
            RigidBody* Trigger_GetContents      (Trigger const*, int i);
            void       Trigger_SetCollisionMask (Trigger*, uint32 mask);
            void       Trigger_SetPos           (Trigger*, Position const* pos);
            void       Trigger_SetPosLocal      (Trigger*, Position const* pos);
            void       Trigger_GetPos           (Trigger const*, Position* out);
            void       Trigger_GetPosLocal      (Trigger const*, Position* out);
            RigidBody* Trigger_GetParent        (Trigger*);
        ]]
    end

    do -- Global Symbol Table
        Trigger = {
            CreateBox        = function(...)
                local instance = libphx.Trigger_CreateBox(...)
                return Core.ManagedObject(instance, libphx.Trigger_Free)
            end,
        }

        if onDef_Trigger then onDef_Trigger(Trigger, mt) end
        Trigger = setmetatable(Trigger, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Trigger')
        local mt = {
            __index = {
                attach           = libphx.Trigger_Attach,
                detach           = libphx.Trigger_Detach,
                getBoundingBox   = libphx.Trigger_GetBoundingBox,
                getContentsCount = libphx.Trigger_GetContentsCount,
                getContents      = libphx.Trigger_GetContents,
                setCollisionMask = libphx.Trigger_SetCollisionMask,
                setPos           = libphx.Trigger_SetPos,
                setPosLocal      = libphx.Trigger_SetPosLocal,
                getPos           = libphx.Trigger_GetPos,
                getPosLocal      = libphx.Trigger_GetPosLocal,
                getParent        = libphx.Trigger_GetParent,
            },
        }

        if onDef_Trigger_t then onDef_Trigger_t(t, mt) end
        Trigger_t = ffi.metatype(t, mt)
    end

    return Trigger
end

return Loader
