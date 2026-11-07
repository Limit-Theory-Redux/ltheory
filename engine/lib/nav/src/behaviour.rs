use glam::DVec3;

use crate::agent::{AT_REST_RADII_PER_S, Body, Goal, Profile};
use crate::braking::stopping_speed;

/// Below this squared length (length units squared), a vector is too short to take a direction from.
pub(crate) const TINY_SQ: f64 = 1e-24;

/// The fraction of the forward thrust an agent spends on holding a circular orbit.
pub const ORBIT_THRUST_SHARE: f64 = 0.5;

/// Gain of the orbit field's pull toward the circle, on the radial error `(|p|² − r²) / r²`.
/// A larger gain turns the field toward the circle more sharply.
pub const ORBIT_FIELD_GAIN: f64 = 1.0;

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

/// A position in another body's frame, in length units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameOffset {
    pub right: f64,
    pub up: f64,
    pub forward: f64,
}

/// What an agent is doing with its motion. Target bodies are snapshots: the caller refreshes them
/// before every step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Behaviour {
    /// Fly at the goal's point at top speed, without braking. An agent with no linear drag has no
    /// top speed, and gets no input.
    Seek(Goal),
    /// Fly to the goal's point and stop on it.
    Arrive(Goal),
    /// Meet a moving body `stand_off` from its hull, along the line of sight, matching its
    /// velocity.
    Pursue {
        target: Body,
        stand_off: f64,
        arrival_radius: f64,
    },
    /// Hold a slot in the leader's frame, matching the leader's velocity. The leader's turning is
    /// not fed forward.
    Follow {
        leader: Body,
        offset: FrameOffset,
        arrival_radius: f64,
    },
    /// Circle the centre at `radius` in the plane through it normal to `normal`, anticlockwise
    /// about `normal`, moving with the centre. `normal` must be non-zero: a zero normal leaves
    /// the agent matching the centre's velocity.
    Orbit {
        centre: Body,
        radius: f64,
        normal: DVec3,
        arrival_radius: f64,
    },
}

impl Behaviour {
    /// Stay where `body` is.
    pub fn hold(body: &Body, arrival_radius: f64) -> Self {
        Self::Arrive(Goal {
            target: body.position,
            arrival_radius,
        })
    }
}

/// The unit vector along `vector`, or `fallback` when it is too short to have a direction.
fn unit_or(vector: DVec3, fallback: DVec3) -> DVec3 {
    if vector.length_squared() < TINY_SQ {
        fallback
    } else {
        vector.normalize()
    }
}

/// A wanted velocity, with the direction of travel taken from it, or `fallback` when it is zero.
fn wanted_of(velocity: DVec3, fallback: DVec3) -> Wanted {
    Wanted {
        velocity,
        direction: unit_or(velocity, fallback),
    }
}

/// Fly straight at the goal and stop on it. The wanted speed is the one the agent can brake from
/// within the distance (`braking::stopping_speed`), capped at the profile's top speed.
pub(crate) fn arrive(body: &Body, profile: &Profile, goal: &Goal) -> Wanted {
    arrive_at(body, profile, goal.target, DVec3::ZERO)
}

/// Fly at a point that moves at `point_velocity` and stop on it, matching its velocity. The wanted
/// velocity is the point's velocity plus a closing speed along the line of sight: the speed the
/// agent can brake from within the distance, capped so that the total stays within the profile's
/// top speed. That keeps the bearing to the point constant, which is the lead. A point that
/// outruns the agent is chased along the line of sight at top speed.
pub(crate) fn arrive_at(
    body: &Body,
    profile: &Profile,
    point: DVec3,
    point_velocity: DVec3,
) -> Wanted {
    let to_point = point - body.position;
    let distance_sq = to_point.length_squared();
    if distance_sq < TINY_SQ {
        return wanted_of(point_velocity, DVec3::ZERO);
    }
    let distance = distance_sq.sqrt();
    let direction = to_point / distance;
    let top = profile.max_speed();
    let point_speed_sq = point_velocity.length_squared();
    let stopping = stopping_speed(profile, distance);
    let closing = if point_speed_sq < TINY_SQ {
        top.min(stopping)
    } else if point_speed_sq > top * top {
        return Wanted {
            velocity: direction * top,
            direction,
        };
    } else {
        let along = point_velocity.dot(direction);
        let room = -along + (along * along + top * top - point_speed_sq).sqrt();
        room.min(stopping)
    };
    wanted_of(point_velocity + direction * closing, direction)
}

