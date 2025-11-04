#include fragment
#include deferred
#include math
#include color
#include noise

uniform float rMin;
uniform float rMax;
uniform float ringHeight;
uniform float seed;
uniform float time;
uniform float rotationSpeed; // max speed for inner radius
uniform float twistFactor;   // controls amount of radial twist

#autovar vec3 eye
#autovar vec3 starDir

in vec3 objPos;

void main() {
    // --- Normalized radial coordinate ---
    float r = length(vec2(objPos.x, objPos.z));
    float rNorm = (r - rMin) / (rMax - rMin);

    // --- Differential rotation ---
    float speed = rotationSpeed * (1.0 - rNorm); // inner faster
    float angle = atan(objPos.z, objPos.x) + time * speed;

    // --- Radial twist ---
    angle += twistFactor * rNorm; // twist increases with radius

    float radius = r;
    vec2 rotPos = vec2(cos(angle), sin(angle)) * radius;
    vec2 angleCircle = vec2(cos(angle), sin(angle)) * 0.5 + 0.5;

    // --- Base vertical factor (global fade) ---
    float yNorm = clamp(objPos.y / ringHeight + 0.5, 0.0, 1.0);
    float baseVerticalFade = mix(0.3, 1.0, exp(-4.0 * abs(yNorm - 0.5)));

    // --- Perspective fade ---
    vec3 toEye = eye - objPos;
    float viewDist = length(toEye);
    float fadeDistance = 1e5;
    float perspectiveFade = clamp(1.0 - viewDist / fadeDistance, 0.0, 1.0);

    // --- Radial edge fade ---
    float edgeWidth = 0.1;
    float radialFade = smoothstep(1.0, 1.0 - edgeWidth, rNorm) *
                       smoothstep(0.0, 0.0 + edgeWidth, rNorm);

    // --- Dense streaks with radial + vertical variation ---
    float streak = 0.0;
    int streakLayers = 50;
    for(int i = 0; i < streakLayers; i++){
        // Radial-dependent offsets
        float pos = fSmoothNoise(vec2(float(i), rNorm*60.0 + seed*float(i)), 1, 2.0) + rNorm*2.0;
        float width = mix(0.02 + rNorm*0.05, 0.06 + rNorm*0.08, fSmoothNoise(vec2(float(i), seed*2.0),1,2.0));
        float intensity = mix(0.2, 0.6, fSmoothNoise(vec2(float(i), rNorm*3.0 + seed*3.0),1,2.0));
        
        // Vertical offset per streak layer
        float yOffset = 0.2 * fSmoothNoise(vec2(float(i), seed*10.0), 1, 2.0); 
        float layerYNorm = clamp(yNorm + yOffset - 0.5, 0.0, 1.0);
        float layerFade = exp(-8.0 * abs(layerYNorm - 0.5)); // per-layer vertical softening

        // Radial frequency varies with radius
        float radialFreq = mix(40.0, 20.0, rNorm);
        float layer = smoothstep(0.5 - width, 0.5 + width, fract(rNorm*radialFreq + pos)) * intensity * layerFade;

        streak += layer;
    }
    streak = streak / (streak + 1.0);
    streak = clamp(streak, 0.0, 1.0);

    // --- Streak-aligned dust with radial variation ---
    vec3 dustCoord = vec3(rNorm*20.0 + rNorm*5.0, yNorm*10.0, angleCircle.x*10.0); 
    float dustNoise = fSmoothNoise(dustCoord, 2, 2.0);
    float dust = 0.05 + 0.08 * dustNoise;
    dust *= streak;
    dust *= 0.8 + 0.2 * fSmoothNoise(vec2(rNorm*50.0, seed*7.0), 1, 2.0);

    float ringMask = clamp(streak + dust, 0.0, 1.0);

    // --- Final alpha ---
    float alpha = radialFade * baseVerticalFade * perspectiveFade * ringMask;

    // --- Color gradient ---
    vec3 innerColor = vec3(0.85, 0.8, 0.7);
    vec3 outerColor = vec3(1.0, 0.98, 0.9);
    vec3 color = mix(innerColor, outerColor, rNorm);
    color *= mix(0.95, 1.05, fSmoothNoise(vec2(rNorm*5.0, yNorm*5.0), 2, 2.0));

    // --- Lighting ---
    float lightFactor = clamp(dot(normalize(vec3(0.0, yNorm-0.5, 0.0)), starDir), 0.0, 1.0);
    color *= 0.7 + 0.3 * lightFactor;

    // --- Output ---
    setAlbedo(color);
    setAlpha(alpha);
    setNormal(normalize(normal));
    setDepth();
    setMaterial(Material_NoShade);
}
