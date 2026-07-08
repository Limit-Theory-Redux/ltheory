use glam::{Mat4, Vec3, Vec4};

/// Camera data for culling and matrix computation
#[derive(Clone, Debug)]
pub struct CameraRenderData {
    /// View matrix
    pub view: Mat4,
    /// Projection matrix
    pub projection: Mat4,
    /// View-projection matrix (cached)
    pub view_projection: Mat4,
    /// Camera position in world space
    pub position: Vec3,
    /// Frustum planes for culling (6 planes: left, right, bottom, top, near, far)
    pub frustum_planes: [Vec4; 6],
}

impl CameraRenderData {
    /// Create camera data from view and projection matrices
    pub fn new(view: Mat4, projection: Mat4, position: Vec3) -> Self {
        let view_projection = projection * view;
        let frustum_planes = Self::extract_frustum_planes(&view_projection);
        Self {
            view,
            projection,
            view_projection,
            position,
            frustum_planes,
        }
    }

    /// Extract frustum planes from view-projection matrix
    fn extract_frustum_planes(vp: &Mat4) -> [Vec4; 6] {
        let row0 = Vec4::new(vp.x_axis.x, vp.y_axis.x, vp.z_axis.x, vp.w_axis.x);
        let row1 = Vec4::new(vp.x_axis.y, vp.y_axis.y, vp.z_axis.y, vp.w_axis.y);
        let row2 = Vec4::new(vp.x_axis.z, vp.y_axis.z, vp.z_axis.z, vp.w_axis.z);
        let row3 = Vec4::new(vp.x_axis.w, vp.y_axis.w, vp.z_axis.w, vp.w_axis.w);

        [
            (row3 + row0).normalize(), // Left
            (row3 - row0).normalize(), // Right
            (row3 + row1).normalize(), // Bottom
            (row3 - row1).normalize(), // Top
            (row3 + row2).normalize(), // Near
            (row3 - row2).normalize(), // Far
        ]
    }

    /// Test if a bounding sphere is inside the frustum
    pub fn sphere_in_frustum(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.frustum_planes {
            let distance = plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w;
            if distance < -radius {
                return false;
            }
        }
        true
    }
}
