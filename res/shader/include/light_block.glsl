/* -- Point light block (group 2) -----------------------------------------------
   Data of one deferred point light, std140, written with `pass:alloc` before
   each `drawFullscreen` (it replaces the old light uniform buffer at binding point 2). The
   block is the shader's group-2 block, so it is bound at block binding 8.

   Usage: #include light_block

   Provides:
     - lightPos (xyz position, camera-relative)
     - lightRadius (light falloff radius; 0 = no falloff)
     - lightColor (RGB color, pre-multiplied by intensity)
     - lightIntensity (light intensity multiplier)
----------------------------------------------------------------------------- */

#group 2
layout(std140) uniform PointLight {
    vec4 positionRadius;    // xyz = position, w = radius
    vec4 colorIntensity;    // rgb = color, w = intensity
};

// Convenience accessors
#define lightPos positionRadius.xyz
#define lightRadius positionRadius.w
#define lightColor (colorIntensity.rgb * colorIntensity.w)
#define lightIntensity colorIntensity.w
