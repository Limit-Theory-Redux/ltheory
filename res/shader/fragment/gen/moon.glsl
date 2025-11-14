#include fragment
#include math
#include noise
#include texcube

uniform float seed;
uniform float craterDensity;
uniform float craterSharpness;
uniform float mariaAmount;

// Generate crater field using cellular noise
float genCraters(vec3 p, float scale, float sharpness) {
  float crater = 0.0;
  float w = 1.0;
  
  for (int i = 0; i < 4; ++i) {
    vec3 sp = p * scale;
    float cell = fCellNoise(sp, seed + float(i) * 3.7, 1, 1.0);
    
    // Create crater rim profile
    float rim = smoothstep(0.3, 0.5, cell) - smoothstep(0.5, 0.7, cell);
    float depth = 1.0 - smoothstep(0.0, 0.5, cell);
    
    crater += w * (depth * 0.8 + rim * 0.2);
    w *= 0.5;
    scale *= 2.3;
  }
  
  return pow(crater, sharpness);
}

// Generate maria (dark flat regions)
float genMaria(vec3 p) {
  float maria = 0.0;
  float w = 1.0;
  vec3 sp = p;
  
  for (int i = 0; i < 3; ++i) {
    float n = fCellNoise(sp, seed + 10.0 + float(i) * 2.3, 1, 1.0);
    maria += w * smoothstep(0.4, 0.6, n);
    w *= 0.6;
    sp *= 2.1;
  }
  
  return saturate(maria);
}

// Generate fine surface detail
float genDetail(vec3 p) {
  float detail = 0.0;
  float w = 1.0;
  vec3 sp = p;
  
  for (int i = 0; i < 6; ++i) {
    detail += w * (fCellNoise(sp, seed + 20.0 + float(i) * 1.7, 4, 1.3) - 0.5);
    w *= 0.5;
    sp *= 2.5;
  }
  
  return detail;
}

float genHeight(vec3 p) {
  // Base terrain with craters
  float craters = genCraters(p, craterDensity, craterSharpness);
  
  // Large scale features
  float largeScale = 0.5 + 0.5 * sin(8.0 * frCellNoise(p * 0.5, seed + 30.0, 8, 1.2));
  
  // Fine detail
  float detail = genDetail(p * 8.0);
  
  // Combine layers
  float height = largeScale * 0.3 + craters * 0.5 + detail * 0.2;
  
  // Add some warping for realism
  vec3 warp = 0.1 * vec3(
    fCellNoise(p, seed + 40.0, 3, 1.2),
    fCellNoise(p, seed + 43.0, 3, 1.2),
    fCellNoise(p, seed + 46.0, 3, 1.2)
  );
  
  height += 0.05 * fCellNoise(p + warp, seed + 50.0, 5, 1.4);
  
  return saturate(height);
}

float genColor(vec3 p) {
  // Maria regions (darker areas)
  float maria = genMaria(p);
  
  // Color variation
  float colorVar = 0.5 + 0.5 * sin(
    5.0 * frCellNoise(p * 2.0, seed + 60.0, 6, 1.3)
  );
  
  // Blend maria with highlands
  return mix(colorVar, 0.3, maria * mariaAmount);
}

float genRoughness(vec3 p) {
  // Roughness based on terrain features
  float maria = genMaria(p);
  float detail = 0.5 + 0.5 * fCellNoise(p * 10.0, seed + 70.0, 4, 1.2);
  
  // Maria are smoother, highlands are rougher
  return mix(detail, 0.4, maria * 0.6);
}

void main() {
  vec3 p = cubeMapDir(uv);
  
  float height = genHeight(p);
  float color = genColor(p);
  float roughness = genRoughness(p);
  
  outColor = vec4(height, color, roughness, 0.0);
}