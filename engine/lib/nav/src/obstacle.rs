use glam::DVec3;

/// A sphere agents steer around. It keeps its velocity over the avoidance horizon.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Obstacle {
    /// Length units.
    pub position: DVec3,
    /// Length units per second.
    pub velocity: DVec3,
    /// Length units.
    pub radius: f64,
}
