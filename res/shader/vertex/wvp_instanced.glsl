/* -- World-View-Projection (Instanced) ------------------------------------------
   Instanced counterpart to `wvp.glsl`: same output contract (`pos`, `normal`,
   `objPos`, and the varyings `vertex.glsl` provides), but `mWorld`/`mWorldIT`
   come from per-instance vertex attributes (see `instanced.glsl`) instead of
   a uniform - use with `InstanceBatch`/`DrawInstancedWithData`, never with a
   plain `DrawMesh`.
----------------------------------------------------------------------------- */

#include instanced

out vec3 objPos;

void main() {
  VS_INSTANCED_BEGIN
  normal = normalize((mWorldIT * vec4(vertex_normal, 0)).xyz);
  vec4 v = vec4(vertex_position, 1.0);
  objPos = v.xyz;
  vec4 wp = mWorld * v;
  pos = wp.xyz;
  gl_Position = mProj * (mView * wp);
  VS_INSTANCED_END
}
