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
            void       Trigger_SetPos           (Trigger*, Vec3f const* pos);
            void       Trigger_SetPosLocal      (Trigger*, Vec3f const* pos);
            void       Trigger_GetPos           (Trigger const*, Vec3f* out);
            void       Trigger_GetPosLocal      (Trigger const*, Vec3f* out);
            RigidBody* Trigger_GetParent        (Trigger*);
        ]]
    end

    do -- Global Symbol Table
        Trigger = {
            Free             = libphx.Trigger_Free,
            CreateBox        = libphx.Trigger_CreateBox,
            Attach           = libphx.Trigger_Attach,
            Detach           = libphx.Trigger_Detach,
            GetBoundingBox   = libphx.Trigger_GetBoundingBox,
            GetContentsCount = libphx.Trigger_GetContentsCount,
            GetContents      = libphx.Trigger_GetContents,
            SetCollisionMask = libphx.Trigger_SetCollisionMask,
            SetPos           = libphx.Trigger_SetPos,
            SetPosLocal      = libphx.Trigger_SetPosLocal,
            GetPos           = libphx.Trigger_GetPos,
            GetPosLocal      = libphx.Trigger_GetPosLocal,
            GetParent        = libphx.Trigger_GetParent,
        }

        if onDef_Trigger then onDef_Trigger(Trigger, mt) end
        Trigger = setmetatable(Trigger, mt)
    end

    do -- Metatype for class instances
        local t  = ffi.typeof('Trigger')
        local mt = {
            __index = {
                managed          = function(self) return ffi.gc(self, libphx.Trigger_Free) end,
                free             = libphx.Trigger_Free,
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
