use glam::DVec3;

use crate::agent::AgentId;

/// What names an obstacle between steps, so the side an agent passes it on is remembered while
/// the set of obstacles around it changes. Keys of the obstacles a caller sets are
/// [`Body`](ObstacleKey::Body); the world names its own agents' spheres itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObstacleKey {
    /// A caller's name for the body, unique among the obstacles set together and kept for as long
    /// as the body lives.
    Body(u64),
    /// The sphere of an agent of the world.
    Agent(AgentId),
}

/// A sphere agents steer around. It keeps its velocity over the avoidance horizon.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Obstacle {
    pub key: ObstacleKey,
    /// Length units.
    pub position: DVec3,
    /// Length units per second.
    pub velocity: DVec3,
    /// Length units.
    pub radius: f64,
}
