#include fragment
#include deferred
#include math

uniform sampler2D sunTex;
uniform vec3 origin;
uniform vec3 starTint;
uniform float starTemp;
uniform float time;
uniform float scale;

// Adapted from trisomie21's solar shader approach
// Smooth 3D noise with configurable resolution
float snoise(vec3 uv, float res) {
    const vec3 s = vec3(1e0, 1e2, 1e4);
    uv *= res;
    vec3 uv0 = floor(mod(uv, res)) * s;
    vec3 uv1 = floor(mod(uv + vec3(1.0), res)) * s;
    vec3 f = fract(uv);
    f = f * f * (3.0 - 2.0 * f);
    vec4 v = vec4(uv0.x + uv0.y + uv0.z,
                  uv1.x + uv0.y + uv0.z,
                  uv0.x + uv1.y + uv0.z,
                  uv1.x + uv1.y + uv0.z);
    vec4 r = fract(sin(v * 1e-3) * 1e5);
    float r0 = mix(mix(r.x, r.y, f.x), mix(r.z, r.w, f.x), f.y);
    r = fract(sin((v + uv1.z - uv0.z) * 1e-3) * 1e5);
    float r1 = mix(mix(r.x, r.y, f.x), mix(r.z, r.w, f.x), f.y);
    return mix(r0, r1, f.z) * 2.0 - 1.0;
}

void main() {
    vec3 N = normalize(normal);
    vec3 V = normalize(eye - pos);
    vec3 P = normalize(vertPos);

    // Spherical coords for base UV
    float u = atan(P.z, P.x) / TAU + 0.5;
    float v = asin(clamp(P.y, -1.0, 1.0)) / PI + 0.5;

    // Time-varying 3D position for volumetric-like noise
    float t = time * 0.015;
    vec3 coord = P * 2.0;

    // Turbulent FBM — multiple octaves for organic solar surface
    float power = 0.0;
    float freqs = 3.0;
    for (int i = 0; i < 6; i++) {
        float n = snoise(coord + vec3(t * 0.1 * float(i + 1), -t * 0.05, t * 0.03), freqs);
        power += clamp(n, 0.0, 1.0) / freqs;
        freqs *= 2.0;
        coord += vec3(n * 0.04);  // domain warping — each octave shifts the next
    }

    // Map turbulence to color via sun texture lookup
    // Use power as a 1D lookup across the texture (horizontal gradient)
    vec2 texCoord = vec2(power * 0.8 + 0.1, 0.5);
    // Also sample with UV for surface detail
    vec2 texCoord2 = vec2(u + power * 0.05, v + power * 0.03);
    vec3 texColor = texture(sunTex, texCoord).rgb;
    vec3 texDetail = texture(sunTex, texCoord2).rgb;
    vec3 surface = mix(texColor, texDetail, 0.4);

    // Bright spots (solar flares / active regions)
    float brightness = pow(power, 1.5) * 1.5;
    surface *= 0.6 + brightness;

    // Limb darkening — edges darker and redder
    float NdotV = max(0.0, dot(N, V));
    float limb = 0.15 + 0.85 * pow(NdotV, 0.4);
    vec3 limbColor = mix(starTint * vec3(0.8, 0.3, 0.1), starTint, pow(NdotV, 0.3));

    // Subtle pulsing
    float pulse = 1.0 + 0.015 * sin(time * 0.3) + 0.01 * sin(time * 0.73);

    // Final color — bright enough for bloom
    vec3 finalColor = surface * limbColor * starTemp * limb * pulse * 2.5;

    setAlbedo(finalColor);
    setAlpha(1.0);
    setDepth();
    setNormal(N);
    setRoughness(0);
    setMaterial(Material_NoShade);
}
