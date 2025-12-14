#include filter
#include color
#include math
#include gamma
#include noise
#include bezier

uniform sampler2D src;
uniform float exposure;
uniform vec2 size;

const float b = 1.25;
const float k = 2.30;

const float kVignetteStrength = 0.25;
const float kVignetteHardness = 32.0;

#define HDR 1
#define COLOR_GRADING 1
#define DESAT 0
#define VIGNETTE 1

void main() {
    vec4 cc = texture(src, uv);
    vec3 c = cc.rgb * exposure;  // Apply exposure here
    c = gamma(c);                // To linear (assuming src is in gamma)

    #if VIGNETTE
    {
        vec2 uvp = vec2(1.0, 1.0) - 2.0 * abs(vec2(0.5, 0.5) - uv);
        c *= 1.0 - kVignetteStrength * exp(-kVignetteHardness * uvp.x);
        c *= 1.0 - kVignetteStrength * exp(-kVignetteHardness * uvp.y);
    }
    #endif

    #if HDR
    c /= pow(lum(c), mix(0.25, 0.0, lum(c)));
    #endif

    // Expmap with contrast correction
    c = 1.0 - exp(-k * pow(c, 1.25 + c));

    #if COLOR_GRADING
    {
        c = beziernorm3(c,
            vec3(0.25, 0.20 + 0.1 * uv.x, 0.35 - 0.15 * uv.y),
            vec3(0.40, 0.50 - 0.20 * uv.y, 0.50),
            vec3(0.80 + 0.2 * uv.y, 0.80, 0.80 - 0.40 * sqrt(uv.x * uv.y))
        );
    }
    #endif

    #if DESAT
    {
        vec3 hsl = toHSL(c);
        hsl.y = mix(hsl.y, 0.0, pow4(hsl.z));
        c = toRGB(hsl);
    }
    #endif

    #if HDR
    c = mix(c, vec3(lum(c)), lum(c));
    #endif

    // Dither
    c -= (2.0 * noise3(noise(uv * 16.0)) - vec3(1.0)) / 256.0;
    c = clamp(c, 0.0, 1.0);

    // Back to gamma space
    c = invGamma(c);

    outColor = vec4(c, cc.a);
}