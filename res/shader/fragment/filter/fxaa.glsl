#version 330

uniform sampler2D src;
uniform vec2 size;

uniform float fxaaQualitySubpix;           
uniform float fxaaQualityEdgeThreshold;    
uniform float fxaaQualityEdgeThresholdMin; 

in vec2 uv;
out vec4 outColor;

#define FxaaTexTop(t, p) texture(t, p)
#define FxaaTexOff(t, p, o) texture(t, p + o)

vec3 FxaaPixelShader(vec2 pos, sampler2D tex, vec2 rcpFrame) {
    vec3 rgbNW = FxaaTexTop(tex, pos + vec2(-1.0, -1.0) * rcpFrame).rgb;
    vec3 rgbNE = FxaaTexTop(tex, pos + vec2( 1.0, -1.0) * rcpFrame).rgb;
    vec3 rgbSW = FxaaTexTop(tex, pos + vec2(-1.0,  1.0) * rcpFrame).rgb;
    vec3 rgbSE = FxaaTexTop(tex, pos + vec2( 1.0,  1.0) * rcpFrame).rgb;
    vec3 rgbM  = FxaaTexTop(tex, pos).rgb;

    const vec3 luma = vec3(0.299, 0.587, 0.114);
    float lumaNW = dot(rgbNW, luma);
    float lumaNE = dot(rgbNE, luma);
    float lumaSW = dot(rgbSW, luma);
    float lumaSE = dot(rgbSE, luma);
    float lumaM  = dot(rgbM,  luma);

    float lumaMin = min(lumaM, min(min(lumaNW, lumaNE), min(lumaSW, lumaSE)));
    float lumaMax = max(lumaM, max(max(lumaNW, lumaNE), max(lumaSW, lumaSE)));

    vec2 dir;
    dir.x = -((lumaNW + lumaNE) - (lumaSW + lumaSE));
    dir.y =  ((lumaNW + lumaSW) - (lumaNE + lumaSE));

    float dirReduce = max((lumaNW + lumaNE + lumaSW + lumaSE) * (0.25 * fxaaQualityEdgeThreshold), fxaaQualityEdgeThresholdMin);
    float rcpDirMin = 1.0 / (min(abs(dir.x), abs(dir.y)) + dirReduce);
    dir = min(vec2(8.0), max(vec2(-8.0), dir * rcpDirMin)) * rcpFrame;

    vec3 rgbA = 0.5 * (
        FxaaTexTop(tex, pos + dir * (1.0/3.0 - 0.5)).rgb +
        FxaaTexTop(tex, pos + dir * (2.0/3.0 - 0.5)).rgb);
    vec3 rgbB = rgbA * 0.5 + 0.25 * (
        FxaaTexTop(tex, pos + dir * -0.5).rgb +
        FxaaTexTop(tex, pos + dir *  0.5).rgb);

    float lumaB = dot(rgbB, luma);
    if ((lumaB < lumaMin) || (lumaB > lumaMax)) return rgbA;
    return rgbB;
}

void main() {
    vec2 rcpFrame = 1.0 / size;
    vec3 color = FxaaPixelShader(uv, src, rcpFrame);
    outColor = vec4(color, 1.0);
}