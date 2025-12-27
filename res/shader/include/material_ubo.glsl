/* -- Material Uniform Buffer Object -------------------------------------------
   Shared material data for per-draw properties using std140 layout.
   Binding point 1 is reserved for material data.

   Usage: #include material_ubo

   Provides:
     - matColor (RGBA base color)
     - matMetallic, matRoughness, matEmission (PBR parameters)

   Note: Binding point is set via glUniformBlockBinding in Rust code
   (GLSL 330 doesn't support binding qualifier; requires GLSL 420+)
----------------------------------------------------------------------------- */

layout(std140) uniform MaterialUBO {
    vec4 ubo_color;     // RGBA base color
    vec4 ubo_params;    // x=metallic, y=roughness, z=emission, w=padding
};

// Convenience accessors
#define matColor ubo_color
#define matMetallic ubo_params.x
#define matRoughness ubo_params.y
#define matEmission ubo_params.z
