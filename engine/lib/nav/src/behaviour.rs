use glam::DVec3;

use crate::agent::{AT_REST_RADII_PER_S, Body, Goal, Profile, Speed};
use crate::braking::{stopping_distance, stopping_speed};

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
    /// Fly through the goal's point at top speed, braking only as far as `pass_speed`, the
    /// largest speed the agent is to have at the point (relative to its frame); infinite for no
    /// braking. An agent with no top speed (no linear drag and no speed cap) gets no input.
    Seek { goal: Goal, pass_speed: Speed },
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
        Self::Arrive(Goal::fixed(body.position, arrival_radius))
    }

    /// The behaviour as seen from a frame that moves at `frame_velocity`: every velocity in it
    /// (a goal's, and those of the bodies it names) has that velocity taken off.
    pub(crate) fn seen_from(self, frame_velocity: DVec3) -> Self {
        match self {
            Self::Seek { goal, pass_speed } => Self::Seek {
                goal: goal.seen_from(frame_velocity),
                pass_speed,
            },
            Self::Arrive(goal) => Self::Arrive(goal.seen_from(frame_velocity)),
            Self::Pursue {
                target,
                stand_off,
                arrival_radius,
            } => Self::Pursue {
                target: target.seen_from(frame_velocity),
                stand_off,
                arrival_radius,
            },
            Self::Follow {
                leader,
                offset,
                arrival_radius,
            } => Self::Follow {
                leader: leader.seen_from(frame_velocity),
                offset,
                arrival_radius,
            },
            Self::Orbit {
                centre,
                radius,
                normal,
                arrival_radius,
            } => Self::Orbit {
                centre: centre.seen_from(frame_velocity),
                radius,
                normal,
                arrival_radius,
            },
        }
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
    arrive_at(body, profile, goal.target, goal.velocity)
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

/// The speed a Seek flies at, `distance` from where it is to pass: top speed, lowered by the
/// braking curve that ends at `pass_speed` there. That is Arrive's curve shifted by the distance
/// braking from `pass_speed` covers, so it reaches `pass_speed` where Arrive's would reach rest.
/// Never below `pass_speed`: near the end the curve's tail cap (a loop of finite gain can't
/// follow its slope at rest) would otherwise put it under.
fn seek_speed(profile: &Profile, distance: f64, pass_speed: Speed, top: Speed) -> Speed {
    if pass_speed >= top {
        return top;
    }
    let braking = stopping_speed(profile, distance + stopping_distance(profile, pass_speed));
    braking.max(pass_speed).min(top)
}

/// Fly through the goal at top speed, slowing to `pass_speed` on entering its arrival radius,
/// which is where the point counts as passed.
fn seek(body: &Body, profile: &Profile, goal: &Goal, pass_speed: Speed) -> Wanted {
    let top = profile.max_speed();
    let to_target = goal.target - body.position;
    if !top.is_finite() || to_target.length_squared() < TINY_SQ {
        return Wanted::NONE;
    }
    let distance = to_target.length();
    let direction = to_target / distance;
    let to_pass = (distance - goal.effective_arrival_radius(body.radius)).max(0.0);
    Wanted {
        velocity: direction * seek_speed(profile, to_pass, pass_speed, top),
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

/// The speed at which `share` (0 to 1) of the forward thrust holds a circle of `radius` against
/// drag and the centripetal demand: the positive root of `s⁴/r² + k²·s² = (share·a)²`, capped at
/// the top speed. 0 for a circle without radius or an agent without thrust.
pub fn circle_speed(profile: &Profile, radius: f64, share: f64) -> f64 {
    let thrust = share * profile.accel_forward;
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
        velocity: centre.velocity,
        arrival_radius: 0.0,
    };
    if normal.length_squared() < TINY_SQ || radius <= 0.0 {
        return (
            wanted_of(centre.velocity, DVec3::ZERO),
            arrival(centre.position),
        );
    }
    let frame = OrbitFrame::new(body, centre, normal);
    let speed = circle_speed(profile, radius, ORBIT_THRUST_SHARE);
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
        Behaviour::Seek { goal, pass_speed } => (seek(body, profile, &goal, pass_speed), goal),
        Behaviour::Arrive(goal) => (arrive(body, profile, &goal), goal),
        Behaviour::Pursue {
            target,
            stand_off,
            arrival_radius,
        } => {
            let goal = Goal {
                target: stand_off_point(body, &target, stand_off),
                velocity: target.velocity,
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
                velocity: leader.velocity,
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
        Behaviour::Seek { .. } | Behaviour::Orbit { .. } => None,
    }
}

/// How far from the body a facing point must be, in the body's radii, for the direction to it to
/// mean something: nearer, the direction swings with every small move of either.
const FACING_MIN_RADII: f64 = 1.0;

/// The unit direction from `body` to `point`, or `None` when the point is within
/// `FACING_MIN_RADII` of the body's centre (or on it).
pub(crate) fn facing_toward(body: &Body, point: DVec3) -> Option<DVec3> {
    let offset = point - body.position;
    let reach = body.radius * FACING_MIN_RADII;
    if offset.length_squared() < TINY_SQ || offset.length_squared() <= reach * reach {
        return None;
    }
    Some(offset.normalize())
}

#[cfg(test)]
mod tests;
