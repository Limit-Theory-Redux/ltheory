#include fragment
#include deferred
#include gamma
#include color
#include math

uniform float brightnessScale;

void main() {
  float r = length(uv);
  float a = 0.0;
  // float fog = getFoginess(farPlane);
  float fog = 0.0;
  vec4 bg = linear(textureCube(envMap, vertPos));
  vec3 c = vertNormal;
  c *= mix(vec3(1.0), bg.xyz, 0.5);
  a += 0.5 * exp(-9.0 * sqrt(r));
  a += 1.0 * exp(-pow2(30.0 * r));
  c *= a*a;
  c *= brightnessScale;

  setAlbedo(c.xyz);
  setAlpha(1.0);
}
