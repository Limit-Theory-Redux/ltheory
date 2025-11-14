#include fragment
#include deferred
#include math
#include color
#include noise
#include scattering2

#autovar vec3 eye
#autovar vec3 starDir

uniform samplerCube surface;
uniform vec3 origin;
uniform vec3 highlandColor;  // Lighter color for highlands
uniform vec3 mariaColor;     // Darker color for maria
uniform float heightMult;
uniform float craterDepth;
uniform float enableAtmosphere;

float heightFn(float h, int octaves, float roughness) {
  float total = 1.0;
  float tw = 0.0;
  float f = PI;
  float w = 1.0;
  float off = 13.579;
  
  for (int i = 0; i < octaves; ++i) {
    total += w * (0.5 + 0.5 * sin(f * h + off));
    tw += w;
    w *= roughness;
    f *= 2.0;
    off += 1.8923;
  }
  
  total /= tw;
  return total;
}

// Enhanced normal calculation for crater rims and details
vec3 calculateDetailedNormal(samplerCube map, vec3 p, float delta) {
  float h = heightFn(texture(map, p).x, 8, 0.65);
  
  vec3 tangent = normalize(cross(p, vec3(0.0, 1.0, 0.0)));
  if (length(tangent) < 0.1) {
    tangent = normalize(cross(p, vec3(1.0, 0.0, 0.0)));
  }
  vec3 bitangent = normalize(cross(p, tangent));
  
  float hx = heightFn(texture(map, normalize(p + tangent * delta)).x, 8, 0.65);
  float hy = heightFn(texture(map, normalize(p + bitangent * delta)).x, 8, 0.65);
  
  vec3 dx = tangent * (hx - h) / delta;
  vec3 dy = bitangent * (hy - h) / delta;
  
  return normalize(p - heightMult * (dx + dy));
}

// Compute self-shadowing from craters and terrain
float computeShadowing(samplerCube map, vec3 p, vec3 L) {
  const int steps = 8;
  float shadow = 1.0;
  float h = heightFn(texture(map, p).x, 8, 0.65);
  
  for (int i = 1; i <= steps; ++i) {
    float t = float(i) / float(steps) * 0.02;
    vec3 samplePos = normalize(p + L * t);
    float sampleH = heightFn(texture(map, samplePos).x, 8, 0.65);
    
    float heightDiff = (sampleH - h) * heightMult - t * 0.5;
    if (heightDiff > 0.0) {
      shadow = min(shadow, exp(-10.0 * heightDiff));
    }
  }
  
  return shadow;
}

void main() {
  vec3 L = starDir;
  vec3 P = pos - origin;
  vec3 V = normalize(pos - eye);
  
  // Sample the surface map
  vec4 map = texture(surface, vertPos);
  
  // Calculate detailed normal with crater features
  vec3 N = calculateDetailedNormal(surface, vertPos, 0.01);
  N = normalize(mix(normalize(normal), N, 0.8));
  
  float NL = max(0.0, dot(N, L));
  
  // Height-based features
  float h = heightFn(map.x, 8, 0.65);
  float colorMask = map.y;
  float roughness = map.z;
  
  // Color blending between highlands and maria
  vec3 baseColor = mix(highlandColor, mariaColor, colorMask);
  
  // Add subtle color variation
  vec3 colorVar = vec3(
    0.95 + 0.05 * sin(37.0 * h),
    0.97 + 0.03 * sin(41.0 * h),
    0.98 + 0.02 * sin(43.0 * h)
  );
  baseColor *= colorVar;
  
  // Self-shadowing from terrain features
  float shadow = computeShadowing(surface, vertPos, L);
  
  // Enhanced terminator lighting (more dramatic shadows)
  float terminator = mix(
    exp(-3.0 * pow(1.0 - NL, 2.0)), 
    1.0, 
    0.02
  );
  
  // Lunar lighting (no atmosphere to scatter light)
  float lighting = NL * shadow * terminator;
  
  // Apply lighting
  vec3 color = baseColor * (0.05 + 0.95 * lighting);
  
  // Optional subtle rim lighting for depth
  float fresnel = pow(1.0 - abs(dot(V, N)), 3.0);
  color += 0.02 * fresnel * max(0.0, NL);
  
  // Optional atmosphere (for moons with thin atmospheres)
  if (enableAtmosphere == 1.0) {
    vec4 atmo = atmosphereDefault(V, eye - origin);
    color = atmo.xyz + color * (1.0 - atmo.w);
  }
  
  FRAGMENT_CORRECT_DEPTH;
  setAlbedo(color);
  setAlpha(1.0);
  setDepth();
  setNormal(N);
  setRoughness(roughness);
  setMaterial(Material_NoShade);
}