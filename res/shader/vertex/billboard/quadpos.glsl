#include vertex
#include math

uniform vec3 origin;
uniform float size;
uniform vec3 up;

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
