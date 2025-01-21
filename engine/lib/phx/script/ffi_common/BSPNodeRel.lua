-- BSPNodeRel ------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('libphx').lib
local BSPNodeRel

do -- C Definitions
    ffi.cdef [[
    BSPNodeRel BSP_NODE_REL_PARENT;
    BSPNodeRel BSP_NODE_REL_BACK;
    BSPNodeRel BSP_NODE_REL_FRONT;
  ]]
end

do -- Global Symbol Table
    BSPNodeRel = {
        Parent = libphx.BSP_NODE_REL_PARENT,
        Back   = libphx.BSP_NODE_REL_BACK,
        Front  = libphx.BSP_NODE_REL_FRONT,
    }

    if onDef_BSPNodeRel then onDef_BSPNodeRel(BSPNodeRel, mt) end
    BSPNodeRel = setmetatable(BSPNodeRel, mt)
end

return BSPNodeRel
