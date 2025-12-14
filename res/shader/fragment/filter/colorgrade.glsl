#version 330
in vec2 uv;
out vec4 fragColor;

uniform sampler2D src;
uniform int mode;

// Color grading parameters
uniform float temperature;   // -1 to 1 (blue to orange)
uniform float tint;          // -1 to 1 (green to magenta)
uniform float saturation;    // 0 to 2
uniform float contrast;      // 0 to 2
uniform float brightness;    // -1 to 1
uniform float vibrance;      // -1 to 1
uniform vec3 lift;           // shadows RGB offset
uniform vec3 gamma;          // midtones RGB power
uniform vec3 gain;           // highlights RGB multiply
uniform float preExposure;

vec3 ColorGradeNeutral(vec3 c) { return c; }

vec3 ColorGradeCinematic(vec3 c) {
    // Crushed blacks, slightly desaturated, teal-orange look
    c = pow(max(c, 0.0), vec3(1.1));
    c = mix(vec3(dot(c, vec3(0.299, 0.587, 0.114))), c, 0.85);
    c = c * vec3(1.05, 1.0, 0.95) + vec3(0.0, 0.01, 0.03);
    return c;
}

vec3 ColorGradeSpace(vec3 c) {
    // Deep blues, enhanced purples, slight desaturation
    c = c * vec3(0.98, 1.0, 1.05);
    float lum = dot(c, vec3(0.2126, 0.7152, 0.0722));
    c = mix(vec3(lum), c, 0.95);
    c = pow(max(c, 0.0), vec3(1.02, 1.0, 0.98));
    return c;
}

vec3 ColorGradeWarm(vec3 c) {
    // Warm, golden tones
    c = c * vec3(1.15, 1.05, 0.9);
    float lum = dot(c, vec3(0.2126, 0.7152, 0.0722));
    c = mix(vec3(lum), c, 1.2);
    return c;
}

vec3 ColorGradeCool(vec3 c) {
    // Cool, blue tones
    c = c * vec3(0.9, 0.95, 1.15);
    float lum = dot(c, vec3(0.2126, 0.7152, 0.0722));
    c = mix(vec3(lum), c, 1.1);
    return c;
}

vec3 ColorGradeVibrant(vec3 c) {
    // Saturated, punchy colors
    float lum = dot(c, vec3(0.2126, 0.7152, 0.0722));
    c = mix(vec3(lum), c, 1.4);
    c = pow(max(c, 0.0), vec3(0.9));
    return c;
}

vec3 ColorGradeBleach(vec3 c) {
    // Bleach bypass effect
    float lum = dot(c, vec3(0.2126, 0.7152, 0.0722));
    c = mix(c, vec3(lum), -0.3);
    c = pow(max(c, 0.0), vec3(1.2));
    return c;
}

vec3 applyTemperature(vec3 c, float temp) {
    // Simple white balance shift
    return c * mix(vec3(1.0, 0.95, 0.85), vec3(0.85, 0.95, 1.15), (temp + 1.0) * 0.5);
}

vec3 applyTint(vec3 c, float tintVal) {
    return c * mix(vec3(1.0, 0.9, 1.0), vec3(0.9, 1.0, 0.9), (tintVal + 1.0) * 0.5);
}

vec3 applyLiftGammaGain(vec3 c, vec3 liftVal, vec3 gammaVal, vec3 gainVal) {
    // Lift (affects shadows)
    c = c + liftVal;
    c = max(c, 0.0);
    
    // Gamma (affects midtones) - only if gamma is not default
    if (gammaVal != vec3(1.0, 1.0, 1.0)) {
        c = pow(c, 1.0 / max(gammaVal, vec3(0.01))); // Prevent division by zero
    }
    
    // Gain (affects highlights)
    c = c * gainVal;
    
    return c;
}

vec3 applyContrast(vec3 c, float contrastVal) {
    // Apply contrast around midpoint, then clamp
    c = (c - 0.5) * contrastVal + 0.5;
    return max(c, 0.0);
}

void main() {
    vec3 color = texture(src, uv).rgb;
    
    // Apply pre-exposure
    color = color * preExposure;
    
    // Apply preset (branchless with multiplication by mode flags)
    vec3 neutral = color;
    vec3 cinematic = ColorGradeCinematic(color);
    vec3 space = ColorGradeSpace(color);
    vec3 warm = ColorGradeWarm(color);
    vec3 cool = ColorGradeCool(color);
    vec3 vibrant = ColorGradeVibrant(color);
    vec3 bleach = ColorGradeBleach(color);
    
    // Branchless selection (only one will be 1.0, rest are 0.0)
    color = neutral * float(mode == 0) +
            cinematic * float(mode == 1) +
            space * float(mode == 2) +
            warm * float(mode == 3) +
            cool * float(mode == 4) +
            vibrant * float(mode == 5) +
            bleach * float(mode == 6);
    
    // Apply adjustments (simplified - always apply, use 0.0 defaults)
    color = applyTemperature(color, temperature);
    color = applyTint(color, tint);
    color = color + brightness;
    color = max(color, 0.0);
    color = (color - 0.5) * contrast + 0.5;
    color = max(color, 0.0);
    
    float lum = dot(color, vec3(0.2126, 0.7152, 0.0722));
    color = mix(vec3(lum), color, saturation);
    color = max(color, 0.0);
    
    // Vibrance with highlight protection
    float highlightMask = smoothstep(0.8, 1.0, lum);
    float adjustedVibrance = vibrance * (1.0 - highlightMask);
    float maxComp = max(max(color.r, color.g), color.b);
    float minComp = min(min(color.r, color.g), color.b);
    float sat = maxComp - minComp;
    float vibranceMask = 1.0 - sat;
    color = mix(vec3(lum), color, 1.0 + adjustedVibrance * vibranceMask);
    color = max(color, 0.0);
    
    // Lift/Gamma/Gain (simplified)
    color = color + lift;
    color = max(color, 0.0);
    color = pow(color, 1.0 / max(gamma, vec3(0.01)));
    color = color * gain;
    
    fragColor = vec4(max(color, 0.0), 1.0);
}