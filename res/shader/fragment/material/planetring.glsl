#include fragment
#include deferred
#include math
#include color
#include noise
#include draw_block

#define time (drawUser[0].x)      // ring rotation time (per draw)

#group 1
layout(std140) uniform MaterialParams {
  float rMin;
  float rMax;
  float ringHeight;
  float seed;
  float rotationSpeed;
  float twistFactor;
  int enableDebug;
  int debugMode;
};

in vec3 objPos;

// Per-ring random numbers come from an integer hash of the seed's bits, not
// from `noise()`. `noise()` is `fract(sin(x) * 4137.3)`: a change in the last
// bit of `sin(x)` moves the result anywhere in 0..1. An expression that only
// reads uniforms (the seed) can be folded by the driver, and it was not
// evaluated the same way on every draw: with identical uniforms, vertices,
// pipeline and destination, the gaps jumped between two layouts on random
// frames (BenchmarkPhases on wgpu, Vulkan and DX12; RMSE 11 to 17). Integer
// arithmetic is exact, so these values are the same however they are
// evaluated; only expressions that vary per fragment still use `noise()`.
uint ringHash(uint x) {
    x ^= x >> 16u;
    x *= 0x7feb352du;
    x ^= x >> 15u;
    x *= 0x846ca68bu;
    x ^= x >> 16u;
    return x;
}

// A number in [0, 1) for (index, channel) of this ring, exact in float.
float ringRandom(int index, int channel) {
    uint h = ringHash(floatBitsToUint(seed) ^ ringHash(uint(index * 8 + channel) + 0x9e3779b9u));
    return float(h >> 8u) * (1.0 / 16777216.0);
}

