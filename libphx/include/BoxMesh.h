#ifndef PHX_BoxMesh
#define PHX_BoxMesh

#include "Common.h"

PHX_API BoxMesh*  BoxMesh_Create   ();
PHX_API void      BoxMesh_Free     (BoxMesh*);

PHX_API void      BoxMesh_Add      (BoxMesh*,
                                    Vec3f const* p, Vec3f const* s, Vec3f const* r, Vec3f const* b);
PHX_API Mesh*     BoxMesh_GetMesh  (BoxMesh*, int res);

#endif
