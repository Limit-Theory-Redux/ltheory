#version 330

/*
 * FXAA 3.11 - PC Quality Implementation
 * Based on NVIDIA's FXAA algorithm by Timothy Lottes
 * This version samples 9+ pixels for better edge detection
 */

uniform sampler2D src;
uniform vec2 size;

uniform float fxaaQualitySubpix;           // 0.75 default - amount of sub-pixel aliasing removal
uniform float fxaaQualityEdgeThreshold;    // 0.166 default - minimum local contrast to apply AA
uniform float fxaaQualityEdgeThresholdMin; // 0.0833 default - trims algorithm from dark areas

in vec2 uv;
out vec4 outColor;

// Luma calculation (sRGB approximation)
float FxaaLuma(vec3 rgb) {
    return rgb.y * (0.587/0.299) + rgb.x;  // Faster approximation
}

void main() {
    vec2 rcpFrame = 1.0 / size;
    vec2 posM = uv;

    // Sample the center and immediate neighbors (9-tap)
    vec3 rgbNW = texture(src, posM + vec2(-1.0, -1.0) * rcpFrame).rgb;
    vec3 rgbNE = texture(src, posM + vec2( 1.0, -1.0) * rcpFrame).rgb;
    vec3 rgbSW = texture(src, posM + vec2(-1.0,  1.0) * rcpFrame).rgb;
    vec3 rgbSE = texture(src, posM + vec2( 1.0,  1.0) * rcpFrame).rgb;
    vec3 rgbN  = texture(src, posM + vec2( 0.0, -1.0) * rcpFrame).rgb;
    vec3 rgbS  = texture(src, posM + vec2( 0.0,  1.0) * rcpFrame).rgb;
    vec3 rgbW  = texture(src, posM + vec2(-1.0,  0.0) * rcpFrame).rgb;
    vec3 rgbE  = texture(src, posM + vec2( 1.0,  0.0) * rcpFrame).rgb;
    vec3 rgbM  = texture(src, posM).rgb;

    // Calculate luma for all samples
    float lumaNW = FxaaLuma(rgbNW);
    float lumaNE = FxaaLuma(rgbNE);
    float lumaSW = FxaaLuma(rgbSW);
    float lumaSE = FxaaLuma(rgbSE);
    float lumaN  = FxaaLuma(rgbN);
    float lumaS  = FxaaLuma(rgbS);
    float lumaW  = FxaaLuma(rgbW);
    float lumaE  = FxaaLuma(rgbE);
    float lumaM  = FxaaLuma(rgbM);

    // Find min/max luma in the local neighborhood
    float lumaMin = min(lumaM, min(min(lumaN, lumaS), min(lumaW, lumaE)));
    float lumaMax = max(lumaM, max(max(lumaN, lumaS), max(lumaW, lumaE)));

    // Expand to include corners
    lumaMin = min(lumaMin, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    lumaMax = max(lumaMax, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));

    // Local contrast check - if contrast is too low, skip AA
    float lumaRange = lumaMax - lumaMin;
    if (lumaRange < max(fxaaQualityEdgeThresholdMin, lumaMax * fxaaQualityEdgeThreshold)) {
        outColor = vec4(rgbM, 1.0);
        return;
    }

    // Sub-pixel aliasing test
    float lumaL = (lumaN + lumaS + lumaE + lumaW) * 0.25;
    float rangeL = abs(lumaL - lumaM);
    float blendL = max(0.0, (rangeL / lumaRange) - 0.25) * (1.0 / 0.75);
    blendL = min(1.0, blendL);

    // Calculate edge direction using Sobel-like filter
    float edgeHorz1 = (-2.0 * lumaN) + lumaNW + lumaNE;
    float edgeHorz2 = (-2.0 * lumaS) + lumaSW + lumaSE;
    float edgeVert1 = (-2.0 * lumaW) + lumaNW + lumaSW;
    float edgeVert2 = (-2.0 * lumaE) + lumaNE + lumaSE;

    float edgeHorz = abs(edgeHorz1) + abs(edgeHorz2);
    float edgeVert = abs(edgeVert1) + abs(edgeVert2);

    // Add center cross contribution
    float edgeHorz3 = abs(lumaW - lumaM) + abs(lumaE - lumaM);
    float edgeVert3 = abs(lumaN - lumaM) + abs(lumaS - lumaM);
    edgeHorz += edgeHorz3 * 2.0;
    edgeVert += edgeVert3 * 2.0;

    // Determine edge orientation
    bool horzSpan = edgeHorz >= edgeVert;

    // Select edge pixels based on orientation
    float luma1 = horzSpan ? lumaN : lumaW;
    float luma2 = horzSpan ? lumaS : lumaE;

    // Calculate gradients
    float gradient1 = abs(luma1 - lumaM);
    float gradient2 = abs(luma2 - lumaM);

    // Choose steeper gradient direction
    bool is1Steeper = gradient1 >= gradient2;
    float gradientScaled = 0.25 * max(gradient1, gradient2);

    // Step perpendicular to edge
    float lengthSign = horzSpan ? rcpFrame.y : rcpFrame.x;
    if (!is1Steeper) lengthSign = -lengthSign;

    // Calculate blend position
    vec2 posB = posM;
    if (!horzSpan) posB.x += lengthSign * 0.5;
    if (horzSpan)  posB.y += lengthSign * 0.5;

    // Search along edge for end
    vec2 offNP = horzSpan ? vec2(rcpFrame.x, 0.0) : vec2(0.0, rcpFrame.y);

    vec2 posN = posB - offNP;
    vec2 posP = posB + offNP;

    float lumaEndN = FxaaLuma(texture(src, posN).rgb) - lumaL;
    float lumaEndP = FxaaLuma(texture(src, posP).rgb) - lumaL;

    bool doneN = abs(lumaEndN) >= gradientScaled;
    bool doneP = abs(lumaEndP) >= gradientScaled;

    // Extend search up to 12 pixels in each direction
    if (!doneN) posN -= offNP * 1.5;
    if (!doneP) posP += offNP * 1.5;

    const int SEARCH_STEPS = 6;
    for (int i = 0; i < SEARCH_STEPS; i++) {
        if (!doneN) lumaEndN = FxaaLuma(texture(src, posN).rgb) - lumaL;
        if (!doneP) lumaEndP = FxaaLuma(texture(src, posP).rgb) - lumaL;

        doneN = doneN || (abs(lumaEndN) >= gradientScaled);
        doneP = doneP || (abs(lumaEndP) >= gradientScaled);

        if (doneN && doneP) break;

        if (!doneN) posN -= offNP * 2.0;
        if (!doneP) posP += offNP * 2.0;
    }

    // Calculate edge blend factor
    float dstN = horzSpan ? (posM.x - posN.x) : (posM.y - posN.y);
    float dstP = horzSpan ? (posP.x - posM.x) : (posP.y - posM.y);

    bool directionN = dstN < dstP;
    float dst = min(dstN, dstP);
    float spanLength = dstN + dstP;

    float pixelOffset = (-dst / spanLength) + 0.5;

    // Check if the edge search found the correct end
    float lumaMLower = lumaM - lumaL;
    bool goodSpan = (directionN ? lumaEndN : lumaEndP) < 0.0 != lumaMLower < 0.0;
    float pixelOffsetGood = goodSpan ? pixelOffset : 0.0;

    // Blend subpixel and edge antialiasing
    float pixelOffsetSubpix = max(pixelOffsetGood, blendL * blendL * fxaaQualitySubpix);

    // Apply offset
    vec2 posF = posM;
    if (!horzSpan) posF.x += pixelOffsetSubpix * lengthSign;
    if (horzSpan)  posF.y += pixelOffsetSubpix * lengthSign;

    vec3 rgbF = texture(src, posF).rgb;
    outColor = vec4(rgbF, 1.0);
}
