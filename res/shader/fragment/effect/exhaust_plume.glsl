#include fragment
#include math
#include noise

// Engine exhaust plume: volumetric streaming jet on a camera-facing
// billboard (single card - always visible from every angle; the raymarch
// supplies the 3D depth). Profile + color ported from the proven legacy
// effect/thruster shader: peaked cone with soft halo, throat flare,
// time-advected flame variation, and strength-driven jet color:
//   boost 0 -> cool blue, boost 1 -> hot orange (the classic engine
//   exhaust ramp, matching the legacy Thruster's color formula).
//
// Billboard contract (vertex/billboard/exhaust.glsl): origin (nozzle),
// size (nozzle radius), up (thrust direction). Fragment-only: age, seed,
// plumeLen, boost.

uniform float age;
uniform float seed;
uniform vec3 origin;
uniform float size;
uniform vec3 up;
uniform float plumeLen;
uniform float boost;

const int MAX_STEPS = 44;

// Rotating-wave turbulence: successive |sin+cos| bands at rising
// frequency, domain rotated/permuted between octaves. Deterministic.
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

void main() {
  float sd = seed * 61.7;
  vec3 ro = eye;
  vec3 rd = normalize(pos - eye);

  // Bounding sphere around the whole plume (nozzle + length).
  float mid = plumeLen * 0.5;
  float bR = sqrt(mid * mid + size * size) + size * 2.0;
  vec3 plumeCenter = origin + up * mid;
  vec3 oc = ro - plumeCenter;
  float b = dot(oc, rd);
  float c = dot(oc, oc) - bR * bR;
  float h = b * b - c;
  if (h < 0.0) {
    discard;
  }
  h = sqrt(h);
  float tNear = max(0.0, -b - h);
  float tFar = -b + h;
  float span = tFar - tNear;
  float baseStep = span / float(MAX_STEPS);

  float ph = 3.2 * age + sd;

  float jit = fract(sin(dot(gl_FragCoord.xy, vec2(12.9898, 78.233)) + sd)
                    * 43758.5453);
  float s = tNear + baseStep * jit * 2.0;

  vec3 accum = vec3(0.0);
  float trans = 1.0;

  // Temperature-order jet color (blackbody): cold/idle = deep orange
  // ember, cruise = amber, full burn/boost = blue-white. Hotter burns
  // BLUER - the physical order (legacy had it inverted: blue at idle,
  // orange at full burn). Boost scales the whole ramp.
  vec3 coldCol = vec3(0.90, 0.26, 0.02);   // idle ember: orange-red
  vec3 midCol  = vec3(1.00, 0.50, 0.10);   // cruise: amber
  vec3 hotCol  = vec3(0.38, 0.66, 1.00);   // full burn: saturated blue
  vec3 col = mix(coldCol, midCol, smoothstep(0.0, 0.6, boost));
  col = mix(col, hotCol, smoothstep(0.6, 1.0, boost));

  for (int i = 0; i < MAX_STEPS; i++) {
    if (s >= tFar || trans < 0.05) {
      break;
    }
    vec3 wp = ro + rd * s;
    vec3 rel = wp - origin;
    float axial = dot(rel, up);            // distance along thrust axis
    vec3 radial = rel - up * axial;        // offset from the axis

    if (axial < -size || axial > plumeLen) {
      s += baseStep * 2.0;
    } else {
      float an = axial / plumeLen;         // 0 nozzle .. 1 tail
      float rr = length(radial);
      float rn = rr / max(size * (1.0 + 0.9 * an), 1e-4);

      // Legacy effect/thruster cone profile, marched in 3D: peaked cone
      // plus soft halo, quick ramp-in after the throat, deep tail fade.
      float v = clamp(an, 0.0, 1.0);
      float uw = max(0.0, rn - 0.25 * sqrt(v));
      float prof = exp(-12.0 * uw) + 0.25 * exp(-12.0 * sqrt(uw));
      prof *= (1.0 - exp(-16.0 * v)) * exp(-10.0 * v);
      prof *= 1.0 + 4.0 * exp(-12.0 * v);

      // ---- Detail: layered deterministic noise at march-step scale ----
      // 1) Angular+axial flame variation (legacy scroll), time-driven.
      float ang = atan(radial.z, radial.x);
      float variation = fSmoothNoise(
          vec2(ang * 2.2 + sd, 20.0 * v - 10.0 * age * 0.35), 2, 1.6);
      // 2) Macro turbulence: rotating-wave churn, advected downstream.
      vec3 fq = radial * (3.0 / max(size, 1e-3))
              + vec3(0.0, 0.0, axial * (4.0 / max(plumeLen, 1e-3)));
      float ch = churn(fq, ph);
      // 3) Filaments: high axial frequency, low across - streaky tongues.
      float fil = fSmoothNoise(
          vec3(axial * (8.0 / max(plumeLen, 1e-3)),
               rr * (2.5 / max(size, 1e-3)),
               rr * (8.0 / max(size, 1e-3))) + sd * 1.3, 2, 2.0);

      prof *= mix(0.75, 1.45,
                  0.45 * variation * variation + 0.35 * ch + 0.20 * fil);

      // Alpha: soft cap keeps the throat->tail gradient along the axis
      // (a saturated alpha collapses the end-on view into one flat
      // disc). Thrust weighting + hard off-state fade: jets extinguish
      // completely as boost falls to zero.
      float a = clamp(prof * 3.0 * (0.5 + 0.5 * boost), 0.0, 1.0)
              * 0.62 * smoothstep(0.0, 0.12, boost);
      accum += trans * col * a;
      trans *= 1.0 - a;
      s += baseStep;
    }
  }

  // Filmic ALU tone map (luma-scaled to avoid per-channel green shift).
  vec3 outc = max(accum * 1.4 - 0.004, 0.0);
  float l = max(max(outc.r, outc.g), outc.b);
  outc *= (l * (6.2 * l + 0.5)) / max(l * (6.2 * l + 1.7) + 0.06, 1e-4)
        / max(l, 1e-4);

  outColor = vec4(outc, 1.0);
  FRAGMENT_CORRECT_DEPTH;
}
