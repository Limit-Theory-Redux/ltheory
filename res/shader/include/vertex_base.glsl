/* -- Vertex stage common code -----------------------------------------------------
   Attributes, varyings, `VS_BEGIN`/`VS_END` and the logarithmic depth. Included
   by `vertex.glsl` (per-draw `mWorld`/`mWorldIT` as loose uniforms) and by
   `vertex_scene.glsl` (the same two matrices from the group-2 `DrawBlock`).
   Needs `common` (for `Fcoef`) and `view_block` included first.
----------------------------------------------------------------------------- */

in vec3 vertex_position;
in vec3 vertex_normal;
in vec2 vertex_uv;
in vec3 vertex_color;

out vec2 uv;
out vec3 pos;
out vec3 normal;
out vec3 vertNormal;
out vec3 vertPos;
out float flogz;

#define VS_BEGIN                                                               \
  uv = vertex_uv;                                                              \
  vertPos = vertex_position;                                                   \
  vertNormal = vertex_normal;                                                  \

#define VS_END                                                                 \
  gl_Position = logDepth(gl_Position);

vec4 logDepth(vec4 p) {
  p.z = log2(max(1e-6, 1.0 + abs(p.w))) * Fcoef - 1.0;
  p.z *= p.w;
  flogz = 1.0 + p.w;
  return p;
}
