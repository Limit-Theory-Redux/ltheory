#include vertex
#include math

uniform vec2 size;

void main() {
  VS_BEGIN
  vec4 wp = mWorld * vec4(vertPos, 1.0);
  vec3 look = safeNorm((mWorld * vec4(0, 0, 1, 0)).xyz);
  if (dot(look, look) < 0.5) {
    look = vec3(0.0, 0.0, 1.0);
  }
  vec3 toCam = safeNorm(eye - wp.xyz);

  vec3 fallbackUp = abs(look.y) < 0.9
      ? vec3(0.0, 1.0, 0.0)
      : vec3(1.0, 0.0, 0.0);
  vec3 fallbackRight = safeNorm(cross(fallbackUp, look));
  vec3 viewRightRaw = cross(toCam, look);
  float viewRightLength = length(viewRightRaw);
  vec3 viewRight = safeNorm(viewRightRaw);
  float viewWeight = smoothstep(0.02, 0.18, viewRightLength);
  vec3 right = safeNorm(mix(fallbackRight, viewRight, viewWeight));

  wp.xyz += size.x * uv.x * right;
  wp.xyz += size.y * uv.y * look;
  pos = wp.xyz;
  gl_Position = mProj * (mView * wp);
  VS_END
}
