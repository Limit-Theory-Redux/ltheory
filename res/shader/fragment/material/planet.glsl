#include fragment
#include deferred
#include math
#include color
#include noise
#include scattering2

#autovar vec3 eye
#autovar vec3 starDir

uniform samplerCube surface;
uniform vec3 origin;
uniform vec3 color1;
uniform vec3 color2;
uniform vec3 color3;
uniform vec3 color4;
uniform float heightMult;
uniform float oceanLevel;
uniform float time;

const float kSpecular = 1.0;
const vec3 kOceanColor = vec3(0.01, 0.13, 0.20);

// Terrain height from noise
float heightFn(float h, int octaves, float roughness) {
    float total = 1.0;
    float tw = 0.0;
    float f = PI;
    float w = 1.0;
    float off = 17.371;

    for (int i = 0; i < octaves; ++i) {
        total += w * (0.5 + 0.5 * sin(f * h + off));
        tw += w;
        w *= roughness;
        f *= 2.0;
        off += 2.3337;
    }
    total /= tw;
    return 1.0 - exp(-2.0 * pow2(max(0.0, total - 0.5)));
}

// Shadow visibility along ray to sun
float visibility(samplerCube map, vec3 p, int octaves, float roughness, float offset, float radius, float strength) {
    vec3 toStar = -starDir;
    const float samples = 8.0;
    float v = 0.0;
    for (float i = 0.0; i < samples; ++i) {
        vec3 sp = normalize(mix(p, toStar, radius * (i + 1.0) / samples));
        float h = heightFn(texture(map, sp).x, octaves, roughness);
        float rh = h - (offset + (length(sp) - 1.0));
        v += exp(-strength * heightMult * max(0.0, rh));
    }
    return v / samples;
}

// Sample cloud texture with zonal wind (east-west flow)
float getCloudLayer(vec3 samplePos, float layerSpeed, float scale, float t) {
    float r = length(samplePos);
    float theta = acos(samplePos.y / r);  // Polar angle
    float phi = atan(samplePos.z, samplePos.x);  // Azimuth

    phi += layerSpeed * t;  // Zonal wind

    // Light turbulence in azimuth
    float turb = sin(samplePos.x * 3.0 + t * 0.3) * 0.01 +
                 cos(samplePos.z * 2.5 + t * 0.25) * 0.01;
    phi += turb;

    // Rebuild direction
    float sinTheta = sin(theta);
    vec3 cloudDir = vec3(
        sinTheta * cos(phi),
        cos(theta),
        sinTheta * sin(phi)
    ) * r;

    cloudDir = normalize(samplePos + (cloudDir - samplePos) * scale);

    float tex = texture(surface, cloudDir).z;
    return pow(tex, 1.4);  // Sharpen clouds
}

void main() {
    vec3 L = starDir;
    vec3 N = normalize(normal);
    vec3 V = normalize(pos - eye);
    float NL = dot(N, L);
    float light = mix(exp(-max(0.0, pow(1.0 - NL, 4.0))), 1.0, 0.01);

    // Sample terrain map
    vec4 map = texture(surface, vertPos);
    float h1 = heightFn(map.x, 9, 0.70);
    float h2 = heightFn(map.x, 3, 0.20);

    // Base terrain color
    vec3 color = mix(color1, color2, h1);
    color = 1.0 - exp(-pow2(3.0 * color));
    color *= visibility(surface, vertPos, 9, 0.70, h1, 0.002, 2.0);
    color = mix(color, kOceanColor, 1.0 - exp(-sqrt(16.0 * max(0.0, h2 - 0.8))));

    // Multi-layer clouds
    float cloudSpeedMod = 0.01;
    float cloud1 = getCloudLayer(vertPos, 0.012, 1.0, time * cloudSpeedMod);
    float cloud2 = getCloudLayer(vertPos, 0.007, 0.85, time * 1.3 * cloudSpeedMod);
    float cloud3 = getCloudLayer(vertPos, 0.003, 0.7, time * 0.7 * cloudSpeedMod);

    float clouds = (1.0 - cloud1) * 0.35 + (1.0 - cloud2) * 0.30 + (1.0 - cloud3) * 0.25;
    clouds = pow(clouds, 1.1);

    // Latitude-based coverage
    float latitude = abs(vertPos.y);
    float latitudeVariation = smoothstep(0.3, 0.7, latitude);
    clouds = mix(clouds, clouds * 1.2, latitudeVariation * 0.3);

    // Cloud density
    float cloudDensity = smoothstep(0.15, 0.65, clouds);
    cloudDensity = pow(cloudDensity, 0.9);
    cloudDensity *= 0.7;

    // Cloud shading
    vec3 cloudColor = vec3(0.98, 1.0, 1.02);
    float cloudThickness = cloudDensity * 2.0;
    float cloudShadow = 1.0 - cloudThickness * 0.2;
    cloudShadow = max(0.85, cloudShadow);

    float cloudNL = max(0.0, NL);
    vec3 cloudLit = cloudColor * (0.8 + 0.5 * cloudNL);

    // Subtle rim lighting
    float fresnel = pow(1.0 - max(0.0, dot(N, -V)), 4.0);
    cloudLit += vec3(0.2, 0.25, 0.3) * fresnel * cloudDensity * 0.8;

    vec3 ambient = vec3(0.22, 0.25, 0.28);
    cloudLit += ambient;

    // Blend clouds
    float finalCloudAlpha = clamp(cloudDensity * 1.2, 0.0, 0.8);
    color = mix(color * cloudShadow, cloudLit, finalCloudAlpha);

    // Atmosphere
    color *= light;
    vec4 atmo = atmosphereDefault(V, eye - origin);
    color = atmo.xyz + color * (1.0 - atmo.w);

    FRAGMENT_CORRECT_DEPTH;

    setAlbedo(color);
    setAlpha(1.0);
    setDepth();
    setNormal(N);
    setRoughness(1.0);
    setMaterial(Material_NoShade);
}