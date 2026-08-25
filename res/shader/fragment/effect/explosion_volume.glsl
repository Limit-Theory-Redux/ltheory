#include fragment
#include math
#include noise

// Volumetric fireball - our own implementation of the classic shadertoy
// "volumetric explosion" style (Duke's explosion / supernova remnant,
// otaviogood's rotating-wave turbulence). Techniques only, no copied code.
//
// Design contract (learned the hard way):
//   * The front radius is CAPPED so density always dies strictly inside
//     the march bound - a circular cutoff means density touched the bound.
//   * Growth starts from a POINT: shell thickness scales with radius, so
//     frame one is a spark, not a pre-sized ball.
//   * Dissipation is FRAGMENTATION: the hollow shell tears into isolated
//     wisps that thin out while still expanding - never a solid ball that
//     uniformly fades.
// All fields are pure trig/value-noise => deterministic across GPUs.
//
// Uniform contract shared with the vertex stage: origin, size, up
// (fragment-only: age, seed). Paired with vertex/billboard/explosion.glsl
// (quad half-extent = size * 1.7).

uniform float age;
uniform float seed;
uniform vec3 origin;
uniform float size;
uniform vec3 up;
// Total intended lifetime (seconds). Lets the effect end at EXACT zero -
// full fade-in, burn-out curve, guaranteed zero tail - and makes loop
// restarts seamless. 0 = no lifetime fade (loop mode manages its own age).
uniform float life;

const float animSpeed = 0.42;
const int   MAX_STEPS = 44;

// Rotating-wave turbulence: successive |sin+cos| bands at rising frequency,
// domain rotated/permuted between octaves. Deterministic (pure trig).
float churn(vec3 p, float ph) {
  float n = 0.0;
  float amp = 0.62;
  float f = 2.1;
  for (int i = 0; i < 4; i++) {
    n += amp * abs(sin(p.y * f + ph) + cos(p.x * f - ph * 0.61));
    p.xz = mat2(-0.7374, 0.6755, -0.6755, -0.7374) * p.xz;
    p.xy = p.yx;
    f *= 1.92;
    amp *= 0.52;
  }
  return n;
}

// Per-instance domain rotation: two seed-derived plane rotations make the
// MACRO lobe structure unique per explosion (not just fine detail).
vec3 seedWarp(vec3 p, float sd) {
  float a1 = fract(sd * 0.113) * 6.2832;
  float a2 = fract(sd * 0.271) * 6.2832;
  p.xy = mat2(cos(a1), -sin(a1), sin(a1), cos(a1)) * p.xy;
  p.yz = mat2(cos(a2), -sin(a2), sin(a2), cos(a2)) * p.yz;
  return p;
}

