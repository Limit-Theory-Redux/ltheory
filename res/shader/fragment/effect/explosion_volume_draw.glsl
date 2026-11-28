#include fragment
#include math
#include noise
#include draw_block

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
// Draw-block contract shared with the vertex stage
// (vertex/billboard/explosion_draw.glsl, quad half-extent = the reach with a margin):
// drawUser[0] = (age, seed, life, steps), drawUser[1] = (origin.xyz, size),
// drawUser[2] = (up.xyz, detail); origin is camera-relative. `steps` is the
// march's step count (0 = MAX_STEPS) and `detail` 1 for the full density
// field, 0 for a cheaper one (one turbulence set, no filaments, notches or
// sparks): the quality level and the explosion's size on screen choose them.
// The march covers only the sphere the fire can reach at its age
// (`reachOf`), which the vertex stage sizes the billboard to as well.

#define age (drawUser[0].x)
#define seed (drawUser[0].y)
// Total intended lifetime (seconds). Lets the effect end at EXACT zero -
// full fade-in, burn-out curve, guaranteed zero tail - and makes loop
// restarts seamless. 0 = no lifetime fade (loop mode manages its own age).
#define life (drawUser[0].z)
#define origin (drawUser[1].xyz)
#define size (drawUser[1].w)
#define up (drawUser[2].xyz)
#define steps (drawUser[0].w)
#define detail (drawUser[2].w)

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
float fireDensity(vec3 lp, float R, float t, float ph, float sd, bool hi,
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
  // The cheap field keeps one churn set at about the same mean.
  float turb = hi
             ? 0.45 * churn(fq, ph)
               + 0.35 * churn(sp * 5.9 + 31.7, ph * 1.37 + 11.0)
               + 0.20 * fSmoothNoise(lp * (5.5 + 4.0 * R) + sd * 0.77
                                     - dir * ph * 0.3, 2, 2.05)
             : 0.80 * churn(fq, ph) + 0.10;

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

  // The edge's softness grows with the fire: a young fireball is a small,
  // ragged core, not a soft ball padded to a fixed size.
  float edge = 0.03 + 0.07 * smoothstep(0.10, 0.60, R);
  float wOut = smoothstep(Ro + edge, Ro - edge, r);
  float wIn  = smoothstep(Ri - 0.06, Ri + 0.16, r);
  float d = wOut * wIn;

  // Fire filaments: detail sampled with HIGH frequency along the radius
  // and LOW across it stretches into thin radial tongues - the streaky
  // look of real flame, instead of puffy cloud lobes. High contrast:
  // bright tongues against dark gaps.
  float fil = hi ? fSmoothNoise(vec3(r * (9.0 + 3.0 * R), sp.y * 2.2, sp.z * 2.2)
                                + ph * 0.21, 2, 2.1)
                : 0.5;
  d *= 0.45 + 0.95 * fil;

  // Extra silhouette raggedness: bite small notches out of the outer edge.
  float notch = hi ? fSmoothNoise(sdir * 6.3 + sd * 0.41, 2, 2.0) : 0.5;
  d *= 1.0 - 0.30 * smoothstep(0.55, 0.95, r / max(Ro, 0.3))
             * smoothstep(0.45, 0.75, notch);

  // Debris sparks: high-frequency knots advecting FASTER than the gas
  // (debris outpaces the fireball) - bright streaks early-to-mid life.
  oSpark = 0.0;
  if (hi) {
    float sparkN = fSmoothNoise(sp * 13.0 - sdir * (t * adv * 2.3) + sd, 2, 2.2);
    oSpark = smoothstep(0.74, 0.92, sparkN)
           * wOut * (1.0 - smoothstep(0.45, 1.05, R));
  }

  // Shock front: a thin bright shell just ahead of the fire, ignition-only
  // (capped at 1.30, safely inside the support zero).
  oShock = exp(-pow((r - min(R * 1.22, 1.30)) / 0.055, 2.0))
         * (1.0 - smoothstep(0.12, 0.55, R));

  // The shock shell is faint and torn by the turbulence: at full weight it
  // read as a smooth, opaque ball well beyond the young fire.
  oShock *= smoothstep(0.55, 1.15, turb);
  d = d + oSpark * 0.9 + oShock * 0.12;

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
  // The opaque floor comes in as the fire grows, so ignition isn't a solid
  // ball the size of its outer edge.
  d = max(d, wOut * wIn * 0.30 * smoothstep(0.25, 0.60, R));

  // Ignition flash at the very center, gone within the first instants.
  d += exp(-r * r * 30.0) * (1.0 - smoothstep(0.04, 0.30, R));

  // Burn-out: surviving wisps thin away while still drifting outward.
  d *= 1.0 - smoothstep(0.95, 1.28, R);

  // Finite support STRICTLY inside the march bound (hard guarantee against
  // the circular cutoff artifact).
  d *= 1.0 - smoothstep(1.20, 1.46, r);

  return max(d, 0.0);
}

