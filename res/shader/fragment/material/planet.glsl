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

// Procedural height function for terrain
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

// Compute terrain visibility for lighting
float visibility(
    samplerCube map, vec3 p, int octaves, float roughness,
    float offset, float radius, float strength)
{
    vec3 toStar = -starDir;
    const float samples = 8.0;
    float v = 0.0;
    float tw = 0.0;
    for (float i = 0.0; i < samples; ++i) {
        vec3 sp = normalize(mix(p, toStar, radius * (i + 1.0) / samples));
        float h = heightFn(texture(map, sp).x, octaves, roughness);
        float rh = h - (offset + (length(sp) - 1.0));
        v += exp(-strength * heightMult * max(0.0, rh));
    }
    return v / samples;
}

// Compute the hurricane center based on time
vec3 getHurricaneCenter(float t) {
    float cycle = mod(t * 0.04, 12.0);
    float spawnTime = smoothstep(1.0, 2.5, cycle) * (1.0 - smoothstep(8.0, 11.0, cycle));
    
    // Path along tropical/subtropical latitudes
    float pathAngle = t * 0.015 + sin(t * 0.02) * 1.5;
    float latitude = sin(t * 0.035) * 0.5 + 0.1;
    
    vec3 center = vec3(
        cos(pathAngle) * cos(latitude),
        sin(latitude),
        sin(pathAngle) * cos(latitude)
    );
    
    return normalize(center) * spawnTime;
}

// Procedural hurricane effect
float hurricaneEffect(vec3 pos, vec3 center, float intensity) {
    if (intensity < 0.01) return 0.0;

    // Project position onto tangent plane at hurricane center
    vec3 centerNorm = normalize(center);
    vec3 toPos = pos - centerNorm * dot(pos, centerNorm);

    // Tangent basis
    vec3 tangent1 = normalize(cross(centerNorm, vec3(0.0, 1.0, 0.0)));
    if (length(tangent1) < 0.1) tangent1 = normalize(cross(centerNorm, vec3(1.0, 0.0, 0.0)));
    vec3 tangent2 = normalize(cross(centerNorm, tangent1));

    // 2D hurricane coordinates
    vec2 uv = vec2(dot(toPos, tangent1), dot(toPos, tangent2));

    // Apply rotation for local spiral spin
    float localRotation = time * 0.35;
    mat2 rot = mat2(cos(localRotation), -sin(localRotation),
                    sin(localRotation),  cos(localRotation));
    uv = rot * uv;

    float r = length(uv);
    float eyeRadius = 0.015;
    float outerRadius = 0.18;
    if (r > outerRadius) return 0.0;

    // Eye and spiral coordinates
    float eye = 1.0 - smoothstep(0.0, eyeRadius, r);
    float z = log(max(r, 0.001)) * 2.0;
    float angle = atan(uv.y, uv.x) / PI;
    float spiralCoord = angle - z * 0.85 - time * 0.25;

    // Combine noise layers for spiral clouds
    float noise1 = sin(spiralCoord * 10.0 + r * 15.0 + cos(angle * 6.0) * 0.5) * 0.5 + 0.5;
    float noise2 = sin(spiralCoord * 18.0 + z * 10.0 + sin(r * 25.0) * 0.3) * 0.5 + 0.5;
    float noise3 = sin(angle * 12.0 + spiralCoord * 7.0 + r * 18.0) * 0.5 + 0.5;
    float cloudNoise = mix(noise1, noise2 * noise3, 0.5);
    cloudNoise = pow(cloudNoise, 1.2);

    // Spiral bands
    float bands = sin(spiralCoord * 10.0 + r * 18.0 - z * 3.0) * 0.5 + 0.5;
    bands = pow(bands, 1.3);
    float pattern = mix(bands, cloudNoise, 0.55);

    // Turbulence for natural variation
    float turbulence = sin(spiralCoord * 22.0 + r * 35.0) * 0.5 + 0.5;
    turbulence *= sin(spiralCoord * 30.0 - z * 8.0) * 0.5 + 0.5;
    pattern = mix(pattern, pattern * turbulence, 0.3);

    // Eye wall with contrast
    float eyeWall = smoothstep(eyeRadius, eyeRadius + 0.018, r) *
                    (1.0 - smoothstep(0.04, 0.07, r));
    eyeWall = pow(eyeWall, 0.7) * 2.0;
    float eyeWallNoise = sin(spiralCoord * 15.0 + time * 0.5) * 0.5 + 0.5;
    eyeWallNoise *= sin(angle * 14.0 + r * 30.0) * 0.5 + 0.5;
    eyeWall *= mix(0.7, 1.0, eyeWallNoise);

    // Falloff for outer spiral arms
    float armFalloff = pow(1.0 - smoothstep(0.08, outerRadius * 0.85, r), 0.5);

    // Combine eye wall and spiral arms
    float hurricane = eyeWall + pattern * armFalloff * 1.2;
    hurricane *= (1.0 - eye * 0.7);
    hurricane = pow(hurricane, 1.1) * 0.85;

    return clamp(hurricane * intensity, 0.0, 1.8);
}

