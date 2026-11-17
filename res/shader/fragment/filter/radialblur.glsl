#include filter
#include math
#include color
#include noise

#group 2
layout(std140) uniform Params {
    float strength;
    float scanlines;
    vec2 center;
    float reach;  // streak length: 1 = the setting's look; more for speed (motion blur)
    vec2 clearCenter;   // a disc left sharp (the travel drive's bubble), in uv
    float clearRadius;  // share of the screen height; 0 = none
    float aspect;       // screen width / height, for the disc
};
#group 3
uniform sampler2D depthBuffer;  // linear depth (slot 1; `src` is slot 0)

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

  // A longer reach: the extra part grows linearly from the centre (a zoom
  // blur), and the taps start at a random offset so long streaks don't show
  // as rows of dots
  float extra = max(0.0, reach - 1.0);
  dir += extra * 0.5 * (uv - center);
  vec2 uvp = uv + (extra > 0.0 ? noise(gl_FragCoord.xy) : 0.0) * a * dir;

  vec3 c = cc * cc;
  float tw = 1.0;

  for (int i = 0; i < 32; ++i) {
    w *= 0.9;
    uvp += a * dir;

    // Sample depth and reject samples with large depth discontinuity, as a
    // share of this pixel's depth: depths are game units from the camera's
    // world position, millions on the sky, so a fixed difference rejected
    // every sample there
    float sampleDepth = texture(depthBuffer, uvp).r;
    float depthDiff = abs(centerDepth - sampleDepth) / max(centerDepth, 1e-6);
    float depthWeight = exp(-depthDiff * 20.0);  // Reject samples far in depth

    vec3 smp = texture(src, uvp).xyz;
    c += w * depthWeight * smp * smp;
    tw += w * depthWeight;
  }

  c /= tw;
  c = sqrt(max(c, 0.0));

  float r = length(2.0 * (uv - center));
  float f2 = 1.0 - exp(-r);

  c = mix(c,
    c * (1.0 + f2 * vec3(0.5, 0.2, 0.1) * sin(radians(180.0) * gl_FragCoord.y)),
    scanlines);

  // Inside the clear disc the picture stays sharp, so the blur doesn't
  // smear what's around it into the bubble
  float keep = 1.0;
  if (clearRadius > 0.0) {
    float clearDist = length((uv - clearCenter) * vec2(aspect, 1.0));
    keep = smoothstep(clearRadius, clearRadius * 1.25 + 0.01, clearDist);
  }

  c = mix(cc, c, strength * keep);
  outColor = vec4(c, 1.0);
}
