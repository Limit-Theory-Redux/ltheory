/* -- Instanced World-View-Projection via texture-fetch (asteroids) ----------
   Texture-fetch instancing variant (GL 3.3): per-instance attribute is a
   u32 INDEX (location 10) into a static data texture. The texture holds
   4 RGBA32F texels per asteroid, laid out as W x H (W = 4096, multiple of
   4 so an asteroid's texels never straddle a row):
     texel i*4+0: rotScale column 0
     texel i*4+1: rotScale column 1
     texel i*4+2: rotScale column 2
     texel i*4+3: xyz = world position (belt-relative), w = scale
   The eye-relative translation (beltOrigin + pos - eye) is composed here,
   so the producer uploads 4 bytes/instance instead of an 84-byte
   InstanceData - the key to 100k+ asteroids on the main thread.
   Maps 1:1 to a storage buffer under wgpu.
----------------------------------------------------------------------------- */

#include vertex

#autovar mat4 mView
#autovar mat4 mProj

uniform sampler2D instanceDataTex;
uniform vec3 beltOrigin;
uniform vec3 cameraEye;

layout(location = 10) in uint instanceIndex;

out vec3 objPos;
out float vertScale;

void main() {
  VS_BEGIN
  int t = int(instanceIndex) * 4;
  int W = textureSize(instanceDataTex, 0).x;
  ivec2 base = ivec2(t % W, t / W);
  mat3 rs = mat3(
    texelFetch(instanceDataTex, base, 0).xyz,
    texelFetch(instanceDataTex, base + ivec2(1, 0), 0).xyz,
    texelFetch(instanceDataTex, base + ivec2(2, 0), 0).xyz
  );
  vec4 posScale = texelFetch(instanceDataTex, base + ivec2(3, 0), 0);

  normal = normalize(rs * vertex_normal);
  vec3 wp = rs * vertex_position + (posScale.xyz + beltOrigin - cameraEye);
  pos = wp;
  objPos = vertex_position;
  vertScale = posScale.w;
  gl_Position = mProj * (mView * vec4(wp, 1.0));
  VS_END
}
