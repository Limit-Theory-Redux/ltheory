#version 330

uniform float casSharpness;  // 0.0 = none, 1.0 = strong
uniform vec2 size;
uniform sampler2D src;

in vec2 uv;
out vec4 outColor;

void main() {
    vec3 col = texture(src, uv).rgb;

    vec3 colN = texture(src, uv + vec2( 0.0,  1.0) / size).rgb;
    vec3 colS = texture(src, uv + vec2( 0.0, -1.0) / size).rgb;
    vec3 colE = texture(src, uv + vec2( 1.0,  0.0) / size).rgb;
    vec3 colW = texture(src, uv + vec2(-1.0,  0.0) / size).rgb;

    float lumaCenter = dot(col,   vec3(0.2126, 0.7152, 0.0722));
    float lumaN      = dot(colN, vec3(0.2126, 0.7152, 0.0722));
    float lumaS      = dot(colS, vec3(0.2126, 0.7152, 0.0722));
    float lumaE      = dot(colE, vec3(0.2126, 0.7152, 0.0722));
    float lumaW      = dot(colW, vec3(0.2126, 0.7152, 0.0722));

    float lumaMin = min(lumaCenter, min(min(lumaN, lumaS), min(lumaE, lumaW)));
    float lumaMax = max(lumaCenter, max(max(lumaN, lumaS), max(lumaE, lumaW)));

    float contrast = lumaMax - lumaMin;
    float adaptiveSharp = casSharpness * (1.0 - smoothstep(0.4, 1.2, contrast));

    // Sharpened luma 
    float sharpenedLuma = lumaCenter + adaptiveSharp * (lumaCenter * 4.0 - (lumaN + lumaS + lumaE + lumaW));

    // Clamp to local range to prevent halos/overshoot
    sharpenedLuma = clamp(sharpenedLuma, lumaMin, lumaMax);

    // Apply proportionally to RGB
    float lumaRatio = sharpenedLuma / max(lumaCenter, 0.0001);
    vec3 finalColor = col * lumaRatio;

    outColor = vec4(finalColor, 1.0);
}