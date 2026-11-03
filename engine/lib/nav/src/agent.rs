use glam::DVec3;

/// Number of control axes the controller writes per agent.
pub const AXIS_COUNT: usize = 6;

/// A control axis: its position in an agent's slice of [`World::axes`](crate::World::axes).
/// Every value is normalised to `[-1, 1]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Axis {
    /// Positive turns the nose up.
    Pitch,
    /// Positive turns the nose to the right.
    Yaw,
    /// Positive rolls clockwise seen from behind. The controller leaves it at 0.
    Roll,
    ThrustForward,
    ThrustRight,
    ThrustUp,
}

/// Position of an agent in a [`World`](crate::World), fixed for the life of the world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AgentId(usize);

impl AgentId {
    pub(crate) fn new(index: usize) -> Self {
        Self(index)
    }

    /// The agent's index, from 0 in the order of [`World::add_agent`](crate::World::add_agent).
    pub fn index(self) -> usize {
        self.0
    }
}

/// An orthonormal frame: the unit directions an agent's thrust and turns are expressed in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub forward: DVec3,
    pub right: DVec3,
    pub up: DVec3,
}

/// The state of an agent's body at the start of a step, in world axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    /// Length units.
    pub position: DVec3,
    /// Length units per second.
    pub velocity: DVec3,
    pub frame: Frame,
    /// Bounding radius, in length units.
    pub radius: f64,
}

impl Body {
    /// At the origin and at rest, facing along -z with +y up, with no extent.
    pub const AT_REST: Self = Self {
        position: DVec3::ZERO,
        velocity: DVec3::ZERO,
        frame: Frame {
            forward: DVec3::NEG_Z,
            right: DVec3::X,
            up: DVec3::Y,
        },
        radius: 0.0,
    };
}

/// What limits an agent's motion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profile {
    /// The speed the agent can reach, in length units per second.
    pub max_speed: f64,
    /// Linear drag `k` in `dv/dt = -k·v`, per second.
    pub linear_drag: f64,
}

/// A point to fly to and stop at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Goal {
    pub target: DVec3,
    /// The agent has arrived when its centre is within this distance of the target.
    pub arrival_radius: f64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Agent {
    pub body: Body,
    pub profile: Profile,
    pub goal: Option<Goal>,
}
