 /* --- Here Lies Magic <3 -------------------------------------------------- */

#include "fragment.glsl"
#include "color.glsl"
#include "math.glsl"
#include "noise.glsl"
#include "texcube.glsl"

uniform vec3 color;
uniform sampler1D lutR;
uniform sampler1D lutG;
uniform sampler1D lutB;
uniform float roughness;
uniform float seed;

const float kScale      = 0.030;
const float kSamples    = 96.00;
const int kIterations   = 30;

/* Greetz Kali! This is a 4D extension of the classic 2D Kaliset, with added
   randomness injection, sinusoidal warping, and axial rotation. Born out of
   far, far too many hours of experimentation. */
float magic(vec3 p) {
  vec4 z = vec4(vec3(0.53) + p, 0.0);
  float a = 0.0, l = 0.0, tw = 0.0, w = 1.0;
  vec4 c = vec4(0.5, 0.55, 0.45, 0.6);
  for (int i = 0; i < kIterations; ++i) {
    float m = dot(z, z);
    z = abs(z) / m - c;
    z += 0.02 * log(1.e-10 + noise4(float(i) + seed));
    z += 0.25 * sin(z);
    a += w * exp(-2.0 * pow2(l - m));
    tw += w;
    if (i > 3) w *= roughness;
    l = m;
    c = c.yzwx;
  }
  return 0.5 + 0.5 * min(cos(30.0 * a / tw), sin(40.0 * a / tw));
}

float bgDensity(vec3 p) {
  return 0.5 + 0.5 * fSmoothNoise(p * 4 + seed, 8, 2.0);
}

vec4 generate(vec3 dir) {
  vec3 c = vec3(0.0);
  float dense = bgDensity(dir);
  c += vec3(0.2 * dense);
  float opacity = 1.0;
  float w = 1.0 / float(kSamples);

  /* Central Star. */ {
    /* Dots between normalized Vec3fs may still be > 1 due to fp precision! */
    float d = max(0.0, 1.0 - dot(dir, starDir));
    float dd = 0.0;
    dd += 8.0 * exp(-sqrt(4096.0 * d));
    dd += 4.0 * exp(-sqrt(sqrt(1024.0 * d)));
    // c = mix(c, sqrt(dd) * color, sqrt(dd));
    c += dd * color;
  }

  /* Absorption. */ {
    vec3 cEmit = color;
    dir *= kScale;
    for (float i = 0.0; i < kSamples; ++i) {
      vec3 p = dir * i * w;
      float t = magic(p);
      t = exp(-t);
      vec3 wave = vec3(
        texture1D(lutR, t).x,
        texture1D(lutG, t).x,
        texture1D(lutB, t).x);
      wave *= sqrt(wave);
      wave = mix(wave, vec3(1.0), 0.999);

      const float k = 6.0;
      const float q = 1.2;
      vec3 vs = exp(-q * wave * vec3(1.0, 0.8, 0.9) * 10.0 * abs(t - 0.85));
      vs -= 1.25 *         exp(-pow(10.0 * abs(t - 0.90), 0.50));
      vs += 0.50 * cEmit * exp(-pow(12.0 * abs(t - 0.90), 0.25));
      c *= exp(-k * w * vs);
      // c = mix(c, vec3(avg(c)), 1.0 - exp(-w));
      opacity *= exp(-k * w * avg(vs));
    }
  }

  /* Normalization. */ {
    float l = lum(c);
    if (l > 1.0) {
      const float limit = 2.0;
      const float k = 1.0 / (limit - 1.0);
      c /= l;
      l = 1.0 - (1.0 / k) * (exp(-k*(l - 1.0)) - 1.0);
      c *= l;
    }
  }

  return vec4(c, opacity);
}

void main() {
  vec3 dir = cubeMapDir(uv);
  fragColor = generate(dir);
}