// Density in normalized local space (>0 inside the fire).
// The domain is FIXED and capped well below the march bound (no geometric
// front can ever trace a sphere against the bound). All motion is radial
// ADVECTION of the turbulence field: features stream outward through the
// fixed volume. Late life, the interior vents while per-direction shell
// thickness varies, so remains are isolated clumps - never a ring/ball.
// Every instance differs macroscopically: rotated domain, jittered lobe
// frequencies, advection speed, growth rate and erosion threshold.
// ... oSpark/oShock: the debris/shock contributions, returned for coloring.
float fireDensity(vec3 lp, float R, float t, float ph, float sd,
                  out float oSpark, out float oShock) {
  float r = length(lp);
  vec3 dir = lp / max(r, 1e-4);
  vec3 sp = seedWarp(lp, sd);
  vec3 sdir = seedWarp(dir, sd);

  // Seed-jittered macro parameters: no two explosions share a shape.
  float lobeFq = 2.0 + 1.1 * fract(sd * 0.173);
  float lobeBq = 2.7 + 1.3 * fract(sd * 0.291);
  float adv    = 1.7 + 0.9 * fract(sd * 0.359);

  // Turbulence sampled in radially-advectioned coordinates: the fire
  // streams outward without any geometric front moving. Two decorrelated
  // churn sets + value noise break up all regularity.
  vec3 fq = sp * 3.3 - sdir * (t * adv);
  float turb = 0.45 * churn(fq, ph)
             + 0.35 * churn(sp * 5.9 + 31.7, ph * 1.37 + 11.0)
             + 0.20 * fSmoothNoise(lp * (5.5 + 4.0 * R) + sd * 0.77
                                   - dir * ph * 0.3, 2, 2.05);

  // Lobe amplitudes damp as the blast matures (keeps everything clear of
  // the support edge even at peak radius).
  float lobeAmp = 0.22 * (1.0 - 0.5 * smoothstep(0.80, 1.15, R));
  float lobeF = churn(sdir * lobeFq + sd, ph * 0.75);
  float lobeB = churn(sdir * lobeBq - sd, ph * 0.62 + 40.0);

  // Outer reach and inner vent, each lobed INDEPENDENTLY per direction:
  // where they meet, the shell has vented through; where they differ,
  // thicker clumps survive longer.
  float Ro = R * (1.0 + lobeAmp * (lobeF - 0.70));
  float Ri = R * (0.52 + lobeAmp * (lobeB - 0.60))
           * smoothstep(0.08, 0.85, R);

  float wOut = smoothstep(Ro + 0.10, Ro - 0.10, r);
  float wIn  = smoothstep(Ri - 0.06, Ri + 0.16, r);
  float d = wOut * wIn;

  // Fire filaments: detail sampled with HIGH frequency along the radius
  // and LOW across it stretches into thin radial tongues - the streaky
  // look of real flame, instead of puffy cloud lobes. High contrast:
  // bright tongues against dark gaps.
  float fil = fSmoothNoise(vec3(r * (9.0 + 3.0 * R), sp.y * 2.2, sp.z * 2.2)
                           + ph * 0.21, 2, 2.1);
  d *= 0.45 + 0.95 * fil;

  // Extra silhouette raggedness: bite small notches out of the outer edge.
  float notch = fSmoothNoise(sdir * 6.3 + sd * 0.41, 2, 2.0);
  d *= 1.0 - 0.30 * smoothstep(0.55, 0.95, r / max(Ro, 0.3))
             * smoothstep(0.45, 0.75, notch);

  // Debris sparks: high-frequency knots advecting FASTER than the gas
  // (debris outpaces the fireball) - bright streaks early-to-mid life.
  float sparkN = fSmoothNoise(sp * 13.0 - sdir * (t * adv * 2.3) + sd, 2, 2.2);
  oSpark = smoothstep(0.74, 0.92, sparkN)
         * wOut * (1.0 - smoothstep(0.45, 1.05, R));

  // Shock front: a thin bright shell just ahead of the fire, ignition-only
  // (capped at 1.30, safely inside the support zero).
  oShock = exp(-pow((r - min(R * 1.22, 1.30)) / 0.055, 2.0))
         * (1.0 - smoothstep(0.12, 0.55, R));

  d = d + oSpark * 0.9 + oShock * 0.8;

  // Erosion strengthens with age (seed-offset threshold): holes open, then
  // only the strongest turbulence pockets keep burning.
  float carve = mix(0.32, 0.76, smoothstep(0.15, 1.0, R))
              + 0.10 * (fract(sd * 0.317) - 0.5);
  d *= smoothstep(carve - 0.18, carve + 0.14, turb + 0.35 * d);

  // Emission network: narrow high-turbulence cells burn brightest - a
  // hot lattice against darker (but still optically thick) gas. The mask
  // comes from the FINE filament field so cells are small and numerous,
  // not broad blobs. The floor keeps the body opaque during the fire
  // phase; only the life fade at the end makes it transparent.
  float cells = smoothstep(0.46, 0.72,
                           0.55 * turb + 0.45 * fil + 0.10 * notch);
  d *= mix(0.52, 1.18, cells);
  d = max(d, wOut * wIn * 0.30);

  // Ignition flash at the very center, gone within the first instants.
  d += exp(-r * r * 30.0) * (1.0 - smoothstep(0.04, 0.30, R));

  // Burn-out: surviving wisps thin away while still drifting outward.
  d *= 1.0 - smoothstep(0.95, 1.28, R);

  // Finite support STRICTLY inside the march bound (hard guarantee against
  // the circular cutoff artifact).
  d *= 1.0 - smoothstep(1.20, 1.46, r);

  return max(d, 0.0);
}

