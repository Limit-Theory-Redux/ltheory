/* -- Instanced World-View-Projection (asteroids) ----------------------------
   Variant of wvp.glsl for instanced rendering: the model matrix arrives as
   per-instance attributes (locations 4-7, set up by the executor's
   DrawInstancedWithData path) and scale as attribute 9. mWorld/mWorldIT
   uniforms are NOT used; for uniform-scale rigid bodies (asteroids)
   normalize(mat3(model) * vertex_normal) == mWorldIT * vec4(n,0) because
   the scale cancels in normalize.
----------------------------------------------------------------------------- */

#include vertex

#autovar mat4 mView
#autovar mat4 mProj

layout(location = 4) in mat4 modelMatrix;
layout(location = 8) in vec4 instanceColor;
layout(location = 9) in float instanceScale;

out vec3 objPos;
out float vertScale;

void main() {
  VS_BEGIN
  normal = normalize((mat3(modelMatrix) * vertex_normal));
  vec4 v = vec4(vertex_position, 1.0);
  objPos = v.xyz;
  vec4 wp = modelMatrix * v;
  pos = wp.xyz;
  vertScale = instanceScale;
  gl_Position = mProj * (mView * wp);
  VS_END
}
