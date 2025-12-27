#include fragment
#include gamma

in vec3 worldOrigin;
in vec3 worldDir;

uniform sampler2D texAlbedo;
uniform sampler2D texDepth;
uniform sampler2D texLighting;

// Ambient light for base illumination when no environment maps
const float ambientLightingScale = 0.15;

void main() {
  vec3 albedo = texture(texAlbedo, uv).xyz;
  vec3 light = texture(texLighting, uv).xyz;
  float depth = texture(texDepth, uv).x;

  // DEBUG MODE: Uncomment one of these lines to diagnose the issue
  // outColor = vec4(albedo, 1.0); return;           // Shows albedo buffer
  // outColor = vec4(light, 1.0); return;            // Shows lighting buffer
  // outColor = vec4(depth * 0.01, depth * 0.01, depth * 0.01, 1.0); return; // Shows depth
  // outColor = vec4(depth > 0.001 ? 1.0 : 0.0, 0.0, 0.0, 1.0); return; // Red where depth exists

  // Combine deferred lighting with albedo
  vec3 c = albedo * light;

  // Add ambient lighting for areas not hit by point lights
  vec3 ambientColor = vec3(0.08, 0.08, 0.10) * ambientLightingScale;
  vec3 ambientLighting = ambientColor * albedo;
  c += ambientLighting;

  // Ensure minimum visibility if albedo exists but lighting is zero
  if (length(albedo) > 0.1 && length(light) < 0.01) {
    c = albedo * 0.3; // Show geometry even without lighting
  }

  outColor = vec4(c, 1.0);
}