/// Fly at the goal at top speed.
fn seek(body: &Body, profile: &Profile, goal: &Goal) -> Wanted {
    let top = profile.max_speed();
    let to_target = goal.target - body.position;
    if !top.is_finite() || to_target.length_squared() < TINY_SQ {
        return Wanted::NONE;
    }
    let direction = to_target.normalize();
    Wanted {
        velocity: direction * top,
        direction,
    }
}

/// Where a pursuer meets the target: `stand_off` from the target's hull, on the line to the agent
/// (behind the target if they coincide).
fn stand_off_point(body: &Body, target: &Body, stand_off: f64) -> DVec3 {
    let outward = unit_or(body.position - target.position, -target.frame.forward);
    target.position + outward * (target.radius + body.radius + stand_off)
}

/// The slot `offset` away from the leader along its own axes.
fn slot_point(leader: &Body, offset: &FrameOffset) -> DVec3 {
    leader.position
        + leader.frame.right * offset.right
        + leader.frame.up * offset.up
        + leader.frame.forward * offset.forward
}

/// The speed at which `ORBIT_THRUST_SHARE` of the forward thrust holds a circle of `radius`
/// against drag and the centripetal demand: the positive root of `s⁴/r² + k²·s² = (share·a)²`.
/// 0 for a circle without radius or an agent without thrust.
fn circle_speed(profile: &Profile, radius: f64) -> f64 {
    let thrust = ORBIT_THRUST_SHARE * profile.accel_forward;
    if radius <= 0.0 || thrust <= 0.0 {
        return 0.0;
    }
    let drag_sq = profile.linear_drag * profile.linear_drag;
    let speed_sq = 0.5
        * radius
        * radius
        * (-drag_sq + (drag_sq * drag_sq + 4.0 * thrust * thrust / (radius * radius)).sqrt());
    speed_sq.sqrt().min(profile.max_speed())
}

/// The geometry of an agent relative to an orbit: the unit normal, the height above the plane,
/// and the unit direction from the axis to the agent within the plane.
struct OrbitFrame {
    normal: DVec3,
    height: f64,
    radial: DVec3,
    /// Distance from the axis.
    ring: f64,
}

impl OrbitFrame {
    fn new(body: &Body, centre: &Body, normal: DVec3) -> Self {
        let normal = normal.normalize();
        let offset = body.position - centre.position;
        let height = offset.dot(normal);
        let in_plane = offset - normal * height;
        let ring = in_plane.length();
        // On the axis any direction across it will do.
        let radial = unit_or(in_plane, normal.any_orthonormal_vector());
        Self {
            normal,
            height,
            radial,
            ring,
        }
    }
}

/// Circle the centre: the in-plane field is the circle's tangent, tilted toward the circle by the
/// normalised radial error; out of plane the agent descends to the plane on the drag rule. The
/// total speed is the circle speed, moving with the centre.
fn orbit(
    body: &Body,
    profile: &Profile,
    centre: &Body,
    radius: f64,
    normal: DVec3,
) -> (Wanted, Goal) {
    let arrival = |target: DVec3| Goal {
        target,
        arrival_radius: 0.0,
    };
    if normal.length_squared() < TINY_SQ || radius <= 0.0 {
        return (
            wanted_of(centre.velocity, DVec3::ZERO),
            arrival(centre.position),
        );
    }
    let frame = OrbitFrame::new(body, centre, normal);
    let speed = circle_speed(profile, radius);
    let tangent = frame.normal.cross(frame.radial);
    let error = (frame.ring * frame.ring - radius * radius) / (radius * radius);
    let in_plane = (tangent - frame.radial * (ORBIT_FIELD_GAIN * error)).normalize();
    let climb = (-frame.height * profile.linear_drag).clamp(-speed, speed);
    let around = (speed * speed - climb * climb).sqrt();
    let mut velocity = in_plane * around + frame.normal * climb;
    let top = profile.max_speed();
    if velocity.length_squared() > top * top {
        velocity *= top / velocity.length();
    }
    (
        wanted_of(velocity + centre.velocity, in_plane),
        arrival(centre.position + frame.radial * radius),
    )
}

