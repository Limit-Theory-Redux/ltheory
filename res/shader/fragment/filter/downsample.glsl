#version 330

#group 3
uniform sampler2D src;
in vec2 uv;
out vec4 outColor;

void main() {
    outColor = texture(src, uv);
}