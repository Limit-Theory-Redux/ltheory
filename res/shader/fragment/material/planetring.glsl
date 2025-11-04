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

    // --- Angle normalized 0-1 ---
    float angle = atan(objPos.z, objPos.x);
    float angleNorm = (angle + PI) / (2.0 * PI);

    // --- Ring parameters ---
    float ringSize = rMax - rMin;

    // --- Radial edge fade ---
    float edgeWidth = ringSize * 0.1;
    float radialFade = smoothstep(rMax, rMax - edgeWidth, r) *
                       smoothstep(rMin, rMin + edgeWidth, r);

    // --- Periodic angular coordinates for seamless noise ---
    vec2 periodicCoord = vec2(cos(angleNorm * 2.0 * PI), sin(angleNorm * 2.0 * PI)) * 5.0;

    // --- Radial & angular noise for variation ---
    float radialNoise = fSmoothNoise(vec2(r / ringSize * 5.0, 0.0), 3, 2.0);
    float angularNoise = fSmoothNoise(periodicCoord, 2, 1.0);
    float variation = radialNoise * 0.5 + angularNoise * 0.5;

    // --- Radial streaks (positions linear, intensity modulated) ---
    float streakPos = fract(r / ringSize * 40.0);
    float streakBase = smoothstep(0.2, 0.8, streakPos);
    streakBase *= mix(0.8, 1.0, variation);
    streakBase = pow(streakBase, 1.8);

    float streakPos2 = fract(r / ringSize * 80.0);
    float streakBase2 = smoothstep(0.3, 0.7, streakPos2);
    streakBase2 *= mix(0.7, 1.0, variation);
    streakBase2 = pow(streakBase2, 1.5);

    float combinedStreaks = streakBase * 0.7 + streakBase2 * 0.3;

    // --- Radial bands for extra texture ---
    float bands = sin(r / ringSize * 15.0 + variation * 2.0) * 0.5 + 0.5;
    bands = smoothstep(0.35, 0.65, bands);

    // --- Combine structure ---
    float structure = mix(1.0, bands, 0.2) * mix(0.5, 1.0, combinedStreaks);

    // --- Radial tapering for realistic fading toward edges ---
    float radialTaper = smoothstep(0.0, 1.0, (r - rMin) / ringSize) * smoothstep(1.0, 0.0, (r - rMin) / ringSize);

    // --- Realistic coloring with subtle angular shifts ---
    vec3 innerColor = vec3(0.85, 0.8, 0.7);    // brownish/gray inner
    vec3 outerColor = vec3(1.0, 0.98, 0.9);    // icy/yellowish outer
    float radialGradient = smoothstep(rMin, rMax, r);

    // Base gradient
    vec3 color = mix(innerColor, outerColor, radialGradient);

    // Add subtle variation along radius and angle
    color *= mix(0.9, 1.0, variation);
    color *= mix(0.85, 1.0, structure);
    color *= radialTaper;  // fade streaks toward edges

    // Add tiny angular hue shift
    float hueShift = (fSmoothNoise(periodicCoord * 0.5, 2, 1.0) - 0.5) * 0.05;
    color.r += hueShift * 0.5;
    color.g -= hueShift * 0.3;

    // --- Alpha modulation ---
    float alpha = radialFade * mix(0.5, 1.0, structure);
    alpha *= 0.9;

    // --- Output ---
    setAlbedo(color);
    setAlpha(alpha);
    setNormal(normalize(normal));
    setDepth();
    setMaterial(Material_NoShade);
}
