/* -- World-View-Projection (Instanced) ----------------------------------------
   Standard projection pipeline for instanced game objects.

   Uses per-instance model matrix instead of uniform mWorld.
   Uses per-instance color for tinting.

   Requires:
     * An active camera to provide view & projection matrices
     * Instance attributes 4-7 for model matrix
     * Instance attribute 8 for color
----------------------------------------------------------------------------- */

#include instanced

out vec3 objPos;

void main() {
  VS_INSTANCED_BEGIN
  normal = normalize(mInstanceIT * vertex_normal);
  vec4 v = vec4(vertex_position, 1.0);
  objPos = v.xyz;
  vec4 wp = mInstance * v;
  pos = wp.xyz;
  gl_Position = mProj * (mView * wp);
  VS_INSTANCED_END
}
