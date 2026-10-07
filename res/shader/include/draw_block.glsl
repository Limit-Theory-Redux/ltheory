/* -- Draw block (group 2) -------------------------------------------------------
   Per-draw data of scene meshes, std140, written into the uniform ring for
   every drawn mesh by `SceneList:submit` and bound to block binding 8 (group 2,
   dynamic offset). The Rust mirror is `DrawBlock` (render/gpu/draw_block.rs); a
   check at shader link compares size and offsets against what the linker
   reports.

   Usage: #include draw_block

   Provides:
     - mWorld   local -> camera-relative world transform (includes the body's scale)
     - mWorldIT inverse transpose of mWorld, for normals
     - drawScale.x  the body's uniform scale (was the `scale` auto-var)
     - drawUser[7]  per-draw values a material fills in (`MaterialType.perDraw`)

   The block must be the first one a shader declares in group 2.
----------------------------------------------------------------------------- */

#group 2
layout(std140) uniform DrawBlock {
    mat4 mWorld;
    mat4 mWorldIT;
    vec4 drawScale;        // x = uniform scale
    vec4 drawUser[7];      // material-defined per-draw values
};
