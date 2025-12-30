#include vertex
#include math

uniform vec2 size;

void main() {
  VS_BEGIN
  vec4 wp = mWorld * vec4(vertPos, 1.0);
  vec3 toCam = normalize(eye - wp.xyz);
  vec3 look = normalize((mWorld * vec4(0, 0, 1, 0)).xyz);
  vec3 right = normalize(cross(toCam, look));
  wp.xyz += size.x * uv.x * right;
  wp.xyz += size.y * uv.y * look;
  pos = wp.xyz;
  gl_Position = mProj * (mView * wp);
  VS_END
}
