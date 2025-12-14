#version 330
in vec2 uv;
out vec4 fragColor;

uniform sampler2D src;
uniform int mode;
uniform float exposure;
uniform vec2 size;

vec3 Linear(vec3 c) { return c; }

vec3 Reinhard(vec3 c) { return c / (c + vec3(1.0)); }

vec3 ACES(vec3 x) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

vec3 Filmic(vec3 x) {
    x = max(vec3(0.0), x - 0.004);
    return (x * (6.2 * x + 0.5)) / (x * (6.2 * x + 1.7) + 0.06);
}

vec3 Uncharted2(vec3 x) {
    const float A = 0.15, B = 0.50, C = 0.10, D = 0.20, E = 0.02, F = 0.30;
    return ((x * (A * x + C * B) + D * E) / (x * (A * x + B) + D * F)) - E / F;
}

vec3 Lottes(vec3 x) {
    const vec3 a = vec3(1.6);
    const vec3 d = vec3(0.977);
    const vec3 hdrMax = vec3(8.0);
    const vec3 midIn = vec3(0.18);
    const vec3 midOut = vec3(0.267);
    
    const vec3 b = (-pow(midIn, a) + pow(hdrMax, a) * midOut) /
                   ((pow(hdrMax, a * d) - pow(midIn, a * d)) * midOut);
    const vec3 c = (pow(hdrMax, a * d) * pow(midIn, a) - pow(hdrMax, a) * pow(midIn, a * d) * midOut) /
                   ((pow(hdrMax, a * d) - pow(midIn, a * d)) * midOut);
    
    return pow(x, a) / (pow(x, a * d) * b + c);
}

vec3 Uchimura(vec3 x) {
    const float P = 1.0;  // max display brightness
    const float a = 1.0;  // contrast
    const float m = 0.22; // linear section start
    const float l = 0.4;  // linear section length
    const float c = 1.33; // black
    const float b = 0.0;  // pedestal
    
    float l0 = ((P - m) * l) / a;
    float L0 = m - m / a;
    float L1 = m + (1.0 - m) / a;
    float S0 = m + l0;
    float S1 = m + a * l0;
    float C2 = (a * P) / (P - S1);
    float CP = -C2 / P;
    
    vec3 w0 = vec3(1.0 - smoothstep(0.0, m, x));
    vec3 w2 = vec3(step(m + l0, x));
    vec3 w1 = vec3(1.0) - w0 - w2;
    
    vec3 T = vec3(m) * pow(x / m, vec3(c)) + vec3(b);
    vec3 S = vec3(P) - (vec3(P) - vec3(S1)) * exp(CP * (x - vec3(S0)));
    vec3 L = m + a * (x - vec3(m));
    
    return T * w0 + L * w1 + S * w2;
}

vec3 GranTurismo(vec3 x) {
    const float P = 1.0;  // max display brightness
    const float a = 1.0;  // contrast
    const float m = 0.22; // linear section start
    const float l = 0.4;  // linear section length
    const float c = 1.33; // black tightness
    const float b = 0.0;  // pedestal
    
    float l0 = ((P - m) * l) / a;
    float L0 = m - m / a;
    float L1 = m + (1.0 - m) / a;
    float S0 = m + l0;
    float S1 = m + a * l0;
    float C2 = (a * P) / (P - S1);
    float CP = -C2 / P;
    
    vec3 w0 = 1.0 - smoothstep(0.0, m, x);
    vec3 w2 = step(m + l0, x);
    vec3 w1 = 1.0 - w0 - w2;
    
    vec3 T = m * pow(x / m, vec3(c)) + b;
    vec3 S = P - (P - S1) * exp(CP * (x - S0));
    vec3 L = m + a * (x - m);
    
    return T * w0 + L * w1 + S * w2;
}

