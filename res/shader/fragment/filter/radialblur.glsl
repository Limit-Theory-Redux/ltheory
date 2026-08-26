#include filter
#include math
#include color
#include noise

uniform float strength;
uniform float scanlines;
uniform vec2 center;
uniform sampler2D depthBuffer;  // Add depth buffer

const float k = 1.0;
const float a = 0.005;

void main() {
  vec3 cc = texture(src, uv).xyz;
  float centerDepth = texture(depthBuffer, uv).r;
  
  float w = 1.0;
  vec2 dir = (uv - center);
  dir *= k;
  dir = sign(dir) * pow2(dir);
  dir /= k;
  
  vec2 uvp = uv;
  
  vec3 c = cc * cc;
  float tw = 1.0;
  
  for (int i = 0; i < 32; ++i) {
    w *= 0.9;
    uvp += a * dir;
    
    // Sample depth and reject samples with large depth discontinuity
    float sampleDepth = texture(depthBuffer, uvp).r;
    float depthDiff = abs(centerDepth - sampleDepth);
    float depthWeight = exp(-depthDiff * 100.0);  // Reject samples far in depth
    
    vec3 sample = texture(src, uvp).xyz;
    c += w * depthWeight * sample * sample;
    tw += w * depthWeight;
  }
  
  c /= tw;
  c = sqrt(max(c, 0.0));
  
  float r = length(2.0 * (uv - center));
  float f2 = 1.0 - exp(-r);
  
  c = mix(c,
    c * (1.0 + f2 * vec3(0.5, 0.2, 0.1) * sin(radians(180.0) * gl_FragCoord.y)),
    scanlines);

  c = mix(cc, c, strength);
  outColor = vec4(c, 1.0);
}