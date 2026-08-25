#include fragment
#include math
#include noise

// Volumetric explosion: procedural raymarched fireball on a view-aligned
// billboard (paired with vertex/billboard/quadpos.glsl). No textures or
// assets - density is fbm value-noise displaced inside an expanding sphere.
//
// Uniform contract shared with the vertex stage (same program => same
// uniform locations): origin (world-space center), size (world-unit radius),
// up (billboard orientation). Fragment-only: age (seconds since spawn),
// seed (per-instance variation).
//
// Cost model: only pixels whose ray hits the bounding sphere march, with a
// fixed step count, early-out on opacity, and a bounded octaves budget -
// several simultaneous explosions stay cheap.

uniform float age;
uniform float seed;
uniform vec3 origin;
uniform float size;
uniform vec3 up;

const float animSpeed = 0.4;
const int   STEPS     = 36;

// Density field in explosion-local space (lp normalized so the fireball
// surface is roughly the unit sphere). t is the normalized animation time.
float density(vec3 lp, float t, float sd) {
  float r = length(lp);
  // Expanding shock radius: fast initial growth, asymptotic settle.
  float R = 1.0 - exp(-1.7 * t);
  if (r > R + 0.12) {
    return 0.0;
  }
  // Turbulent domain warp, scaled against the current radius so early
  // frames stay smooth and later frames break into plumes.
  vec3 q = lp * (3.0 / max(R, 0.3));
  q += 0.9 * R * vec3(
    fSmoothNoise(q.zy + sd, 2, 2.0),
    fSmoothNoise(q.xz - sd, 2, 2.0),
    fSmoothNoise(q.yx + sd * 1.7, 2, 2.0));
  float n = fSmoothNoise(q * 2.2 + sd, 4, 2.05);
  // Soft shell: dense near the shock front, wispy interior.
  float body = smoothstep(R + 0.1, 0.15 * R + 0.05, r + 0.22 * (n - 0.5));
  return clamp(body * (0.45 + 1.15 * n), 0.0, 1.5);
}

void main() {
  float t = animSpeed * age;
  float sd = seed * 61.7;

  vec3 ro = eye;
  vec3 rd = normalize(pos - eye);

  // Bounding-sphere intersection (world units).
  vec3 oc = ro - origin;
  float b = dot(oc, rd);
  float c = dot(oc, oc) - size * size;
  float h = b * b - c;
  if (h < 0.0) {
    discard;
  }
  h = sqrt(h);
  float tNear = max(0.0, -b - h);
  float tFar = -b + h;
  float stepLen = (tFar - tNear) / float(STEPS);

  // Life envelope: pop-in flash, long ember decay.
  float fade = exp(-1.1 * max(0.0, age));
  float ignite = 1.0 - exp(-14.0 * max(0.0, age));

  vec3 accum = vec3(0.0);
  float trans = 1.0;

  for (int i = 0; i < STEPS; i++) {
    float s = tNear + (float(i) + 0.5) * stepLen;
    vec3 wp = ro + rd * s;
    vec3 lp = (wp - origin) / size;
    float d = density(lp, t, sd) * fade * ignite;
    if (d > 0.003) {
      // Blackbody-style ramp: red-orange shell, white-hot core.
      float rL = length(lp);
      float heat = clamp(d * (1.4 - rL), 0.0, 1.0);
      vec3 col = mix(vec3(1.0, 0.18, 0.02), vec3(1.0, 0.62, 0.18), heat);
      col = mix(col, vec3(1.0, 0.95, 0.75), pow(heat, 3.0));
      float a = d * stepLen * 9.0;
      accum += trans * col * a;
      trans *= 1.0 - clamp(a, 0.0, 0.95);
      if (trans < 0.03) {
        break;
      }
    }
  }

  outColor = vec4(accum, 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
