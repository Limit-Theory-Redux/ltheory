-- Trigger ---------------------------------------------------------------------

---@class Trigger
---@field CreateBox fun(half_extents: Vec3): Trigger
---@field Attach fun(self, parent: RigidBody, offset: Vec3)
---@field Detach fun(self, parent: RigidBody)
---@field GetBoundingBox fun(self, result: Box3)
---@field GetContentsCount fun(self): integer
---@field GetContents fun(self, i: integer): RigidBody
---@field SetCollisionMask fun(self, mask: integer)
---@field SetPos fun(self, pos: Vec3)
---@field SetPosLocal fun(self, pos: Vec3)
---@field GetPos fun(self, result: Vec3)
---@field GetPosLocal fun(self, result: Vec3)
---@field GetParent fun(self): RigidBody

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
            ---@param half_extents Vec3
            ---@return Trigger
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
                ---@param parent RigidBody
                ---@param offset Vec3
                attach           = libphx.Trigger_Attach,
                ---@param parent RigidBody
                detach           = libphx.Trigger_Detach,
                ---@param [out] Box3
                getBoundingBox   = libphx.Trigger_GetBoundingBox,
                ---@return integer
                getContentsCount = libphx.Trigger_GetContentsCount,
                -- Will only include the parent object when a compound is within the trigger.
                ---@param i integer
                ---@return RigidBody
                getContents      = libphx.Trigger_GetContents,
                ---@param mask integer
                setCollisionMask = libphx.Trigger_SetCollisionMask,
                ---@param pos Vec3
                setPos           = libphx.Trigger_SetPos,
                ---@param pos Vec3
                setPosLocal      = libphx.Trigger_SetPosLocal,
                ---@param [out] Vec3
                getPos           = libphx.Trigger_GetPos,
                ---@param [out] Vec3
                getPosLocal      = libphx.Trigger_GetPosLocal,
                ---@return RigidBody
                getParent        = libphx.Trigger_GetParent,
            },
        }

        if onDef_Trigger_t then onDef_Trigger_t(t, mt) end
        Trigger_t = ffi.metatype(t, mt)
    end

    return Trigger
end

return Loader
