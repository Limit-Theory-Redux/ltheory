#include fragment
#include math

// Screen-space lens flare: anamorphic streaks, central glow, ghost chain
// along the screen-center axis, and an iris halo - the artifacts a real
// camera lens produces for a bright in-frame source.
//
// uniform contract: lightPos (UV, y-down), lightColor (linear), intensity
// (0..~2), screenSize (pixels), xStreak (0-1 diagonal streaks), ghosts
// (0-1 ghost-chain strength), halo (0-1 ring strength).

uniform vec2 lightPos;
uniform vec3 lightColor;
uniform float intensity;
uniform vec2 screenSize;
uniform float xStreak;
uniform float ghosts;
uniform float halo;

// One ghost disc: soft-edged circle at position p with radius r.
float ghostDisc(vec2 uv, vec2 p, float r) {
  return smoothstep(r, r * 0.55, length(uv - p));
}

void main() {
  vec2 screenUV = vec2(gl_FragCoord.x / screenSize.x, 1.0 - gl_FragCoord.y / screenSize.y);
  vec2 center = lightPos;
  vec2 dir = screenUV - center;

  float aspect = screenSize.x / screenSize.y;
  vec2 dirA = vec2(dir.x, dir.y * aspect);
  float distA = length(dirA);

  // --- Anamorphic horizontal streak ---
  float hStreak = exp(-abs(dir.y * aspect) * 120.0)
              * 1.0 / (1.0 + abs(dir.x) * 2.5)
              * intensity * 0.35;

  // Secondary thinner streak
  float rotY = dir.x * 0.05 + dir.y * 0.9987;
  float rotX = dir.x * 0.9987 - dir.y * 0.05;
  float hStreak2 = exp(-abs(rotY * aspect) * 200.0)
               * 1.0 / (1.0 + abs(rotX) * 4.0)
               * intensity * 0.12;

  // --- X-shaped diagonal streaks (only at close range / high intensity) ---
  float xStreakVal = 0.0;
  if (xStreak > 0.0) {
    float cos45 = 0.7071;
    float sin45 = 0.7071;

    float d1y = -dir.x * sin45 + dir.y * cos45;
    float d1x =  dir.x * cos45 + dir.y * sin45;
    float xs1 = exp(-abs(d1y * aspect) * 80.0) * 1.0 / (1.0 + abs(d1x) * 3.0);

    float d2y = dir.x * sin45 + dir.y * cos45;
    float d2x = dir.x * cos45 - dir.y * sin45;
    float xs2 = exp(-abs(d2y * aspect) * 80.0) * 1.0 / (1.0 + abs(d2x) * 3.0);

    xStreakVal = (xs1 + xs2) * xStreak * intensity * 0.2;
  }

  // --- Central glow ---
  float glow = 1.0 / (1.0 + distA * distA * 300.0) * intensity * 0.5;

  // --- Ghost chain: reflections bouncing between lens elements appear
  // mirrored through the screen center at fixed fractions of the
  // light-to-center vector. Varied radii and subtle per-ghost tint. ---
  vec3 ghostAcc = vec3(0.0);
  if (ghosts > 0.0) {
    vec2 toCenter = vec2(0.5) - center;
    // fraction, radius (in screen heights), tint multiplier
    vec2 g0 = center + toCenter * 0.35;  float r0 = 0.030;
    vec2 g1 = center + toCenter * 0.65;  float r1 = 0.018;
    vec2 g2 = center + toCenter * 0.90;  float r2 = 0.050;
    vec2 g3 = center + toCenter * 1.25;  float r3 = 0.024;
    vec2 g4 = center + toCenter * 1.60;  float r4 = 0.070;
    ghostAcc += vec3(1.00, 0.85, 0.60) * ghostDisc(screenUV, g0, r0) * 0.16;
    ghostAcc += vec3(0.70, 0.90, 1.00) * ghostDisc(screenUV, g1, r1) * 0.12;
    ghostAcc += vec3(1.00, 0.60, 0.35) * ghostDisc(screenUV, g2, r2) * 0.08;
    ghostAcc += vec3(0.60, 1.00, 0.75) * ghostDisc(screenUV, g3, r3) * 0.10;
    ghostAcc += vec3(0.95, 0.80, 1.00) * ghostDisc(screenUV, g4, r4) * 0.06;
    ghostAcc *= ghosts * intensity;
  }

  // --- Iris halo: bright ring centered midway through the flip ---
  vec3 haloAcc = vec3(0.0);
  if (halo > 0.0) {
    vec2 haloCenter = mix(center, vec2(0.5), 0.5);
    vec2 hd = screenUV - haloCenter;
    hd.y *= aspect;
    float haloDist = length(hd);
    float ring = exp(-pow((haloDist - 0.42) / 0.05, 2.0));
    haloAcc = vec3(1.0, 0.85, 0.65) * ring * 0.10 * halo * intensity;
  }

  // Chromatic separation on streaks
  vec3 streakColor;
  streakColor.r = (hStreak + xStreakVal) * 1.1 + hStreak2;
  streakColor.g = (hStreak + xStreakVal) * 0.95 + hStreak2 * 0.9;
  streakColor.b = (hStreak + xStreakVal * 0.7) * 0.8 + hStreak2 * 0.85;

  vec3 flare = lightColor * (glow + streakColor)
             + ghostAcc + haloAcc;

  // Smooth overall falloff
  flare *= smoothstep(1.5, 0.0, distA);

  outColor = vec4(flare, 1.0);
}
