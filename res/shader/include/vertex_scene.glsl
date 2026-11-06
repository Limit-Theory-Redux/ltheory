/* -- Vertex stage of scene meshes -------------------------------------------------
   Like `vertex.glsl`, but `mWorld`/`mWorldIT` come from the group-2 `DrawBlock`
   (see `draw_block.glsl`) that `SceneList:submit` writes per drawn mesh, not
   from loose uniforms.
----------------------------------------------------------------------------- */

#include common
#include view_block
#include draw_block
#include vertex_base
