// Draw-block variant of beam.glsl for scene passes: color (xyz) and alpha (w)
// are drawUser[0], size (xy) and seed (z) drawUser[1] of the group-2 DrawBlock.
#include fragment
#include math
#include noise
#include draw_block

#define color (drawUser[0].xyz)
#define alpha (drawUser[0].w)
#define size (drawUser[1].xy)
#define seed (drawUser[1].z)

void main() {
  float radial = abs(uv.x);
  float travel = saturate(uv.y);
  float radialAA = max(fwidth(radial), 0.0015);
  float longitudinal = smoothstep(0.0, 0.025, travel) *
                       (1.0 - smoothstep(0.965, 1.0, travel));
  float boundary = 1.0 - smoothstep(0.72 - radialAA, 1.0 + radialAA, radial);
  float halo = exp(-pow2(3.25 * radial));
  float core = exp(-pow2(10.0 * radial));
  float filament = exp(-pow2(18.0 * (radial - 0.30)));

  float noiseScale = max(4.0, min(48.0, size.y * 2.0));
  float broadNoise = fSmoothNoise(
      vec2(travel * noiseScale + seed * 7.0, seed + 19.0), 3, 1.7);
  float fineNoise = fSmoothNoise(
      vec2(travel * noiseScale * 2.7 - seed * 11.0, seed * 3.0 + 5.0), 2, 2.1);
  float variation = mix(0.82, 1.18, broadNoise);
  variation *= mix(0.92, 1.08, fineNoise);

  vec3 hotCore = mix(color, vec3(1.0, 0.90, 0.84), 0.78);
  vec3 emission = color * (0.42 * halo * boundary + 0.20 * filament)
      + hotCore * (1.55 * core);
  emission *= alpha * longitudinal * variation;
  outColor = vec4(emission, 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