vec3 NarkowiczACES(vec3 x) {
    const float a = 2.51;
    const float b = 0.03;
    const float c = 2.43;
    const float d = 0.59;
    const float e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

vec3 ReinhardExtended(vec3 v, float max_white) {
    vec3 numerator = v * (1.0 + (v / vec3(max_white * max_white)));
    return numerator / (1.0 + v);
}

vec3 ReinhardLuminance(vec3 color) {
    float lum = dot(color, vec3(0.2126, 0.7152, 0.0722));
    float toneMappedLum = lum / (1.0 + lum);
    return color * (toneMappedLum / lum);
}

vec3 AgX(vec3 val) {
    const mat3 agx_mat = mat3(
        0.842479062253094, 0.0423282422610123, 0.0423756549057051,
        0.0784335999999992, 0.878468636469772, 0.0784336,
        0.0792237451477643, 0.0791661274605434, 0.879142973793104
    );
    
    const mat3 agx_mat_inv = mat3(
        1.19687900512017, -0.0528968517574562, -0.0529716355144438,
        -0.0980208811401368, 1.15190312990417, -0.0980434501171241,
        -0.0990297440797205, -0.0989611768448433, 1.15107367264116
    );
    
    const float min_ev = -12.47393;
    const float max_ev = 4.026069;
    
    // Input transform
    val = agx_mat * val;
    
    // Log2 space encoding
    val = clamp(log2(val), min_ev, max_ev);
    val = (val - min_ev) / (max_ev - min_ev);
    
    // Apply sigmoid
    val = clamp(val, 0.0, 1.0);
    vec3 x2 = val * val;
    vec3 x4 = x2 * x2;
    val = + 15.5     * x4 * x2
          - 40.14    * x4 * val
          + 31.96    * x4
          - 6.868    * x2 * val
          + 0.4298   * x2
          + 0.1191   * val
          - 0.00232;
    
    // Output transform
    return agx_mat_inv * val;
}

vec3 Illustris(vec3 x) {
    // Designed for space rendering: deep blacks, bright highlights, stellar contrast
    
    // Parameters tuned for space scenes
    const float blackPoint = 0.008;      // Crush near-blacks for deep space
    const float whitePoint = 12.0;       // Allow bright overexposure for stars/engines
    const float contrast = 1.4;          // Enhanced contrast for dramatic lighting
    const float shoulder = 0.85;         // Gentle shoulder to preserve star blooms
    const float toe = 0.25;              // Steeper toe for rich blacks
    
    // Lift shadows slightly above zero, then apply black crush
    x = max(vec3(0.0), x - blackPoint);
    
    // Split into luminance and chrominance to preserve color in highlights
    float lum = dot(x, vec3(0.2126, 0.7152, 0.0722));
    vec3 chroma = x / (lum + 0.0001);
    
    // Apply contrast curve in log space for natural feel
    lum = pow(lum, contrast);
    
    // Toe: compress shadows with power curve
    float toeScale = pow(lum / (lum + toe), toe);
    
    // Shoulder: soft rolloff for highlights while allowing overexposure
    float shoulderScale = 1.0 - exp(-lum / whitePoint);
    shoulderScale = pow(shoulderScale, 1.0 / shoulder);
    
    // Blend toe and shoulder based on luminance
    float blend = smoothstep(0.0, 0.5, lum / whitePoint);
    lum = mix(lum * toeScale, shoulderScale * whitePoint, blend);
    
    // Reconstruct color with enhanced saturation in midtones
    vec3 color = chroma * lum;
    
    // Subtle saturation boost in midtones (space nebulae, engine glows)
    float midtones = exp(-pow((lum - 0.3) / 0.4, 2.0));
    float satBoost = 1.0 + 0.15 * midtones;
    color = mix(vec3(lum), color, satBoost);
    
    return color;
}

void main() {
    vec3 color = texture(src, uv).rgb * exposure;

    if (mode == 0)      color = Linear(color);
    else if (mode == 1) color = Reinhard(color);
    else if (mode == 2) color = ACES(color);
    else if (mode == 3) color = Filmic(color);
    else if (mode == 4) {
        color = Uncharted2(color);
        color /= Uncharted2(vec3(11.2));
    }
    else if (mode == 5) color = Lottes(color);
    else if (mode == 6) color = Uchimura(color);
    else if (mode == 7) color = GranTurismo(color);
    else if (mode == 8) color = NarkowiczACES(color);
    else if (mode == 9) color = ReinhardExtended(color, 4.0);
    else if (mode == 10) color = ReinhardLuminance(color);
    else if (mode == 11) color = AgX(color);
    else if (mode == 12) color = Illustris(color);

    // Gamma correction
    color = pow(color, vec3(1.0 / 2.2));

    fragColor = vec4(color, 1.0);
}