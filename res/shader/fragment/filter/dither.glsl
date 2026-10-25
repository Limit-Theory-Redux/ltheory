#version 330

#group 2
layout(std140) uniform Params {
    float strength;
};
#group 3
uniform sampler2D src;
in vec2 uv;
out vec4 outColor;

// Simple triangular blue noise approximation (no texture needed)
float triangularNoise(vec2 n) {
    float t = fract(dot(gl_FragCoord.xy, vec2(12.9898, 78.233)));
    return fract(t * (t * 3.14 + 1.0));
}

void main() {
    vec3 color = texture(src, uv).rgb;
    float noise = (triangularNoise(uv) - 0.5) * strength / 256.0;  // small offset
    outColor = vec4(color + noise, 1.0);
}