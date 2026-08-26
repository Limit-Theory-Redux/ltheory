#include fragment
#include deferred
#include math
#include noise

uniform float intensity;
uniform float driveSpeed;
uniform float time;
uniform float effectScale;

void main() {
  vec3 N = normalize(normal);
  vec3 V = normalize(eye - pos);
  vec3 P = vertPos;

  float t = time;
  float speedFactor = clamp((driveSpeed - 1.0) / 30.0, 0.0, 1.0);

  // Energy flowing over the hull from front to back
  float flow1 = smoothNoise(P * 3.0 + vec3(0, 0, -t * 4.0));
  float flow2 = smoothNoise(P * 6.0 + vec3(t * 0.5, -t * 0.3, -t * 6.0));
  float flow3 = smoothNoise(P * 12.0 + vec3(-t * 0.7, t * 0.4, -t * 8.0));
  float energy = flow1 * 0.4 + flow2 * 0.35 + flow3 * 0.25;

  // Pulsing glow
  float pulse = 0.7 + 0.3 * sin(t * 3.0 + P.z * 5.0);

  // Edge glow (energy shell)
  float fresnel = pow(1.0 - abs(dot(N, V)), 1.5);

  // Overall brightness — visible everywhere but brighter at edges
  float effect = (0.15 + energy * 0.5 + fresnel * 0.5) * pulse * intensity;

  // Color
  vec3 color = mix(vec3(0.2, 0.4, 1.0), vec3(0.6, 0.8, 1.0), speedFactor);

  vec3 finalColor = color * effect * 5.0;

  if (intensity < 0.01) discard;

  setAlbedo(finalColor);
  setAlpha(1.0);
  setDepth();
  setNormal(N);
  setRoughness(0);
  setMaterial(Material_NoShade);
}
