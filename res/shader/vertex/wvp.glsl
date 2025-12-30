/* -- World-View-Projection ----------------------------------------------------
   The standard projection pipeline for game objects in world-space.

   Requires:
     * An active camera to provide view & projection matrices (via CameraUBO)
     * mWorld matrix providing object's local->world transform
     * mWorldIT matrix (inverse-transpose of mWorld)
----------------------------------------------------------------------------- */

#include vertex

out vec3 objPos;

void main() {
  VS_BEGIN
  normal = normalize((mWorldIT * vec4(vertex_normal, 0)).xyz);
  vec4 v = vec4(vertex_position, 1.0);
  objPos = v.xyz;
  vec4 wp = mWorld * v;
  pos = wp.xyz;
  gl_Position = mProj * (mView * wp);
  VS_END
}