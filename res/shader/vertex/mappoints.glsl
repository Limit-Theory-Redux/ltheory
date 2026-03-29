#include vertex

uniform vec2 mapOffset;  // screen position of belt/ring center
uniform float mapZoom;   // zoom factor
uniform vec2 screenSize; // viewport size
uniform float dotSize;   // dot half-size in pixels

void main() {
  uv = vertex_uv;

  vec2 worldPos = vertex_position.xy;
  vec2 screenPos = mapOffset + worldPos * mapZoom;

  // Add pixel-sized offset based on UV corner
  vec2 cornerOffset = (vertex_uv - 0.5) * 2.0 * dotSize;
  screenPos += cornerOffset;

  vec2 ndc = (screenPos / screenSize) * 2.0 - 1.0;
  ndc.y = -ndc.y;
  gl_Position = vec4(ndc, 0.0, 1.0);
}
