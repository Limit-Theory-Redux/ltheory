#include fragment
#include math
#include noise
#include texcube

uniform sampler2D baseMoonTex;
uniform float seed;
uniform float craterDensity;     // 0.0 - 1.0: sparse → dense
uniform float craterSharpness;   // 0.0 - 1.0: soft → sharp craters
uniform float mountainHeight;    // 0.0 - 1.0: flat → tall highlands
uniform float mountainScale;     // 0.0 - 1.0: broad → fine mountain features
uniform float mariaAmount;       // 0.0 - 1.0: no maria → strong maria
uniform float proceduralBlend;   // 0.0 - 1.0: photo → procedural
uniform float brightRayStrength; // 0.0 - 1.0: no rays → strong bright rays

mat3 randomRotation(float s) {
    float a = fract(s * 0.0183) * 6.28318;
    float b = fract(s * 0.0271) * 6.28318;
    float c = fract(s * 0.0419) * 6.28318;
    float ca = cos(a), sa = sin(a);
    float cb = cos(b), sb = sin(b);
    float cc = cos(c), sc = sin(c);
    return mat3(
        cb * cc,          -cb * sc,         sb,
        sa * sb * cc + ca * sc,  sa * sb * sc - ca * cc,  -sa * cb,
        -ca * sb * cc + sa * sc, -ca * sb * sc - sa * cc,  ca * cb
    );
}

vec2 dirToUV(vec3 dir) {
    vec3 n = normalize(dir);
    float u = 0.5 + atan(n.z, n.x) / (2.0 * PI);
    float v = 0.5 - asin(clamp(n.y, -1.0, 1.0)) / PI;
    return vec2(u, v);
}

float genCraters(vec3 p, float density) {
    p = randomRotation(seed) * p;

    float result = 0.0;
    float amp = 1.0;
    float totalAmp = 0.0;
    float scale = mix(1.0, 4.0, density);  // Density controls base frequency

    for (int i = 0; i < 6; ++i) {
        vec3 sp = p * scale;
        vec3 cell = floor(sp + 0.5);
        vec3 frac = sp - cell;

        float minDist = 1e30;

        for (int zo = -1; zo <= 1; ++zo)
        for (int yo = -1; yo <= 1; ++yo)
        for (int xo = -1; xo <= 1; ++xo) {
            vec3 neighbor = cell + vec3(xo, yo, zo);
            float h = dot(neighbor, vec3(12.9898, 78.233, 45.164)) + seed + float(i)*113.7;
            vec3 offset = fract(sin(h) * vec3(4372.137, 8371.377, 1890.643)) * 0.7 - 0.35;
            vec3 fp = offset + vec3(xo, yo, zo);
            float dist = dot(frac - fp, frac - fp);
            minDist = min(minDist, dist);
        }

        minDist = sqrt(minDist);

        float hash = fract(sin(dot(cell, vec3(12.9898, 78.233, 45.164))) * 43758.5453);
        float radius = mix(0.2, 0.55, hash);
        float depth = mix(0.5, 1.2, hash);
        float d = minDist / radius;

        float crater = 0.0;
        if (d < 1.0) {
            // Sharpness controls falloff and rim
            float sharpnessFactor = mix(1.0, 2.8, craterSharpness);
            float shape = pow(smoothstep(1.0, 0.0, d), sharpnessFactor);
            crater = shape * depth;

            float rimStrength = mix(0.2, 0.6, craterSharpness);
            float rim = smoothstep(0.7, 0.9, d) - smoothstep(0.9, 1.0, d);
            crater -= rim * rimStrength * depth;
        }

        result += amp * crater;
        totalAmp += amp;
        amp *= 0.58;
        scale *= 2.05;
    }
    return result / totalAmp;
}