void main() {
    // --- Normalized radial coordinate ---
    float r = length(vec2(objPos.x, objPos.z));
    float rNorm = (r - rMin) / (rMax - rMin);

    // --- Differential rotation ---
    float speed = rotationSpeed * (1.0 - rNorm);
    float angle = atan(objPos.z, objPos.x) + time * speed;
    angle += twistFactor * rNorm;

    float radius = r;
    vec2 rotPos = vec2(cos(angle), sin(angle)) * radius;
    vec2 angleCircle = vec2(cos(angle), sin(angle)) * 0.5 + 0.5;

    // --- Base vertical factor ---
    float yNorm = clamp(objPos.y / ringHeight + 0.5, 0.0, 1.0);
    float baseVerticalFade = mix(0.3, 1.0, exp(-4.0 * abs(yNorm - 0.5)));

    // --- Perspective fade ---
    vec3 toEye = eye - objPos;
    float viewDist = length(toEye);
    float fadeDistance = 1e5;
    float perspectiveFade = clamp(1.0 - viewDist / fadeDistance, 0.0, 1.0);

    // --- Radial edge fade ---
    float edgeWidth = 0.08;
    float radialFade = smoothstep(1.0, 1.0 - edgeWidth, rNorm) *
                       smoothstep(0.0, 0.0 + edgeWidth, rNorm);

    // --- Ring gaps: Cassini-like division pattern ---
    float gapPattern = 1.0;
    
    // Major gap (Cassini division analog)
    float majorGapPos = 0.72;
    float majorGapWidth = 0.06;
    float majorGap = smoothstep(majorGapPos - majorGapWidth*0.5, majorGapPos - majorGapWidth*0.3, rNorm) *
                     (1.0 - smoothstep(majorGapPos + majorGapWidth*0.3, majorGapPos + majorGapWidth*0.5, rNorm));
    gapPattern *= 1.0 - majorGap * 0.92;
    
    // Minor gaps with procedural variation
    for(int i = 0; i < 5; i++) {
        float gapPos = ringRandom(i, 0) * 0.7 + 0.15;
        float gapWidth = 0.015 + 0.025 * ringRandom(i, 1);
        float gapDepth = 0.4 + 0.5 * ringRandom(i, 2);
        
        float gap = smoothstep(gapPos - gapWidth, gapPos - gapWidth*0.5, rNorm) *
                    (1.0 - smoothstep(gapPos + gapWidth*0.5, gapPos + gapWidth, rNorm));
        gapPattern *= 1.0 - gap * gapDepth;
    }

    // --- Dense streaks with improved density variation ---
    float streak = 0.0;
    int streakLayers = 50;
    for(int i = 0; i < streakLayers; i++){
        float pos = fSmoothNoise(vec2(float(i), rNorm*60.0 + seed*float(i)), 1, 2.0) + rNorm*2.0;
        float width = mix(0.015 + rNorm*0.04, 0.05 + rNorm*0.07, ringRandom(8 + i, 0));
        float intensity = mix(0.15, 0.65, fSmoothNoise(vec2(float(i), rNorm*3.0 + seed*3.0),1,2.0));
        
        // Variable vertical offset
        float yOffset = 0.25 * ringRandom(8 + i, 1);
        float layerYNorm = clamp(yNorm + yOffset - 0.5, 0.0, 1.0);
        float layerFade = exp(-7.0 * abs(layerYNorm - 0.5));

        // Better radial frequency scaling
        float radialFreq = mix(50.0, 18.0, pow(rNorm, 0.7));
        float layer = smoothstep(0.5 - width, 0.5 + width, fract(rNorm*radialFreq + pos)) * intensity * layerFade;

        streak += layer;
    }
    streak = streak / (streak + 1.2); // Adjusted tonemap for more dynamic range
    streak = clamp(streak, 0.0, 1.0);

    // --- Enhanced dust with density modulation ---
    vec3 dustCoord = vec3(rNorm*25.0, yNorm*12.0, angleCircle.x*12.0); 
    float dustNoise = fSmoothNoise(dustCoord, 3, 2.1);
    float dust = 0.04 + 0.1 * dustNoise;
    dust *= streak * 0.9;
    
    // Radial density variation
    // The per-ring offset is a whole number (exact); the noise varies per fragment.
    float densityVar = 0.6 + 0.4 * fSmoothNoise(rNorm*30.0 + floor(ringRandom(0, 7) * 4096.0), 2, 2.0);
    dust *= densityVar;

    // Apply gap pattern
    float ringMask = clamp((streak + dust) * gapPattern, 0.0, 1.0);

    // --- Final alpha with better edge behavior ---
    float alpha = radialFade * baseVerticalFade * perspectiveFade * ringMask;
    alpha = pow(alpha, 0.9); // Slight gamma adjustment for better visibility

    // --- Improved color gradient ---
    vec3 innerColor = vec3(0.82, 0.76, 0.68);
    vec3 midColor = vec3(0.92, 0.88, 0.82);
    vec3 outerColor = vec3(1.0, 0.97, 0.88);
    
    // Three-way color blend
    float colorMix1 = smoothstep(0.0, 0.45, rNorm);
    float colorMix2 = smoothstep(0.45, 1.0, rNorm);
    vec3 color = mix(innerColor, midColor, colorMix1);
    color = mix(color, outerColor, colorMix2);
    
    // Color variation
    color *= mix(0.93, 1.07, fSmoothNoise(vec2(rNorm*8.0, yNorm*6.0), 2, 2.0));
    
    // Subtle color shifts in gaps
    color *= mix(1.0, 0.85, (1.0 - gapPattern) * 0.3);

    // --- Enhanced lighting ---
    float lightFactor = clamp(dot(normalize(vec3(0.0, yNorm-0.5, 0.0)), starDir), 0.0, 1.0);
    float ambientOcclusion = 0.75 + 0.25 * gapPattern; // Darker in gaps
    color *= (0.65 + 0.35 * lightFactor) * ambientOcclusion;

    // --- Output ---
    setAlbedo(color);
    setAlpha(alpha);
    setNormal(normalize(normal));
    setDepth();
    setMaterial(Material_NoShade);
}