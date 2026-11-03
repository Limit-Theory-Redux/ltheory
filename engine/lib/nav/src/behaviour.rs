use glam::DVec3;

use crate::agent::{Body, Goal, Profile};

/// Below this squared length (length units squared), a vector is too short to take a direction from.
pub(crate) const TINY_SQ: f64 = 1e-24;

/// What a behaviour wants: a velocity, and the direction the agent moves in to get it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Wanted {
    /// World velocity, in length units per second.
    pub velocity: DVec3,
    /// Unit direction of travel; zero when the agent has nowhere to go.
    pub direction: DVec3,
}

impl Wanted {
    pub(crate) const NONE: Self = Self {
        velocity: DVec3::ZERO,
        direction: DVec3::ZERO,
    };
}

/// Fly straight at the goal and stop on it. Coasting under linear drag `k` from speed `v` covers
/// `v / k`, so the wanted speed is `distance · k`, capped at the profile's top speed.
pub(crate) fn arrive(body: &Body, profile: &Profile, goal: &Goal) -> Wanted {
    let to_target = goal.target - body.position;
    let distance_sq = to_target.length_squared();
    if distance_sq < TINY_SQ {
        return Wanted::NONE;
    }
    let distance = distance_sq.sqrt();
    let direction = to_target / distance;
    let speed = profile.max_speed.min(distance * profile.linear_drag);
    Wanted {
        velocity: direction * speed,
        direction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Frame;

    fn body_at(position: DVec3) -> Body {
        Body {
            position,
            velocity: DVec3::ZERO,
            frame: Frame {
                forward: DVec3::NEG_Z,
                right: DVec3::X,
                up: DVec3::Y,
            },
            radius: 1.0,
        }
    }

    const PROFILE: Profile = Profile {
        max_speed: 100.0,
        linear_drag: 2.0,
    };

    fn goal_at(target: DVec3) -> Goal {
        Goal {
            target,
            arrival_radius: 1.0,
        }
    }

    #[test]
    fn far_goal_gets_top_speed() {
        let wanted = arrive(
            &body_at(DVec3::ZERO),
            &PROFILE,
            &goal_at(DVec3::new(0.0, 0.0, -1000.0)),
        );
        assert_eq!(wanted.velocity, DVec3::new(0.0, 0.0, -100.0));
        assert_eq!(wanted.direction, DVec3::NEG_Z);
    }

    #[test]
    fn near_goal_gets_the_drag_stopping_speed() {
        let wanted = arrive(
            &body_at(DVec3::ZERO),
            &PROFILE,
            &goal_at(DVec3::new(10.0, 0.0, 0.0)),
        );
        assert_eq!(wanted.velocity, DVec3::new(20.0, 0.0, 0.0));
    }

    #[test]
    fn goal_under_the_agent_wants_nothing() {
        let wanted = arrive(&body_at(DVec3::ONE), &PROFILE, &goal_at(DVec3::ONE));
        assert_eq!(wanted, Wanted::NONE);
    }
}
