/* -- Camera Uniform Buffer Object ---------------------------------------------
   Shared camera data across all shaders using std140 layout.
   Binding point 0 is reserved for camera data.

   Usage: #include camera_ubo

   Provides:
     - mView, mProj, mViewInv, mProjInv matrices
     - eye position (camera world position)
     - starDir (primary light direction)

   Note: Binding point is set via glUniformBlockBinding in Rust code
   (GLSL 330 doesn't support binding qualifier; requires GLSL 420+)
----------------------------------------------------------------------------- */

layout(std140) uniform CameraUBO {
    mat4 ubo_mView;
    mat4 ubo_mProj;
    mat4 ubo_mViewInv;
    mat4 ubo_mProjInv;
    vec4 ubo_eye;      // xyz = eye position, w = padding
    vec4 ubo_starDir;  // xyz = star direction, w = padding
};

// Convenience accessors (maintain compatibility with existing code)
#define mView ubo_mView
#define mProj ubo_mProj
#define mViewInv ubo_mViewInv
#define mProjInv ubo_mProjInv
#define eye ubo_eye.xyz
#define starDir ubo_starDir.xyz
