-- BoxMesh ---------------------------------------------------------------------
local ffi = require('ffi')
local libphx = require('ffi.libphx').lib
local BoxMesh

do -- C Definitions
   ffi.cdef [[
    BoxMesh* BoxMesh_Create  ();
    void     BoxMesh_Free    (BoxMesh*);
    void     BoxMesh_Add     (BoxMesh*, Vec3f const* p, Vec3f const* s, Vec3f const* r, Vec3f const* b);
    Mesh*    BoxMesh_GetMesh (BoxMesh*, int res);
  ]]
end

do -- Global Symbol Table
   BoxMesh = {
      Create  = libphx.BoxMesh_Create,
      Free    = libphx.BoxMesh_Free,
      Add     = function(self, px, py, pz, sz, sy, sz, rx, ry, rz, bx, by, bz)
         return libphx.BoxMesh_Add(self, Vec3f(px, py, pz), Vec3f(sx, sy, sz), Vec3f(rx, ry, rz), Vec3f(bx, by, bz))
      end,
      GetMesh = libphx.BoxMesh_GetMesh,
   }

   if onDef_BoxMesh then onDef_BoxMesh(BoxMesh, mt) end
   BoxMesh = setmetatable(BoxMesh, mt)
end

do -- Metatype for class instances
   local t  = ffi.typeof('BoxMesh')
   local mt = {
      __index = {
         managed = function(self) return ffi.gc(self, libphx.BoxMesh_Free) end,
         free    = libphx.BoxMesh_Free,
         add     = function(self, px, py, pz, sx, sy, sz, rx, ry, rz, bx, by, bz)
            return libphx.BoxMesh_Add(self, Vec3f(px, py, pz), Vec3f(sx, sy, sz), Vec3f(rx, ry, rz), Vec3f(bx, by, bz))
         end,
         getMesh = libphx.BoxMesh_GetMesh,
      },
   }

   if onDef_BoxMesh_t then onDef_BoxMesh_t(t, mt) end
   BoxMesh_t = ffi.metatype(t, mt)
end

return BoxMesh
