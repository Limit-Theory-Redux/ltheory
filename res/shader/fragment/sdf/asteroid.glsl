#include fragment
#include math
#include noise

// `TexGen.Volume` writes the slice's origin, du and dv into the first three members.
#group 2
layout(std140) uniform Params {
    vec4 genOrigin;
    vec4 genDu;
    vec4 genDv;
    int octaves;
    float seed;
    float smoothness;
};
#define origin genOrigin.xyz
#define du genDu.xyz
#define dv genDv.xyz

void main() {
  vec3 p = origin + du * uv.x + dv * uv.y;
  float n = fCellNoise(2.0 * p, seed, octaves, smoothness);
  float d = length(p) - mix(0.05, 1.0, n);
  outColor.x = d;
}
