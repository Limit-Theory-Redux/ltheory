/* -- View block (group 0) -------------------------------------------------------
   Per-pass frame data, std140, ring-allocated by the engine at `beginPass` and
   bound to block binding 0 (group 0). `Renderer:setCamera` supplies the camera,
   the pass supplies the UI projection and viewport. The Rust mirror is
   `ViewBlock` (render/gpu/view_block.rs); a startup assert checks size and
   offsets against what the linker reports.

   Usage: #include view_block

   Provides:
     - mView, mProj, mViewInv, mProjInv matrices
     - eye position (camera world position; rendering is camera-relative)
     - starDir (primary light direction)
     - mProjUI (orthographic projection of the pass) and mWorldViewUI
       (`pass:setUiTransform`), used by the UI/fullscreen vertex shaders
     - ubo_viewport (x, y, w, h in pixels)

   Note: GLSL 330 has no `layout(binding=)`; the preprocessor's `#group`
   directive assigns the binding (see render/gpu/layout.rs).
----------------------------------------------------------------------------- */

#group 0
layout(std140) uniform ViewBlock {
    mat4 ubo_mView;
    mat4 ubo_mProj;
    mat4 ubo_mViewInv;
    mat4 ubo_mProjInv;
    vec4 ubo_eye;          // xyz = eye position, w = 1
    vec4 ubo_starDir;      // xyz = star direction, w = padding
    mat4 ubo_mProjUI;
    mat4 ubo_mWorldViewUI;
    vec4 ubo_viewport;     // x, y, w, h in pixels
    vec4 ubo_time;         // reserved
};

// Convenience accessors (maintain compatibility with existing code)
#define mView ubo_mView
#define mProj ubo_mProj
#define mViewInv ubo_mViewInv
#define mProjInv ubo_mProjInv
#define eye ubo_eye.xyz
#define starDir ubo_starDir.xyz
#define mProjUI ubo_mProjUI
#define mWorldViewUI ubo_mWorldViewUI
