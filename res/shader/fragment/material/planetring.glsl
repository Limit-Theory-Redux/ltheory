#include fragment
#include deferred
#include math
#include color
#include noise

uniform float rMin;
uniform float rMax;

in vec3 objPos;

void main() {
    // --- Radial distance ---
    float r = length(vec2(objPos.x, objPos.z));
    
    // --- Angle normalized [0,1] ---
    float angle = atan(objPos.z, objPos.x);
    float angleNorm = (angle + PI) / (2.0 * PI);
    
    // --- Ring info ---
    float ringSize = rMax - rMin;
    
    // --- Radial fade ---
    float edgeWidth = ringSize * 0.1;
    float alpha = smoothstep(rMax, rMax - edgeWidth, r) *
                  smoothstep(rMin, rMin + edgeWidth, r);
    alpha = pow(alpha, 2.0);
    
    // --- Radial & angular noise for variation ---
    vec2 noiseCoordRadial = vec2(r / ringSize * 5.0, 0.0);
    vec2 noiseCoordAngular = vec2(angleNorm * 10.0, r / ringSize * 2.0);
    float variation = fSmoothNoise(noiseCoordRadial, 3, 2.0) * 0.5 +
                      fSmoothNoise(noiseCoordAngular, 2, 1.0) * 0.5;
    
    // --- Radial streaks with smooth transitions ---
    float radialFrequency = 80.0;      // number of streaks along radius
    float angularVariation = angleNorm * 2.0 * PI * 1.5; // small angular variation
    float streaksRaw = sin(r / ringSize * radialFrequency * 2.0 * PI + angularVariation + variation * 3.0);
    
    // Normalize to 0-1
    streaksRaw = streaksRaw * 0.5 + 0.5;
    
    // Smooth multiple layers for soft transitions
    float streaks = smoothstep(0.3, 0.7, streaksRaw);         // base smooth
    streaks *= smoothstep(0.0, 1.0, streaksRaw);             // extra blending
    streaks = pow(streaks, 1.5);                              // slightly sharpen
    
    // --- Optional secondary finer streaks ---
    float streaks2Raw = sin(r / ringSize * radialFrequency * 2.0 * PI * 1.5 + angularVariation + variation * 5.0);
    streaks2Raw = streaks2Raw * 0.5 + 0.5;
    float streaks2 = smoothstep(0.35, 0.65, streaks2Raw);
    streaks2 *= smoothstep(0.0, 1.0, streaks2Raw);
    streaks2 = pow(streaks2, 1.2);
    
    // --- Combine streak layers ---
    float combinedStreaks = streaks * 0.7 + streaks2 * 0.3;
    
    // --- Radial bands for extra detail ---
    float bands = sin(r / ringSize * 15.0 + variation * 2.0) * 0.5 + 0.5;
    bands = smoothstep(0.35, 0.65, bands);
    
    // --- Combine structure ---
    float structure = mix(1.0, bands, 0.2) * mix(0.5, 1.0, combinedStreaks);
    
    // --- Color ---
    float radialGradient = smoothstep(rMin, rMax, r);
    vec3 color = mix(vec3(1.0, 1.0, 1.0), vec3(0.95, 0.97, 1.0), radialGradient * 0.3);
    color *= mix(0.88, 1.0, variation);
    color *= mix(0.85, 1.0, structure);
    
    // --- Alpha modulation ---
    alpha *= mix(0.5, 1.0, structure);
    alpha *= 0.5;
    
    // --- Output ---
    setAlbedo(color);
    setAlpha(alpha);
    setNormal(normalize(normal));
    setDepth();
    setMaterial(Material_NoShade);
}
