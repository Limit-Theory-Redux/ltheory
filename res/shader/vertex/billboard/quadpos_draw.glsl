// Draw-block variant of the explosion billboard for scene passes: the world
// position of the center is drawUser[1].xyz and the size drawUser[1].w, the
// camera's up vector drawUser[2].xyz.
#include vertex_scene
#include math

#define origin (drawUser[1].xyz)
#define size (drawUser[1].w)
#define up (drawUser[2].xyz)

void main() {
  VS_BEGIN
  vec4 wp = vec4(vertPos + origin, 1.0);
  vec3 look = normalize(eye - wp.xyz);
  vec3 right = cross(look, up);
  wp.xyz += size * uv.x * right;
  wp.xyz += size * uv.y * up;
  pos = wp.xyz;
  gl_Position = mProj * (mView * wp);
  VS_END
}
