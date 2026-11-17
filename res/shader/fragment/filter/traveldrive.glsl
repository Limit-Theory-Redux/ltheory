#include filter
#include math

// Travel drive overlay: a pulsing ring around the ship and star streaks,
// added on top of the picture (RenderCoreSystem:travelDrive).
#group 2
layout(std140) uniform Params {
    float intensity;    // 0-1: overall effect strength (ramps up/down)
    float driveSpeed;   // current travel drive multiplier
    float time;         // seconds
    float aspect;       // screen width / height
    vec2 shipScreenPos; // the ship on screen, in uv (0..1)
};

void main() {
  vec3 scene = texture(src, uv).xyz;

  vec2 center = shipScreenPos;
  vec2 dir = uv - center;

  // In screen heights, so the ring is round on any screen shape
  vec2 dirA = vec2(dir.x * aspect, dir.y);
  float dist = length(dirA);

  // --- Energy bubble / shield effect ---
  // Ring that pulses outward from ship
  float ringRadius = 0.15 + 0.02 * sin(time * 3.0);
  float ring = exp(-pow((dist - ringRadius) * 30.0, 2.0));
  ring *= intensity;

  // Inner glow (stronger when charging)
  float innerGlow = exp(-dist * 8.0) * intensity * 0.3;

  // --- Star streaks (speed lines) when active ---
  float streaks = 0.0;
  if (driveSpeed > 2.0) {
    // Radial lines emanating from center (motion blur effect)
    float angle = atan(dirA.y, dirA.x);
    float speedFactor = min(1.0, (driveSpeed - 2.0) / 50.0);

    // Multiple streak frequencies for variety
    float streak1 = pow(abs(sin(angle * 20.0 + time * 2.0)), 40.0);
    float streak2 = pow(abs(sin(angle * 35.0 - time * 1.5)), 50.0);
    float streak3 = pow(abs(sin(angle * 12.0 + time * 3.0)), 30.0);

    // Streaks radiate outward, stronger at edges
    float radialFade = smoothstep(0.05, 0.3, dist) * smoothstep(0.8, 0.4, dist);
    streaks = (streak1 * 0.5 + streak2 * 0.3 + streak3 * 0.2) * radialFade * speedFactor * intensity;
  }

  // --- Color ---
  // Destiny-inspired: blue-white energy with slight purple tinge
  vec3 chargeColor = vec3(0.3, 0.5, 1.0);   // blue during charge
  vec3 activeColor = vec3(0.6, 0.8, 1.0);   // brighter blue-white when active
  vec3 streakColor = vec3(0.7, 0.85, 1.0);  // near-white streaks

  float speedT = min(1.0, driveSpeed / 50.0);
  vec3 bubbleColor = mix(chargeColor, activeColor, speedT);

  // Slight colour split on the ring
  vec3 ringColorShifted = vec3(
    ring * 1.1,
    ring * 0.95,
    ring * 1.2
  );

  // Edge distortion shimmer
  float shimmer = sin(dist * 60.0 - time * 8.0) * 0.5 + 0.5;
  shimmer = shimmer * shimmer * ring * 0.3;

  vec3 finalColor = bubbleColor * (ringColorShifted + innerGlow + shimmer)
                  + streakColor * streaks;

  // Vignette darkening at edges when at high speed (tunnel vision)
  float tunnelFactor = clamp((driveSpeed - 10.0) / 80.0, 0.0, 1.0);
  float tunnel = smoothstep(0.7, 0.3, dist) * tunnelFactor * 0.3;
  finalColor += vec3(0.02, 0.03, 0.06) * tunnel;

  outColor = vec4(scene + finalColor, 1.0);
}
