/* -- Shape parameter block (group 2) --------------------------------------------
   Named parameters of the UI shapes that read more than their vertex
   attributes (`ui/stroke`: its look). The batcher writes one block per run of
   vertices into the uniform ring and binds it as the first block of group 2;
   each vertex carries the offset (in vec4s) of its own values, so primitives
   with different values share one draw. The Rust side is
   render/gpu/shape_params.rs (`SHAPE_PARAM_VEC4S`); each shader declares its
   parameter list in Rust (`ShaderParams`, e.g. `STROKE_LOOK` in
   render/stroke.rs) and names the slots with defines.

   Usage: #include shape_params
----------------------------------------------------------------------------- */

#group 2
layout(std140) uniform ShapeParams {
    vec4 shapeParams[64];
};