/// What `behaviour` wants of the agent now, and the point it wants the agent at.
pub(crate) fn steer(behaviour: &Behaviour, body: &Body, profile: &Profile) -> (Wanted, Goal) {
    match *behaviour {
        Behaviour::Seek(goal) => (seek(body, profile, &goal), goal),
        Behaviour::Arrive(goal) => (arrive(body, profile, &goal), goal),
        Behaviour::Pursue {
            target,
            stand_off,
            arrival_radius,
        } => {
            let goal = Goal {
                target: stand_off_point(body, &target, stand_off),
                arrival_radius,
            };
            (arrive_at(body, profile, goal.target, target.velocity), goal)
        }
        Behaviour::Follow {
            leader,
            offset,
            arrival_radius,
        } => {
            let goal = Goal {
                target: slot_point(&leader, &offset),
                arrival_radius,
            };
            (arrive_at(body, profile, goal.target, leader.velocity), goal)
        }
        Behaviour::Orbit {
            centre,
            radius,
            normal,
            arrival_radius,
        } => {
            let (wanted, goal) = orbit(body, profile, &centre, radius, normal);
            (
                wanted,
                Goal {
                    arrival_radius,
                    ..goal
                },
            )
        }
    }
}

/// The unit direction the nose is to hold while `body` is on station, or `None` when the
/// behaviour has no such direction or `body` is still outside its goal's arrival radius (then
/// the nose follows the course). A follower faces where its leader faces, a pursuer where a
/// moving target heads, and an agent that holds a point or pursues a resting target keeps its
/// own heading. `goal` is the one `steer` returned for `behaviour`.
pub(crate) fn rest_facing(behaviour: &Behaviour, body: &Body, goal: &Goal) -> Option<DVec3> {
    let on_station =
        body.position.distance(goal.target) <= goal.effective_arrival_radius(body.radius);
    if !on_station {
        return None;
    }
    match *behaviour {
        Behaviour::Arrive(_) => Some(body.frame.forward),
        Behaviour::Follow { leader, .. } => Some(leader.frame.forward),
        Behaviour::Pursue { target, .. } => {
            let moves = target.velocity.length() > target.radius * AT_REST_RADII_PER_S;
            Some(if moves {
                target.velocity.normalize()
            } else {
                body.frame.forward
            })
        }
        Behaviour::Seek(_) | Behaviour::Orbit { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Frame;

    fn body_at(position: DVec3) -> Body {
        Body {
            position,
            radius: 1.0,
            ..Body::AT_REST
        }
    }

    const PROFILE: Profile = Profile::TEST;

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
    fn near_goal_gets_the_speed_the_agent_can_brake_from() {
        let distance = 10.0;
        let wanted = arrive(
            &body_at(DVec3::ZERO),
            &PROFILE,
            &goal_at(DVec3::new(distance, 0.0, 0.0)),
        );
        assert_eq!(wanted.velocity.x, stopping_speed(&PROFILE, distance));
        assert!(wanted.velocity.x > distance * PROFILE.linear_drag);
        assert_eq!(wanted.direction, DVec3::X);
    }

    #[test]
    fn goal_under_the_agent_wants_nothing() {
        let wanted = arrive(&body_at(DVec3::ONE), &PROFILE, &goal_at(DVec3::ONE));
        assert_eq!(wanted, Wanted::NONE);
    }

    #[test]
    fn seek_keeps_top_speed_up_to_the_point() {
        let near = goal_at(DVec3::new(0.0, 0.0, -0.5));
        let wanted = seek(&body_at(DVec3::ZERO), &PROFILE, &near);
        assert_eq!(wanted.velocity, DVec3::new(0.0, 0.0, -100.0));
        assert_eq!(wanted.direction, DVec3::NEG_Z);
    }

    #[test]
    fn seek_without_a_top_speed_wants_nothing() {
        let no_drag = Profile {
            linear_drag: 0.0,
            ..PROFILE
        };
        let wanted = seek(
            &body_at(DVec3::ZERO),
            &no_drag,
            &goal_at(DVec3::new(0.0, 0.0, -100.0)),
        );
        assert_eq!(wanted, Wanted::NONE);
    }

    #[test]
    fn a_moving_point_far_away_is_chased_at_top_speed_with_its_lateral_velocity() {
        // The point runs across the line of sight at 30; the lead keeps the bearing constant.
        let velocity = DVec3::new(30.0, 0.0, 0.0);
        let wanted = arrive_at(
            &body_at(DVec3::ZERO),
            &PROFILE,
            DVec3::new(0.0, 0.0, -5000.0),
            velocity,
        );
        assert!((wanted.velocity.length() - 100.0).abs() < 1e-9);
        assert!((wanted.velocity.x - 30.0).abs() < 1e-9);
        assert!(wanted.velocity.z < 0.0);
        // The nose points where the agent goes, ahead of the point.
        assert!(wanted.direction.x > 0.0);
    }

    #[test]
    fn a_moving_point_that_is_close_is_matched_in_velocity() {
        let velocity = DVec3::new(30.0, 0.0, 0.0);
        let wanted = arrive_at(
            &body_at(DVec3::ZERO),
            &PROFILE,
            DVec3::new(0.0, 0.0, -1e-6),
            velocity,
        );
        assert!((wanted.velocity - velocity).length() < 1e-4);
        let on_it = arrive_at(&body_at(DVec3::ONE), &PROFILE, DVec3::ONE, velocity);
        assert_eq!(on_it.velocity, velocity);
        assert_eq!(on_it.direction, DVec3::X);
    }

    #[test]
    fn a_point_that_outruns_the_agent_is_chased_along_the_line_of_sight() {
        let wanted = arrive_at(
            &body_at(DVec3::ZERO),
            &PROFILE,
            DVec3::new(0.0, 0.0, -5000.0),
            DVec3::new(0.0, 0.0, -150.0),
        );
        assert_eq!(wanted.velocity, DVec3::new(0.0, 0.0, -100.0));
    }

    #[test]
    fn a_resting_point_is_the_same_as_arrive() {
        let body = body_at(DVec3::new(3.0, 4.0, 5.0));
        let point = DVec3::new(-60.0, 20.0, 7.0);
        assert_eq!(
            arrive_at(&body, &PROFILE, point, DVec3::ZERO),
            arrive(&body, &PROFILE, &goal_at(point))
        );
    }

    #[test]
    fn a_pursuer_aims_at_the_stand_off_point_on_the_line_to_itself() {
        let target = Body {
            position: DVec3::new(0.0, 0.0, -100.0),
            velocity: DVec3::new(5.0, 0.0, 0.0),
            radius: 4.0,
            ..Body::AT_REST
        };
        let behaviour = Behaviour::Pursue {
            target,
            stand_off: 10.0,
            arrival_radius: 1.0,
        };
        let (wanted, goal) = steer(&behaviour, &body_at(DVec3::ZERO), &PROFILE);
        // Target radius 4 + agent radius 1 + stand-off 10, back along the line to the agent.
        assert!((goal.target - DVec3::new(0.0, 0.0, -85.0)).length() < 1e-9);
        assert_eq!(goal.arrival_radius, 1.0);
        assert!((wanted.velocity.x - 5.0).abs() < 1e-9);
    }

    #[test]
    fn a_pursuer_on_top_of_its_target_backs_off_behind_it() {
        let target = Body {
            position: DVec3::new(1.0, 2.0, 3.0),
            ..body_at(DVec3::ZERO)
        };
        let behaviour = Behaviour::Pursue {
            target,
            stand_off: 9.0,
            arrival_radius: 1.0,
        };
        let (_, goal) = steer(&behaviour, &body_at(target.position), &PROFILE);
        // Behind the target is +z for a target facing -z.
        assert!((goal.target - (target.position + DVec3::Z * 11.0)).length() < 1e-9);
    }

    #[test]
    fn a_follower_aims_at_its_slot_in_the_leaders_frame() {
        // A leader that faces +x, with +y up, so its right is +z.
        let leader = Body {
            position: DVec3::new(10.0, 0.0, 0.0),
            velocity: DVec3::new(0.0, 0.0, 7.0),
            frame: Frame {
                forward: DVec3::X,
                right: DVec3::Z,
                up: DVec3::Y,
            },
            ..body_at(DVec3::ZERO)
        };
        let behaviour = Behaviour::Follow {
            leader,
            offset: FrameOffset {
                right: 2.0,
                up: 3.0,
                forward: -5.0,
            },
            arrival_radius: 1.0,
        };
        let (wanted, goal) = steer(&behaviour, &body_at(DVec3::new(0.0, 0.0, 100.0)), &PROFILE);
        assert!((goal.target - DVec3::new(5.0, 3.0, 2.0)).length() < 1e-9);
        assert!(wanted.velocity.length() <= 100.0 + 1e-9);
    }

    /// A frame that faces +x with +y up, so that it differs from the body's own (facing -z).
    const FACING_X: Frame = Frame {
        forward: DVec3::X,
        right: DVec3::Z,
        up: DVec3::Y,
    };

    /// The facing of `behaviour` for an agent of radius 1 that faces -z at `position`.
    fn facing_at(behaviour: &Behaviour, position: DVec3) -> Option<DVec3> {
        let body = body_at(position);
        let (_, goal) = steer(behaviour, &body, &PROFILE);
        rest_facing(behaviour, &body, &goal)
    }

    fn follow(leader: Body) -> Behaviour {
        Behaviour::Follow {
            leader,
            offset: FrameOffset {
                right: 0.0,
                up: 0.0,
                forward: 0.0,
            },
            arrival_radius: 5.0,
        }
    }

    #[test]
    fn a_follower_on_station_faces_where_its_leader_faces() {
        let leader = Body {
            frame: FACING_X,
            ..body_at(DVec3::ZERO)
        };
        let on_station = facing_at(&follow(leader), DVec3::new(0.0, 0.0, 3.0));
        assert_eq!(on_station, Some(DVec3::X));
        let far = facing_at(&follow(leader), DVec3::new(0.0, 0.0, 30.0));
        assert_eq!(
            far, None,
            "outside the arrival radius the nose follows the course"
        );
    }

    #[test]
    fn an_agent_that_holds_a_point_keeps_its_heading() {
        let point = Behaviour::Arrive(goal_at(DVec3::new(0.0, 0.0, 10.0)));
        assert_eq!(
            facing_at(&point, DVec3::new(0.0, 0.0, 10.5)),
            Some(DVec3::NEG_Z)
        );
        assert_eq!(facing_at(&point, DVec3::ZERO), None);
    }

    #[test]
    fn a_pursuer_on_station_faces_a_moving_targets_heading_and_keeps_its_own_for_a_resting_one() {
        let pursue = |velocity: DVec3| Behaviour::Pursue {
            target: Body {
                velocity,
                ..body_at(DVec3::ZERO)
            },
            stand_off: 4.0,
            arrival_radius: 5.0,
        };
        // On the stand-off point: target radius 1 + agent radius 1 + 4, along the line to the agent.
        let on_station = DVec3::new(0.0, 0.0, 6.0);
        let moving = facing_at(&pursue(DVec3::new(0.0, 7.0, 0.0)), on_station);
        assert_eq!(moving, Some(DVec3::Y));
        let resting = facing_at(&pursue(DVec3::new(0.0, 0.5, 0.0)), on_station);
        assert_eq!(resting, Some(DVec3::NEG_Z));
    }

    #[test]
    fn seek_and_orbit_have_no_facing() {
        let seek = Behaviour::Seek(goal_at(DVec3::new(0.0, 0.0, 10.0)));
        assert_eq!(facing_at(&seek, DVec3::new(0.0, 0.0, 10.0)), None);
        let orbit = Behaviour::Orbit {
            centre: body_at(DVec3::ZERO),
            radius: ORBIT_RADIUS,
            normal: DVec3::Y,
            arrival_radius: 7.0,
        };
        assert_eq!(facing_at(&orbit, DVec3::new(ORBIT_RADIUS, 0.0, 0.0)), None);
    }

    #[test]
    fn hold_is_arrive_at_the_bodys_position() {
        let body = body_at(DVec3::new(1.0, 2.0, 3.0));
        assert_eq!(
            Behaviour::hold(&body, 4.0),
            Behaviour::Arrive(Goal {
                target: body.position,
                arrival_radius: 4.0
            })
        );
    }

    const ORBIT_RADIUS: f64 = 1000.0;

    fn orbit_wanted(at: DVec3, centre: &Body, normal: DVec3) -> (Wanted, Goal) {
        let behaviour = Behaviour::Orbit {
            centre: *centre,
            radius: ORBIT_RADIUS,
            normal,
            arrival_radius: 7.0,
        };
        steer(&behaviour, &body_at(at), &PROFILE)
    }

    #[test]
    fn the_circle_speed_solves_its_balance() {
        let speed = circle_speed(&PROFILE, ORBIT_RADIUS);
        let thrust = ORBIT_THRUST_SHARE * PROFILE.accel_forward;
        let drag = PROFILE.linear_drag;
        let lhs = speed.powi(4) / (ORBIT_RADIUS * ORBIT_RADIUS) + drag * drag * speed * speed;
        assert!((lhs - thrust * thrust).abs() < 1e-6 * thrust * thrust);
        assert!(speed <= PROFILE.max_speed());
        // Without drag, the centripetal acceleration is the thrust: s² / r = thrust.
        let no_drag = Profile {
            linear_drag: 0.0,
            ..PROFILE
        };
        let expected = (thrust * ORBIT_RADIUS).sqrt();
        assert!((circle_speed(&no_drag, ORBIT_RADIUS) - expected).abs() < 1e-9);
        assert_eq!(circle_speed(&PROFILE, 0.0), 0.0);
    }

    #[test]
    fn on_the_circle_the_orbit_goes_around_anticlockwise_about_the_normal() {
        let centre = body_at(DVec3::ZERO);
        let at = DVec3::new(ORBIT_RADIUS, 0.0, 0.0);
        let (wanted, goal) = orbit_wanted(at, &centre, DVec3::Y);
        // Anticlockwise about +y from +x heads toward -z.
        assert!(wanted.velocity.z < 0.0);
        assert!(wanted.velocity.x.abs() < 1e-9 && wanted.velocity.y.abs() < 1e-9);
        assert!((wanted.velocity.length() - circle_speed(&PROFILE, ORBIT_RADIUS)).abs() < 1e-9);
        assert!((goal.target - at).length() < 1e-9);
        assert_eq!(goal.arrival_radius, 7.0);
        let (reversed, _) = orbit_wanted(at, &centre, DVec3::NEG_Y);
        assert!((reversed.velocity + wanted.velocity).length() < 1e-9);
    }

    #[test]
    fn the_orbit_field_turns_toward_the_circle_from_outside_and_inside() {
        let centre = body_at(DVec3::ZERO);
        let (outside, _) =
            orbit_wanted(DVec3::new(2.0 * ORBIT_RADIUS, 0.0, 0.0), &centre, DVec3::Y);
        assert!(outside.velocity.x < 0.0, "inward from outside");
        assert!(outside.velocity.z < 0.0, "still going around");
        let (inside, _) = orbit_wanted(DVec3::new(0.5 * ORBIT_RADIUS, 0.0, 0.0), &centre, DVec3::Y);
        assert!(inside.velocity.x > 0.0, "outward from inside");
        assert!(inside.velocity.z < 0.0, "still going around");
    }

    #[test]
    fn the_orbit_descends_to_its_plane_from_above_and_below() {
        let centre = body_at(DVec3::ZERO);
        let (above, goal) = orbit_wanted(DVec3::new(ORBIT_RADIUS, 30.0, 0.0), &centre, DVec3::Y);
        assert!(above.velocity.y < 0.0);
        assert!((goal.target - DVec3::new(ORBIT_RADIUS, 0.0, 0.0)).length() < 1e-9);
        let (below, _) = orbit_wanted(DVec3::new(ORBIT_RADIUS, -30.0, 0.0), &centre, DVec3::Y);
        assert!(below.velocity.y > 0.0);
        assert!(above.velocity.length() <= PROFILE.max_speed() + 1e-9);
    }

    #[test]
    fn the_orbit_moves_with_its_centre() {
        let moving = Body {
            velocity: DVec3::new(0.0, 0.0, 10.0),
            ..body_at(DVec3::ZERO)
        };
        let at = DVec3::new(ORBIT_RADIUS, 0.0, 0.0);
        let (still, _) = orbit_wanted(at, &body_at(DVec3::ZERO), DVec3::Y);
        let (carried, _) = orbit_wanted(at, &moving, DVec3::Y);
        assert!((carried.velocity - still.velocity - moving.velocity).length() < 1e-9);
    }

    #[test]
    fn an_agent_on_the_axis_picks_a_direction_out_instead_of_failing() {
        let centre = body_at(DVec3::ZERO);
        let (wanted, _) = orbit_wanted(DVec3::ZERO, &centre, DVec3::Y);
        assert!(wanted.velocity.is_finite() && wanted.velocity.length() > 0.0);
    }

    #[test]
    fn a_zero_normal_leaves_the_agent_with_the_centres_velocity() {
        let centre = Body {
            velocity: DVec3::new(1.0, 2.0, 3.0),
            ..body_at(DVec3::ZERO)
        };
        let (wanted, _) = orbit_wanted(DVec3::new(50.0, 0.0, 0.0), &centre, DVec3::ZERO);
        assert_eq!(wanted.velocity, centre.velocity);
    }
}
