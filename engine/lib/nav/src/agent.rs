use glam::DVec3;

use crate::avoidance::Commitment;
use crate::behaviour::Behaviour;

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
    /// Radians per second, about the world axes.
    pub angular_velocity: DVec3,
    pub frame: Frame,
    /// Bounding radius, in length units.
    pub radius: f64,
}

impl Body {
    /// At the origin and at rest, facing along -z with +y up, with no extent.
    pub const AT_REST: Self = Self {
        position: DVec3::ZERO,
        velocity: DVec3::ZERO,
        angular_velocity: DVec3::ZERO,
        frame: Frame {
            forward: DVec3::NEG_Z,
            right: DVec3::X,
            up: DVec3::Y,
        },
        radius: 0.0,
    };
}

/// An acceleration, in length units per second squared.
pub type Acceleration = f64;

/// An angular acceleration, in radians per second squared.
pub type AngularAcceleration = f64;

/// A drag rate `k` in `dv/dt = -k·v`, per second.
pub type DragRate = f64;

/// What limits an agent's motion: the accelerations its thrusters and turning gear give at full
/// input along each of its own axes, and the drag that slows it. A limit of 0 means the agent
/// has no authority on that axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Profile {
    pub linear_drag: DragRate,
    pub angular_drag: DragRate,
    /// Thrust along the nose; forward and reverse are equal.
    pub accel_forward: Acceleration,
    pub accel_right: Acceleration,
    pub accel_up: Acceleration,
    /// Turn about the right axis (pitch), the up axis (yaw) and the forward axis (roll).
    pub turn_pitch: AngularAcceleration,
    pub turn_yaw: AngularAcceleration,
    pub turn_roll: AngularAcceleration,
}

#[cfg(test)]
impl Profile {
    /// A profile for unit tests: a top speed of 100 against a drag of 2, side thrust a quarter
    /// of the forward thrust, and quick turns.
    pub(crate) const TEST: Self = Self {
        linear_drag: 2.0,
        angular_drag: 5.0,
        accel_forward: 200.0,
        accel_right: 50.0,
        accel_up: 50.0,
        turn_pitch: 20.0,
        turn_yaw: 20.0,
        turn_roll: 6.0,
    };
}

impl Profile {
    /// The speed at which forward thrust balances linear drag, in length units per second;
    /// infinite without drag.
    pub fn max_speed(&self) -> f64 {
        if self.linear_drag > 0.0 {
            self.accel_forward / self.linear_drag
        } else {
            f64::INFINITY
        }
    }
}

/// The smallest arrival radius an agent is held to, in agent radii. Agents whose goals lie
/// closer together than their hulls can't all hold a place inside a smaller circle: they keep
/// avoiding each other around the goals without any of them arriving.
pub const MIN_ARRIVAL_RADII: f64 = 2.0;

/// A point to fly to and stop at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Goal {
    pub target: DVec3,
    /// The agent has arrived when its centre is within this distance of the target, or within
    /// [`MIN_ARRIVAL_RADII`] of its own radii if that is larger.
    pub arrival_radius: f64,
}

impl Goal {
    /// The distance from the target within which an agent of `body_radius` has arrived.
    pub fn effective_arrival_radius(&self, body_radius: f64) -> f64 {
        self.arrival_radius.max(body_radius * MIN_ARRIVAL_RADII)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Agent {
    pub body: Body,
    pub profile: Profile,
    pub behaviour: Option<Behaviour>,
    /// The side the agent passes the sphere it is avoiding on, kept between steps.
    pub commitment: Option<Commitment>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_arrival_radius_is_floored_at_two_agent_radii() {
        let goal = |arrival_radius| Goal {
            target: DVec3::ZERO,
            arrival_radius,
        };
        assert_eq!(
            goal(1.0).effective_arrival_radius(5.0),
            5.0 * MIN_ARRIVAL_RADII
        );
        assert_eq!(goal(100.0).effective_arrival_radius(5.0), 100.0);
        assert_eq!(goal(1.0).effective_arrival_radius(0.0), 1.0);
    }
}
