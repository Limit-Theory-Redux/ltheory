-- AUTO GENERATED. DO NOT MODIFY!
-- Collision -------------------------------------------------------------------
local Loader = {}

function Loader.declareType()
    ffi.cdef [[
        typedef struct RigidBody RigidBody;
        typedef struct Collision {
            uint32     index;
            uint32     count;
            RigidBody* body0;
            RigidBody* body1;
        } Collision;
    ]]

    return 1, 'Collision'
end

function Loader.defineType()
    local ffi = require('ffi')
    local libphx = require('libphx').lib
    local Collision

    do -- Global Symbol Table
        Collision = {}

        if onDef_Collision then onDef_Collision(Collision, mt) end
        Collision = setmetatable(Collision, mt)
    end

    return Collision
end

return Loader
