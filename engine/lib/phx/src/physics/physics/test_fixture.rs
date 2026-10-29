//! Bodies and steps shared by the physics tests.

use super::*;

/// One frame at 60 Hz, in seconds.
pub(super) const STEP: f32 = 1.0 / 60.0;
pub(super) const SHIP_MASS: f32 = 5000.0;

/// A sphere of `radius` game units with a ship's mass, not yet in a world.
pub(super) fn sphere_body(radius: f64, kinematic: bool) -> Box<RigidBody> {
    let mut body = RigidBody::new_sphere();
    body.set_scale(radius as f32);
    body.set_mass(SHIP_MASS);
    body.set_kinematic(kinematic);
    body
}
