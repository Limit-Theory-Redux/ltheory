#include vertex
#include math

// Explosion-specific billboard: identical contract to quadpos.glsl
// (origin/size/up shared by name with the fragment stage) but the quad
// half-extent overshoots `size` so turbulent plumes extending past the
// unit shell are never clipped by the geometry.
uniform vec3 origin;
uniform float size;
uniform vec3 up;

const float QUAD_SCALE = 1.7;

void main() {
  VS_BEGIN
  vec4 wp = vec4(vertPos + origin, 1.0);
  vec3 look = normalize(eye - wp.xyz);
  vec3 right = cross(look, up);
  wp.xyz += size * QUAD_SCALE * uv.x * right;
  wp.xyz += size * QUAD_SCALE * uv.y * up;
  pos = wp.xyz;
  gl_Position = mProj * (mView * wp);
  VS_END
}
