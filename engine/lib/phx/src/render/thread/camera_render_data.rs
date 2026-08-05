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
            normalize_plane(row3 + row0), // Left
            normalize_plane(row3 - row0), // Right
            normalize_plane(row3 + row1), // Bottom
            normalize_plane(row3 - row1), // Top
            normalize_plane(row3 + row2), // Near
            normalize_plane(row3 - row2), // Far
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

/// Normalize a plane equation `ax + by + cz + d = 0` (packed as
/// `Vec4(a, b, c, d)`) so that `plane.xyz . point + plane.w` gives the true
/// signed distance from `point` to the plane.
///
/// `Vec4::normalize()` would divide by `sqrt(a^2+b^2+c^2+d^2)`, which
/// incorrectly folds `d` into the divisor; the correct divisor is the
/// normal's own length, `sqrt(a^2+b^2+c^2)` - i.e. `plane.truncate().length()`.
/// Getting this wrong doesn't change which side of the plane a point is on,
/// only the computed *distance* to it, so it shows up as incorrect culling
/// specifically near the frustum boundary (spheres popping in/out too early
/// or too late), not as a wholesale broken frustum.
fn normalize_plane(plane: Vec4) -> Vec4 {
    let len = plane.truncate().length();
    if len > 0.0 { plane / len } else { plane }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn distance(plane: Vec4, center: Vec3) -> f32 {
        plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w
    }

    #[test]
    fn plane_normalization_uses_normal_length_not_full_vector_length() {
        // Plane x = 5 (inward normal points toward -x), scaled by 2 so the
        // normal isn't already unit length: -2x + 10 = 0 -> (-2, 0, 0, 10).
        // Normal length = 2; full 4-vector length = sqrt(4+100) ~= 10.2 - a
        // large enough gap between the two divisors that a sphere clearly
        // outside the frustum gets wrongly kept if `d` is folded into the
        // divisor (the old `Vec4::normalize()` bug).
        let plane = Vec4::new(-2.0, 0.0, 0.0, 10.0);
        let correct = normalize_plane(plane);
        let buggy = plane.normalize(); // what the old code did

        assert!(
            (correct.x - -1.0).abs() < 1e-6 && (correct.w - 5.0).abs() < 1e-6,
            "correct normalization should divide by the normal length only: got {correct:?}"
        );

        // A sphere well past the plane on the "outside" - should be culled.
        let center = Vec3::new(8.0, 0.0, 0.0);
        let radius = 1.0;

        let correct_distance = distance(correct, center);
        let buggy_distance = distance(buggy, center);

        assert!(
            correct_distance < -radius,
            "sphere at x=8 (radius 1) is fully past the x=5 plane and must be culled \
             with correct normalization, got distance {correct_distance}"
        );
        assert!(
            buggy_distance >= -radius,
            "sanity check: this scenario is chosen so the old Vec4::normalize() bug \
             under-scales the distance enough to wrongly keep a sphere that should be \
             culled - if this fails, the scenario no longer exercises the bug"
        );
    }
}
