#include common

layout(location = 0) out vec4 outBuffer0;
layout(location = 1) out vec4 outBuffer1;
layout(location = 2) out vec4 outZBufferL;

void main() {
  // Sentinel values identify attachment order without lighting or compositing.
  outBuffer0 = vec4(1.0, 0.0, 0.0, 1.0);
  outBuffer1 = vec4(0.0, 1.0, 0.0, 1.0);
  outZBufferL = vec4(0.75, 0.0, 0.0, 1.0);
}
