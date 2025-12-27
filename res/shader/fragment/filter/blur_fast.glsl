#version 330

// Optimized Gaussian blur using bilinear filtering trick
// 13-tap blur with 7 texture samples (covers ~25 pixels effectively)

in vec2 uv;
out vec4 outColor;

uniform sampler2D src;
uniform vec2 dir;      // (1,0) for horizontal, (0,1) for vertical
uniform vec2 size;

void main() {
    vec2 texel = dir / size;

    // Wider Gaussian kernel for smoother bloom
    // Weights sum to 1.0
    float w0 = 0.1964825501511404;
    float w1 = 0.2969069646728344;  // offset 1.4117647058823530
    float w2 = 0.0944703978045498;  // offset 3.2941176470588234
    float w3 = 0.0103813624011998;  // offset 5.1764705882352940

    vec4 result = texture(src, uv) * w0;

    // Bilinear offsets for wider coverage
    vec2 off1 = texel * 1.4117647058823530;
    vec2 off2 = texel * 3.2941176470588234;
    vec2 off3 = texel * 5.1764705882352940;

    result += texture(src, uv + off1) * w1;
    result += texture(src, uv - off1) * w1;
    result += texture(src, uv + off2) * w2;
    result += texture(src, uv - off2) * w2;
    result += texture(src, uv + off3) * w3;
    result += texture(src, uv - off3) * w3;

    outColor = result;
}
