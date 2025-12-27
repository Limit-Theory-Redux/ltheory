#version 330

// FXAA 2.9 - Faster quality preset
// 5-tap initial + 4-step edge search = ~9 samples vs 9+12 in quality version

uniform sampler2D src;
uniform vec2 size;
uniform float fxaaQualitySubpix;
uniform float fxaaQualityEdgeThreshold;
uniform float fxaaQualityEdgeThresholdMin;

in vec2 uv;
out vec4 outColor;

float luma(vec3 rgb) {
    return dot(rgb, vec3(0.299, 0.587, 0.114));
}

void main() {
    vec2 rcpFrame = 1.0 / size;

    // 5-tap cross pattern (faster than 9-tap)
    vec3 rgbN = texture(src, uv + vec2(0, -1) * rcpFrame).rgb;
    vec3 rgbS = texture(src, uv + vec2(0,  1) * rcpFrame).rgb;
    vec3 rgbW = texture(src, uv + vec2(-1, 0) * rcpFrame).rgb;
    vec3 rgbE = texture(src, uv + vec2( 1, 0) * rcpFrame).rgb;
    vec3 rgbM = texture(src, uv).rgb;

    float lumaN = luma(rgbN);
    float lumaS = luma(rgbS);
    float lumaW = luma(rgbW);
    float lumaE = luma(rgbE);
    float lumaM = luma(rgbM);

    float lumaMin = min(lumaM, min(min(lumaN, lumaS), min(lumaW, lumaE)));
    float lumaMax = max(lumaM, max(max(lumaN, lumaS), max(lumaW, lumaE)));
    float lumaRange = lumaMax - lumaMin;

    // Early exit for low contrast
    if (lumaRange < max(fxaaQualityEdgeThresholdMin, lumaMax * fxaaQualityEdgeThreshold)) {
        outColor = vec4(rgbM, 1.0);
        return;
    }

    // Edge direction
    float edgeH = abs(lumaN - lumaM) + abs(lumaS - lumaM);
    float edgeV = abs(lumaW - lumaM) + abs(lumaE - lumaM);
    bool isHorz = edgeH >= edgeV;

    // Blend direction
    float luma1 = isHorz ? lumaN : lumaW;
    float luma2 = isHorz ? lumaS : lumaE;
    float grad1 = abs(luma1 - lumaM);
    float grad2 = abs(luma2 - lumaM);

    float lengthSign = isHorz ? rcpFrame.y : rcpFrame.x;
    if (grad2 > grad1) lengthSign = -lengthSign;

    // Sub-pixel blend
    float lumaL = (lumaN + lumaS + lumaW + lumaE) * 0.25;
    float rangeL = abs(lumaL - lumaM);
    float blend = clamp(rangeL / lumaRange, 0.0, 1.0);
    blend = blend * blend * fxaaQualitySubpix;

    // Offset sample
    vec2 posF = uv;
    if (isHorz) posF.y += lengthSign * blend;
    else        posF.x += lengthSign * blend;

    outColor = vec4(texture(src, posF).rgb, 1.0);
}
