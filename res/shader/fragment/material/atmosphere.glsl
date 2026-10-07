#include fragment
#include math
#include color
#include noise
#include scattering2
#include draw_block

#define origin (mWorld[3].xyz)    // camera-relative position of the planet
#define rPlanet (drawScale.x)     // the body's scale

#group 1
layout(std140) uniform MaterialParams {
  float rAtmo;
};

void main() {
  vec3 L = starDir;
  vec3 N = normalize(normal);
  vec3 V = normalize(pos - eye);
  vec4 atmo = atmosphereDefault(V, eye - origin, rPlanet, rAtmo);
  float depth = length(pos - eye);
  float a = exp(-max(0.0, depth / (1.0e6) - 1.0) / 0.01);
  a = 1.0;
  vec4 c = a * vec4(atmo.xyz, atmo.w);
  outColor = c;
  FRAGMENT_CORRECT_DEPTH;
}