// Multi-layer cloud sampling
float getCloudLayer(vec3 samplePos, float layerSpeed, float scale, float t) {
    vec3 north = vec3(0.0, 1.0, 0.0);
    vec3 east = normalize(cross(north, normalize(samplePos)));
    vec3 up = normalize(samplePos);
    
    vec3 flow = east * t * layerSpeed + cross(up, east) * t * (layerSpeed * 0.4);

    // Turbulence for natural cloud motion
    float turbulence = sin(samplePos.x * 3.0 + t * 0.3) * 0.02 +
                       cos(samplePos.z * 2.5 + t * 0.25) * 0.02;
    flow += up * turbulence;

    vec3 cloudDir = normalize(samplePos + flow * scale);
    return texture(surface, cloudDir).z;
}

void main() {
    vec3 L = starDir;
    vec3 P = pos - origin;
    vec3 N = normalize(normal);
    vec3 V = normalize(pos - eye);
    float NL = dot(N, L);
    float light = mix(exp(-max(0.0, pow(1.0 - NL, 4.0))), 1.0, 0.01);

    // Sample terrain height
    vec4 map = texture(surface, vertPos);
    float h1 = heightFn(map.x, 9, 0.70);
    float h2 = heightFn(map.x, 3, 0.20);

    // Base terrain color
    vec3 color = mix(color1, color2, h1);
    color = 1.0 - exp(-pow2(3.0 * color));

    // Apply terrain visibility
    color *= visibility(surface, vertPos, 9, 0.70, h1, 0.002, 2.0);

    // Blend in ocean
    color = mix(color, kOceanColor, 1.0 - exp(-sqrt(16.0 * max(0.0, h2 - 0.8))));

    // --- Multi-layer cloud system ---
    float cloud1 = getCloudLayer(vertPos, 0.012, 1.0, time);
    float cloud2 = getCloudLayer(vertPos, 0.007, 0.85, time * 1.3);
    float cloud3 = getCloudLayer(vertPos, 0.003, 0.7, time * 0.7);

    // Combine cloud layers
    float clouds = (1.0 - cloud1) * 0.42 + (1.0 - cloud2) * 0.38 + (1.0 - cloud3) * 0.32;

    // Adjust clouds by latitude
    float latitude = abs(vertPos.y);
    float latitudeVariation = smoothstep(0.2, 0.8, latitude);
    clouds = mix(clouds, clouds * 1.25, latitudeVariation * 0.3);

    // --- Hurricane effect ---
    vec3 hurricaneCenter = getHurricaneCenter(time);
    float hurricaneIntensity = length(hurricaneCenter);
    float hurricaneContribution = hurricaneEffect(vertPos, normalize(hurricaneCenter), hurricaneIntensity);

    // Hurricanes dominate cloud coverage locally
    clouds = max(clouds, hurricaneContribution);

    // Cloud density and opacity adjustments
    float cloudDensity = smoothstep(0.12, 0.58, clouds);
    cloudDensity = pow(cloudDensity, 0.8);
    cloudDensity *= clamp(1.0 - h1 * 0.5, 0.0, 1.0);

    // Hurricane cloud color
    float isHurricane = smoothstep(0.03, 0.15, hurricaneContribution);
    vec3 cloudColor = mix(vec3(0.92, 0.95, 0.98), vec3(0.98, 0.99, 1.0), isHurricane);

    // Cloud lighting and shadow
    float cloudThickness = cloudDensity * (1.0 + hurricaneContribution * 1.8);
    float cloudShadow = exp(-cloudThickness * 2.8);
    float cloudNL = max(0.0, NL);
    vec3 cloudLit = cloudColor * mix(0.45, 1.3, pow(cloudNL, 0.75));

    // Fresnel edge lighting
    float fresnel = pow(1.0 - max(0.0, dot(N, -V)), 2.8);
    cloudLit += vec3(0.3, 0.35, 0.4) * fresnel * cloudDensity * cloudNL;

    // Hurricane eye shading
    if (hurricaneIntensity > 0.01) {
        vec3 toHurricane = normalize(hurricaneCenter);
        vec3 toPos = vertPos - toHurricane * dot(vertPos, toHurricane);
        float distToEye = length(toPos);
        float eyeEffect = exp(-distToEye * 80.0) * hurricaneIntensity;
        cloudLit = mix(cloudLit, vec3(0.15, 0.18, 0.22), eyeEffect * 0.9);
    }

    vec3 ambient = vec3(0.2, 0.22, 0.25);

    // Final cloud blending
    float finalCloudAlpha = clamp(cloudDensity * 1.2, 0.0, 0.88);
    color = mix(color * cloudShadow, cloudLit + ambient, finalCloudAlpha);

    // Apply lighting and atmosphere
    color *= light;
    vec4 atmo = atmosphereDefault(V, eye - origin);
    color = atmo.xyz + color * (1.0 - atmo.w);

    FRAGMENT_CORRECT_DEPTH;

    // Output fragment
    setAlbedo(color);
    setAlpha(1.0);
    setDepth();
    setNormal(N);
    setRoughness(1.0);
    setMaterial(Material_NoShade);
}
