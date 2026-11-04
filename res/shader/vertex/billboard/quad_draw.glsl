// Draw-block variant of quad.glsl for scene passes: mWorld and the sprite size
// come from the group-2 DrawBlock (drawUser[1].x = size).
#include vertex_scene
#include math

#define size (drawUser[1].x)

void main() {
  VS_BEGIN
  vec4 wp = mWorld * vec4(vertPos, 1.0);
  vec3 look = normalize(eye - wp.xyz);
  vec3 up = normalize(ortho(look));
  vec3 right = cross(look, up);
  wp.xyz += size * uv.x * right;
  wp.xyz += size * uv.y * up;
  pos = wp.xyz;
  gl_Position = mProj * (mView * wp);
  VS_END
}