void main() {
  float t = animSpeed * age;
  float sd = seed * 61.7;

  vec3 ro = eye;
  vec3 rd = normalize(pos - eye);

  // Oversized bounding sphere matching the billboard overshoot.
  const float BOUND = 1.65;
  vec3 oc = ro - origin;
  float b = dot(oc, rd);
  float c = dot(oc, oc) - size * BOUND * BOUND;
  float h = b * b - c;
  if (h < 0.0) {
    discard;
  }
  h = sqrt(h);
  float tNear = max(0.0, -b - h);
  float tFar = -b + h;

  float span = tFar - tNear;
  float baseStep = span / float(MAX_STEPS);

  // Front radius: fast ignition, strong deceleration, capped at 1.15 -
  // comfortably inside the 1.46 support zero and the 1.65 march bound.
  float R = 1.15 * (1.0 - exp(-1.5 * t));
  float ph = 1.55 * t + sd;

  // Lifetime envelope: smooth fade-in from nothing, guaranteed fade to
  // EXACT zero over the final third of life (and stays zero - no pop-out).
  // life <= 0 disables the lifetime fade (script did not opt in).
  float env = 1.0;
  if (life > 0.0) {
    float u = clamp(age / life, 0.0, 1.0);
    env = smoothstep(0.0, 0.06, u) * (1.0 - smoothstep(0.68, 0.995, u));
  }

  // Jittered entry kills slice banding.
  float jit = fract(sin(dot(gl_FragCoord.xy, vec2(12.9898, 78.233)) + sd)
                    * 43758.5453);
  float s = tNear + baseStep * jit * 2.0;

  vec3 accum = vec3(0.0);
  float trans = 1.0;

  for (int i = 0; i < MAX_STEPS; i++) {
    if (s >= tFar || trans < 0.05) {
      break;
    }
    vec3 lp = (ro + rd * s - origin) / size;
    float sparkC = 0.0;
    float shockC = 0.0;
    float raw = fireDensity(lp, R, t, ph, sd, sparkC, shockC) * env;

    if (raw > 0.003) {
      float dens = raw * exp(-1.05 * max(0.0, age));
      float rL = length(lp);

      // Blackbody-style emission: Planckian locus hue for the given heat
      // (deep red -> orange -> yellow-white), NOT a flat gradient. The
      // body tops out in yellow-orange; white is flash-only.
      float heat = clamp((1.05 - 0.78 * rL) * (0.35 + dens * 1.15)
                         * (1.25 - 0.35 * smoothstep(0.5, 1.15, R)),
                         0.0, 0.82);
      vec3 col = mix(vec3(0.20, 0.01, 0.0), vec3(1.0, 0.22, 0.01),
                     smoothstep(0.0, 0.45, heat));
      col = mix(col, vec3(1.0, 0.55, 0.08), smoothstep(0.45, 0.82, heat));
      col = mix(col, vec3(1.0, 0.86, 0.55), smoothstep(0.82, 1.0,
                         clamp(heat * 1.25, 0.0, 1.0)) * 0.6);

      // Sparks are incandescent solids: hotter than the gas, whiter.
      col += vec3(1.0, 0.80, 0.45) * sparkC * 0.6;

      // Shock front is ionized, blue-white hot.
      col += vec3(0.85, 0.90, 1.0) * shockC * 0.5;

      // Global cooling: the whole body shifts red as the blast ages.
      col *= mix(vec3(1.0), vec3(1.0, 0.52, 0.30),
                 smoothstep(0.55, 1.15, R));

      // Subtle high-frequency flicker on emission (real fire shimmers).
      float fl = 1.0 + 0.07 * sin(29.0 * age + sd * 1.71)
                      * sin(16.3 * age + sd * 0.53);

      // Inner-light bleed: hot core glow, tinted yellow-white but capped
      // so only the innermost gas reads "white hot".
      col += vec3(1.0, 0.88, 0.68) * exp(-rL * rL * 7.0) * 0.35;

      // HDR flash headroom: the ignition burst exceeds 1.0 so the engine's
      // bloom catches it like a real overexposed camera; the body stays
      // below, keeping saturation through the filmic curve.
      float flashBoost = 1.0 + 2.6 * exp(-age * 9.0);
      col *= 0.92 * fl * flashBoost;

      float a = clamp(dens * 18.0, 0.0, 1.0) * 0.82;
      accum += trans * col * a;
      trans *= 1.0 - a;
      s += baseStep;
    } else {
      // Distance-accelerated stepping through empty space.
      s += clamp(-raw * 0.5 * size, baseStep * 0.6, span * 0.22);
    }
  }

  // Filmic ALU tone map.
  vec3 outc = max(accum - 0.004, 0.0);
  outc = (outc * (6.2 * outc + 0.5)) / (outc * (6.2 * outc + 1.7) + 0.06);

  // Density-aware absorption: thick gas self-shadows, pulling the pale
  // body down into saturated deep-orange/red (real fire is never cream).
  float shade = 1.0 - 0.38 * clamp(accum.r * 1.4, 0.0, 1.0);
  outc *= mix(vec3(1.0), vec3(1.06, 0.62, 0.34), 1.0 - shade);

  outColor = vec4(outc, 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
