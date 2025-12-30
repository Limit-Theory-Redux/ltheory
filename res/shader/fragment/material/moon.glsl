#include fragment
#include deferred
#include math
#include color
#include noise
#include scattering2
#include texcube


uniform samplerCube surface;
uniform vec3 origin;
uniform vec3 highlandColor;
uniform vec3 mariaColor;
uniform float heightMult;
uniform float enableAtmosphere;

vec3 calculateDetailedNormal(samplerCube map, vec3 p, float delta) {
    float h = texture(map, p).x;

    vec3 tangent = normalize(cross(p, vec3(0.0, 1.0, 0.0)));
    if (length(tangent) < 0.1) tangent = normalize(cross(p, vec3(1.0, 0.0, 0.0)));
    vec3 bitangent = normalize(cross(p, tangent));

    float hx = texture(map, normalize(p + tangent * delta)).x;
    float hy = texture(map, normalize(p + bitangent * delta)).x;

    vec3 dx = tangent * (hx - h) / delta;
    vec3 dy = bitangent * (hy - h) / delta;

    return normalize(p - heightMult * 1.4 * (dx + dy));
}

float computeShadowing(samplerCube map, vec3 p, vec3 L) {
    const int steps = 16;
    float shadow = 1.0;
    float h = texture(map, p).x;

    for (int i = 1; i <= steps; ++i) {
        float t = float(i) / float(steps) * 0.04;
        vec3 pos = normalize(p + L * t);
        float sampleH = texture(map, pos).x;
        float dh = (sampleH - h) * heightMult - t * 0.3;
        if (dh > 0.0) shadow = min(shadow, 1.0 - 7.0 * dh);
    }
    return max(shadow, 0.0);
}

void main() {
    vec3 L = normalize(starDir);
    vec3 V = normalize(pos - eye);
    vec3 p = vertPos;

    vec4 data = texture(surface, p);
    vec3 detailedN = calculateDetailedNormal(surface, p, 0.009);
    vec3 N = normalize(mix(normalize(normal), detailedN, 0.97));

    float NL = max(0.0, dot(N, L));
    float colorMask = pow(data.y, 0.5);
    float roughness = data.z;

    vec3 albedo = mix(highlandColor, mariaColor, colorMask);

    float shadow = computeShadowing(surface, p, L);

    // NO ambient — dark side is black
    float lighting = NL * shadow;  // Only direct light
    lighting += 0.002;  // Barely visible glow on dark side

    vec3 color = albedo * lighting;

    // Very subtle rim lighting at terminator
    float fresnel = pow(1.0 - max(dot(V, N), 0.0), 5.0);
    color += vec3(0.008) * fresnel * NL;  // Only visible near edge

    // No atmosphere on Moon
    // Remove or comment out atmosphere blending if present

    FRAGMENT_CORRECT_DEPTH;
    setAlbedo(color);
    setAlpha(1.0);
    setDepth();
    setNormal(N);
    setRoughness(roughness);
    setMaterial(Material_NoShade);
}