float genBrightRays(vec3 p) {
    p = randomRotation(seed + 51.9) * p * 0.6;

    float brightness = 0.0;
    float amp = 1.0;

    for (int i = 0; i < 5; ++i) {
        vec3 sp = p * (0.8 + float(i)*0.4);
        vec3 cell = floor(sp + 0.5);
        vec3 frac = sp - cell;

        float minDist = 1e30;

        for (int zo = -1; zo <= 1; ++zo)
        for (int yo = -1; yo <= 1; ++yo)
        for (int xo = -1; xo <= 1; ++xo) {
            vec3 neighbor = cell + vec3(xo, yo, zo);
            float h = dot(neighbor, vec3(12.9898, 78.233, 45.164)) + seed + 97.3;
            vec3 offset = fract(sin(h) * vec3(4372.137, 8371.377, 1890.643)) * 0.6 - 0.3;
            vec3 fp = offset + vec3(xo, yo, zo);
            float dist = dot(frac - fp, frac - fp);
            if (dist < minDist) minDist = dist;
        }

        minDist = sqrt(minDist);

        float hash = fract(sin(dot(cell, vec3(12.9898, 78.233, 45.164))) * 43758.5453);
        if (hash > 0.8) {
            float radius = mix(1.2, 3.5, pow(hash - 0.8, 0.5));
            float d = minDist / radius;
            if (d < 1.0) {
                float ray = pow(smoothstep(1.0, 0.0, d), 0.4);
                brightness += amp * ray * (hash - 0.8) * 8.0;
            }
        }
        amp *= 0.6;
    }
    return saturate(brightness) * brightRayStrength;
}

float genMariaMask(vec3 p) {
    p = randomRotation(seed + 9.7) * p;
    float m = 0.0;
    m += smoothstep(0.4, 0.7, fCellNoise(p * 0.7, seed + 15.0, 2, 1.5)) * 0.7;
    m += smoothstep(0.3, 0.8, fCellNoise(p * 1.1, seed + 18.0, 1, 1.0)) * 0.3;
    return saturate(m);
}

float genFineDetail(vec3 p) {
    p = randomRotation(seed + 23.1) * p;
    float n = 0.0;
    float w = 1.0;
    for (int i = 0; i < 4; ++i) {
        n += w * smoothNoise(p * (25.0 + float(i)*12.0));
        w *= 0.5;
    }
    return n * 0.06;
}

float genHeight(vec3 p) {
    float maria = genMariaMask(p) * mariaAmount;
    float craters = genCraters(p, craterDensity);
    // mountainScale: 0.0 = very broad (0.4), 1.0 = finer (2.0)
    float mScale = mix(0.4, 2.0, mountainScale);
    float mountains = fCellNoise(randomRotation(seed + 11.7) * p * mScale, seed + 30.0, 5, 1.8) * mountainHeight * 0.6;
    float fine = genFineDetail(p);

    float procedural = 0.5 + (craters * (1.0 - maria * 0.6)) * 0.5 + mountains * 0.3 + fine;

    vec2 uv = dirToUV(p);
    vec4 tex = texture(baseMoonTex, uv);
    float photoHeight = dot(tex.rgb, vec3(0.299, 0.587, 0.114));

    return saturate(mix(photoHeight * 0.25 + 0.4, procedural, proceduralBlend));
}

float genColor(vec3 p) {
    vec2 uv = dirToUV(p);
    vec4 tex = texture(baseMoonTex, uv);
    float photoColor = dot(tex.rgb, vec3(0.299, 0.587, 0.114));

    float maria = genMariaMask(p) * mariaAmount;
    float variation = 0.5 + 0.12 * (fCellNoise(randomRotation(seed + 17.4) * p * 3.0, seed + 40.0, 4, 1.6) - 0.5);
    variation = mix(variation, variation * 0.6 - 0.05, maria);

    float brightRays = genBrightRays(p);
    float proceduralColor = saturate(variation + genFineDetail(p) + brightRays * 0.4);

    return saturate(mix(photoColor * 1.1 - 0.05, proceduralColor, proceduralBlend));
}

float genRoughness(vec3 p) {
    vec2 uv = dirToUV(p);
    vec4 tex = texture(baseMoonTex, uv);
    float photoRough = 0.75 - dot(tex.rgb, vec3(0.333)) * 0.15;

    float brightRays = genBrightRays(p);
    float procRough = 0.78 - genCraters(p, craterDensity) * 0.12 + genFineDetail(p) * 0.15 - brightRays * 0.08;

    return saturate(mix(photoRough, procRough, proceduralBlend));
}

void main() {
    vec3 p = cubeMapDir(uv);

    float height = genHeight(p);
    float color = genColor(p);
    float roughness = genRoughness(p);

    outColor = vec4(height, color, roughness, 0.0);
}