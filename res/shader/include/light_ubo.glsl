/* -- Light Uniform Buffer Object ----------------------------------------------
   Shared light data for deferred lighting using std140 layout.
   Binding point 2 is reserved for light data.

   Usage: #include light_ubo

   Provides:
     - lightPos (xyz position)
     - lightRadius (light falloff radius)
     - lightColor (RGB color)
     - lightIntensity (light intensity multiplier)

   Note: Binding point is set via glUniformBlockBinding in Rust code
   (GLSL 330 doesn't support binding qualifier; requires GLSL 420+)
----------------------------------------------------------------------------- */

layout(std140) uniform LightUBO {
    vec4 ubo_positionRadius;    // xyz = position, w = radius
    vec4 ubo_colorIntensity;    // rgb = color, w = intensity
};

// Convenience accessors
#define lightPos ubo_positionRadius.xyz
#define lightRadius ubo_positionRadius.w
#define lightColor (ubo_colorIntensity.rgb * ubo_colorIntensity.w)
#define lightIntensity ubo_colorIntensity.w
