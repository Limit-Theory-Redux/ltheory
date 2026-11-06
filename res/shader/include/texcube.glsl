#ifndef include_texcube
#define include_texcube

/* Cube-face generation (`TexGen.Cube`, `gen_ir_map`): the including shader
   declares its group-2 `Params` block with `vec4 genLook; vec4 genUp;
   vec4 genSize;` as the first three members (the engine writes the face's
   look and up vectors and size there), and includes this file after the
   block. */

#define cubeLook genLook.xyz
#define cubeUp genUp.xyz
#define cubeSize genSize.x

vec3 cubeMapDir(vec2 uv) {
  uv = 2.0 * uv - vec2(1.0, 1.0);
  vec3 cubeRight = normalize(cross(cubeUp, cubeLook));
  return normalize(cubeLook + uv.x * cubeRight - uv.y * cubeUp);
}

#endif
