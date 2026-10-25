#ifndef include_fdm
#define include_fdm

#include texturing

const float kDefaultMaxFreq = 1.0;

float getFDMFrequency() {
  return kDefaultMaxFreq / pow(length(pos - eye), 0.75);
}

/* Frequency-domain-mixing parameters. Call sites sample the texture at both
   frequencies and blend (sampler parameters are avoided for wgpu/naga):
     float fLo, fHi, fT;
     getFDMParams(fLo, fHi, fT);
     vec4 c = mix(texture(tex, triplanarCoords(fLo * p)),
                  texture(tex, triplanarCoords(fHi * p)), fT); */
void getFDMParams(out float freqLo, out float freqHi, out float t) {
  float frequency = getFDMFrequency();
  freqHi = pow(2.0, ceil(log2(frequency)));
  freqLo = freqHi * 0.5;
  t = frequency / freqLo - 1.0;
}

#endif
