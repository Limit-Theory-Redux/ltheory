/* -- World-View-Projection ----------------------------------------------------
   The standard projection pipeline for game objects in world-space.

   Requires:
     * An active camera to provide view & projection matrices
     * The group-2 DrawBlock (see draw_block.glsl): mWorld, the object's
       local->world transform, and mWorldIT, its inverse-transpose
----------------------------------------------------------------------------- */

#include vertex_scene

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