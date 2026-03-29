#include fragment
#include math

uniform vec2 lightPos;
uniform vec3 lightColor;
uniform float intensity;
uniform vec2 screenSize;
uniform float xStreak;    // 0-1: how much X-shaped streak to add (only at high intensity)

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

    // Diagonal streak 1 (45 degrees)
    float d1y = -dir.x * sin45 + dir.y * cos45;
    float d1x =  dir.x * cos45 + dir.y * sin45;
    float xs1 = exp(-abs(d1y * aspect) * 80.0) * 1.0 / (1.0 + abs(d1x) * 3.0);

    // Diagonal streak 2 (-45 degrees)
    float d2y = dir.x * sin45 + dir.y * cos45;
    float d2x = dir.x * cos45 - dir.y * sin45;
    float xs2 = exp(-abs(d2y * aspect) * 80.0) * 1.0 / (1.0 + abs(d2x) * 3.0);

    xStreakVal = (xs1 + xs2) * xStreak * intensity * 0.2;
  }

  // --- Central glow ---
  float glow = 1.0 / (1.0 + distA * distA * 300.0) * intensity * 0.5;

  // --- Subtle ghost ---
  vec2 ghostDir = vec2(0.5) - center;
  vec2 ghostPos = center + ghostDir * 0.5;
  float ghost = smoothstep(0.03, 0.0, length(screenUV - ghostPos)) * intensity * 0.03;

  // Chromatic separation on streaks
  vec3 streakColor;
  streakColor.r = (hStreak + xStreakVal) * 1.1 + hStreak2;
  streakColor.g = (hStreak + xStreakVal) * 0.95 + hStreak2 * 0.9;
  streakColor.b = (hStreak + xStreakVal * 0.7) * 0.8 + hStreak2 * 0.85;

  vec3 flare = lightColor * (glow + ghost) + lightColor * streakColor;

  // Smooth overall falloff
  flare *= smoothstep(1.5, 0.0, distA);

  outColor = vec4(flare, 1.0);
}
