layout (location = 0) in vec4 vertex_position;
layout (location = 2) in vec2 vertex_uv;

out vec2 uv;

void main() {
  uv = vertex_uv;
  gl_Position = vertex_position;
}
