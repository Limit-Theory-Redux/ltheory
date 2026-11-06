#include fragment

void main() {
  // One source texel alternates red/green. The fractional reduction shaders
  // intentionally sample between adjacent source texels, so linear filtering
  // produces a blend while point filtering preserves one texel.
  float checker = mod(floor(uv.x * 1280.0), 2.0);
  outColor = vec4(checker, 1.0 - checker, 0.0, 1.0);
}