// How far from the centre (in sizes) the fire reaches at front radius R:
// the lobed outer edge (up to about 1.33 R plus its softness), the shock
// shell (1.22 R, capped at 1.30, plus its width) and the ignition flash.
// Shared with the vertex stage's billboard size.
float reachOf(float R) {
  return min(1.46, 1.3 * R + 0.25);
}

void main() {
  float t = animSpeed * age;
  float sd = seed * 61.7;

  vec3 ro = eye;
  vec3 rd = normalize(pos - eye);

  // Front radius: fast ignition, strong deceleration, capped at 1.15 -
  // comfortably inside the 1.46 support zero.
  float R = 1.15 * (1.0 - exp(-1.5 * t));

  // The sphere the fire can reach at this age (outer lobes, shock shell and
  // flash all lie inside it): early in life it is small, so the march and
  // the pixels it covers are too.
  float BOUND = reachOf(R);
  vec3 oc = ro - origin;
  float b = dot(oc, rd);
  float c = dot(oc, oc) - (size * BOUND) * (size * BOUND);
  float h = b * b - c;
  if (h < 0.0) {
    discard;
  }
  h = sqrt(h);
  float tNear = max(0.0, -b - h);
  float tFar = -b + h;

  float span = tFar - tNear;
  int nSteps = steps > 0.5 ? min(int(steps), MAX_STEPS) : MAX_STEPS;
  float baseStep = span / float(nSteps);
  bool hi = detail > 0.5;
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
    if (i >= nSteps || s >= tFar || trans < 0.05) {
      break;
    }
    vec3 lp = (ro + rd * s - origin) / size;
    float sparkC = 0.0;
    float shockC = 0.0;
    float raw = fireDensity(lp, R, t, ph, sd, hi, sparkC, shockC) * env;

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
      // Empty space: the density is no distance to skip by, so a full step.
      s += baseStep;
    }
  }

  // Filmic ALU tone map.
  vec3 outc = max(accum - 0.004, 0.0);
  outc = (outc * (6.2 * outc + 0.5)) / (outc * (6.2 * outc + 1.7) + 0.06);

  // Density-aware absorption: thick gas self-shadows, pulling the pale
  // body down into saturated deep-orange/red (real fire is never cream).
  float shade = 1.0 - 0.38 * clamp(accum.r * 1.4, 0.0, 1.0);
  outc *= mix(vec3(1.0), vec3(1.06, 0.62, 0.34), 1.0 - shade);

  // Premultiplied: the color is the light the fire adds, the alpha how much
  // of the scene behind it hides (the march stops at 5% transmittance, so
  // that counts as opaque).
  float coverage = clamp((1.0 - trans) / 0.95, 0.0, 1.0);
  outColor = vec4(outc, coverage);
  FRAGMENT_CORRECT_DEPTH;
}
