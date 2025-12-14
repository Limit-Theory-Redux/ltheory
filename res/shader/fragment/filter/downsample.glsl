#version 330

uniform sampler2D src;
in vec2 uv;
out vec4 outColor;

void main() {
    outColor = texture(src, uv);
}