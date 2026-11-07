use glam::DVec3;

use crate::agent::{AgentId, Body, Goal, Profile};
use crate::behaviour::{TINY_SQ, Wanted};
use crate::braking::{stopping_distance, stopping_speed};
use crate::obstacle::{Obstacle, ObstacleKey};

/// How far ahead and how widely an agent avoids obstacles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvoidanceTuning {
    /// Obstacles the agent would reach later than this are ignored, in seconds.
    pub horizon_s: f64,
    /// The largest distance kept from an obstacle's surface besides the agent's own radius, in
    /// agent radii. The distance kept grows with the speed the agent closes on the obstacle at,
    /// from [`MIN_CLEARANCE_RADII`] up to this.
    pub clearance_radii: f64,
    /// The new course passes the obstacle at this multiple of the clearance kept from it. A
    /// value above 1 keeps the course off the boundary the threat test uses, so it doesn't flip
    /// every step.
    pub aim_factor: f64,
}

impl Default for AvoidanceTuning {
    fn default() -> Self {
        Self {
            horizon_s: 10.0,
            clearance_radii: 2.0,
            aim_factor: 1.05,
        }
    }
}

/// How many spheres one step may take the tangent courses of. It bounds the work to a few
/// courses, each tested against every obstacle, instead of growing with the obstacle count.
const MAX_EXPANDED_THREATS: usize = 4;

/// The clearance kept from an obstacle the agent isn't closing on, in agent radii. It is capped
/// by [`AvoidanceTuning::clearance_radii`] when that is smaller.
pub const MIN_CLEARANCE_RADII: f64 = 0.5;

/// The time an agent takes to turn onto a new course, in seconds. The closing-speed limit stops
/// an agent this long at its current speed short of a hull, so a fast agent whose course flips
/// near a sphere still has room to turn. `Profile` has no turn rate to derive it from.
pub const BRAKING_REACTION_S: f64 = 0.3;

/// The obstacle an agent reaches first on a course, in the obstacle's frame.
#[derive(Clone, Copy)]
struct Threat {
    /// Seconds until the agent touches the inflated sphere; 0 when it is already inside.
    entry_time: f64,
    /// Agent position minus obstacle position.
    offset: DVec3,
    /// Wanted velocity minus obstacle velocity.
    relative_velocity: DVec3,
    /// Obstacle radius plus the agent's radius and clearance.
    reach: f64,
    /// Obstacle radius plus the agent's radius and the aim factor times the clearance: the
    /// radius a detour passes at.
    aim_reach: f64,
    obstacle_velocity: DVec3,
}

impl Threat {
    fn is_inside(&self) -> bool {
        self.entry_time == 0.0 && self.offset.length_squared() <= self.reach * self.reach
    }
}

/// The relative slack on the distance a course can cover within the horizon when deciding whether
/// a sphere can be reached at all. It absorbs the rounding of the quadratic in
/// [`Sphere::threat`], so a sphere is only skipped when no course could reach it.
const REACHABLE_SLACK: f64 = 1.0 + 1.0e-6;

/// What the course tests of one agent need of an obstacle, computed once per step from the state
/// at the start of the step.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Sphere {
    key: ObstacleKey,
    /// Agent position minus obstacle position.
    offset: DVec3,
    /// The length of `offset`.
    distance: f64,
    /// The velocity the courses are tested against: the obstacle's own, or the mean of the two
    /// agents' for a sphere that avoids back (see [`Peers`]).
    velocity: DVec3,
    /// The share of the avoidance this agent takes: 1, or [`RECIPROCAL_SHARE`] against an agent
    /// that avoids back.
    share: f64,
    /// Obstacle radius plus the agent's radius.
    hull: f64,
    /// The hull plus the clearance kept.
    reach: f64,
    /// The hull plus the aim factor times the clearance.
    aim_reach: f64,
    /// The squared distance minus the squared reach: 0 or less when the agent is within the reach.
    reach_gap_sq: f64,
    /// Whether any course no faster than the speed bound the sphere was made for can come within
    /// the reach inside the horizon. A sphere that can't be reached is never a threat to such a
    /// course.
    reachable: bool,
}

impl Sphere {
    /// The time the agent, flying `wanted_velocity`, first comes within the reach; `None` when it
    /// doesn't within `horizon_s`.
    fn threat(&self, wanted_velocity: DVec3, horizon_s: f64) -> Option<Threat> {
        let relative_velocity = wanted_velocity - self.velocity;
        let make = |entry_time| Threat {
            entry_time,
            offset: self.offset,
            relative_velocity,
            reach: self.reach,
            aim_reach: self.aim_reach,
            obstacle_velocity: self.velocity,
        };

        // Solve |offset + relative_velocity·t| = reach for the first t ≥ 0: a·t² + 2b·t + c = 0.
        let c = self.reach_gap_sq;
        if c <= 0.0 {
            return Some(make(0.0));
        }
        let a = relative_velocity.length_squared();
        let b = self.offset.dot(relative_velocity);
        if b >= 0.0 || a < TINY_SQ {
            return None;
        }
        let discriminant = b * b - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let entry_time = (-b - discriminant.sqrt()) / a;
        (entry_time <= horizon_s).then(|| make(entry_time))
    }
}

/// The share of the avoidance an agent takes against an agent that avoids back: half, so that
/// the pair together takes all of it (the reciprocal velocity obstacle).
const RECIPROCAL_SHARE: f64 = 0.5;

/// What one agent of a world knows of the agents of that world: whether each steers, and what
/// each tracks.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Peer {
    /// Whether the agent has a behaviour, so it steers around others.
    pub steering: bool,
    /// The obstacle the agent's behaviour tracks, which it leaves out of its own scene.
    pub tracks: Option<ObstacleKey>,
}

/// The agents around the agent being steered, for [`avoid`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct Peers<'a> {
    /// The agent being steered.
    pub own: AgentId,
    /// The obstacle its behaviour tracks; it is left out of the scene.
    pub tracked: Option<ObstacleKey>,
    /// Every agent of the world, in agent order.
    pub all: &'a [Peer],
}

impl Peers<'_> {
    /// An agent with no neighbours that tracks nothing.
    #[cfg(test)]
    pub fn alone() -> Peers<'static> {
        Peers {
            own: AgentId::new(0),
            tracked: None,
            all: &[],
        }
    }

    /// The share of the avoidance of `key` the agent takes: [`RECIPROCAL_SHARE`] when `key` is a
    /// steering agent that doesn't track this one (it avoids this agent back), 1 otherwise.
    fn share_of(&self, key: ObstacleKey) -> f64 {
        let ObstacleKey::Agent(id) = key else {
            return 1.0;
        };
        let avoids_back = self
            .all
            .get(id.index())
            .is_some_and(|peer| peer.steering && peer.tracks != Some(ObstacleKey::Agent(self.own)));
        if avoids_back { RECIPROCAL_SHARE } else { 1.0 }
    }
}

/// The agent and the spheres its courses are tested against.
struct Scene<'a> {
    body: &'a Body,
    profile: &'a Profile,
    tuning: &'a AvoidanceTuning,
    /// The agent's goal, if any: no sphere keeps a clearance that would put it out of reach.
    goal: Option<Goal>,
    peers: Peers<'a>,
    /// One per obstacle but the one the agent tracks, in the order of the obstacles.
    spheres: &'a [Sphere],
}

impl Scene<'_> {
    /// Replaces `out` with the spheres of `obstacles` but the tracked one, for courses no faster
    /// than `speed_bound`.
    fn fill_spheres(&self, obstacles: &[Obstacle], speed_bound: f64, out: &mut Vec<Sphere>) {
        out.clear();
        out.extend(
            obstacles
                .iter()
                .filter(|obstacle| Some(obstacle.key) != self.peers.tracked)
                .map(|obstacle| self.sphere(obstacle, speed_bound)),
        );
    }

    fn sphere(&self, obstacle: &Obstacle, speed_bound: f64) -> Sphere {
        let clearance = self.clearance(obstacle);
        let hull = obstacle.radius + self.body.radius;
        let offset = self.body.position - obstacle.position;
        let reach = hull + clearance;
        let reach_gap_sq = offset.length_squared() - reach * reach;
        let distance = offset.length();
        // An agent that avoids back is tested as moving at the mean of the two velocities, the
        // apex of the reciprocal cone; each takes the half of the detour that its share asks.
        let share = self.peers.share_of(obstacle.key);
        let velocity = if share < 1.0 {
            self.body.velocity.lerp(obstacle.velocity, share)
        } else {
            obstacle.velocity
        };
        // Relative to the sphere an agent closes in at its speed plus the sphere's.
        let travel = course_travel(speed_bound + velocity.length(), self.tuning);
        Sphere {
            key: obstacle.key,
            offset,
            distance,
            velocity,
            share,
            hull,
            reach,
            aim_reach: hull + self.tuning.aim_factor * clearance,
            reach_gap_sq,
            reachable: reach_gap_sq <= 0.0 || distance - reach <= travel * REACHABLE_SLACK,
        }
    }

    /// The sphere (its index and threat) the agent reaches first flying `velocity`, which must be
    /// no faster than the speed bound the spheres were made for.
    fn first_threat(&self, velocity: DVec3) -> Option<(usize, Threat)> {
        self.spheres
            .iter()
            .enumerate()
            .filter(|(_, sphere)| sphere.reachable)
            .filter_map(|(index, sphere)| {
                sphere
                    .threat(velocity, self.tuning.horizon_s)
                    .map(|threat| (index, threat))
            })
            .min_by(|a, b| a.1.entry_time.total_cmp(&b.1.entry_time))
    }

    /// The distance the agent keeps from the obstacle's hull besides its own radius: the
    /// distance it takes to brake from its current closing speed ([`stopping_distance`]), between
    /// [`MIN_CLEARANCE_RADII`] and `clearance_radii` agent radii. It depends on the agent's
    /// state at the start of the step only, never on the course being tested. It never exceeds
    /// what leaves the goal's arrival circle outside the sphere's reach, down to 0, so a goal
    /// next to a sphere can still be approached.
    fn clearance(&self, obstacle: &Obstacle) -> f64 {
        let largest = self.body.radius * self.tuning.clearance_radii;
        let smallest = self.smallest_clearance();
        let closing = closing_speed(self.body, obstacle);
        let stopping = stopping_distance(self.profile, closing);
        let goal_room = self.goal.map_or(f64::INFINITY, |goal| {
            let hull = obstacle.radius + self.body.radius;
            goal.target.distance(obstacle.position)
                - hull
                - goal.effective_arrival_radius(self.body.radius)
        });
        stopping.max(smallest).min(largest).min(goal_room.max(0.0))
    }

    /// The clearance kept from an obstacle the agent isn't closing on.
    fn smallest_clearance(&self) -> f64 {
        self.body.radius * MIN_CLEARANCE_RADII.min(self.tuning.clearance_radii)
    }

    /// The distance short of a hull that the closing-speed limit stops the agent at: the
    /// distance it covers in [`BRAKING_REACTION_S`] at its current speed, at least the smallest
    /// clearance.
    fn braking_margin(&self) -> f64 {
        self.smallest_clearance()
            .max(self.body.velocity.length() * BRAKING_REACTION_S)
    }
}

/// The distance a course at `speed` covers within the avoidance horizon, in length units.
fn course_travel(speed: f64, tuning: &AvoidanceTuning) -> f64 {
    speed * tuning.horizon_s
}

/// The speed no course of the agent exceeds: its top speed, or its wanted or current speed if
/// larger. `wanted_speed` is 0 when no course has been chosen yet.
fn speed_bound(body: &Body, profile: &Profile, wanted_speed: f64) -> f64 {
    profile
        .max_speed()
        .max(wanted_speed)
        .max(body.velocity.length())
}

/// How far from an agent's position an obstacle's surface can lie and still matter to it: the
/// distance a course can cover within the horizon, plus the agent's radius and the largest
/// clearance kept. A sphere beyond it is never reachable, so the cull in [`avoid`] drops it; a
/// caller that looks for obstacles queries this far. The obstacle's own motion is not included.
pub fn sensing_range(body: &Body, profile: &Profile, tuning: &AvoidanceTuning) -> f64 {
    let largest_clearance = body.radius * tuning.clearance_radii;
    course_travel(speed_bound(body, profile, 0.0), tuning) + body.radius + largest_clearance
}

/// The speed the agent's body closes on the obstacle's centre at, from their velocities; 0 when
/// it moves away.
fn closing_speed(body: &Body, obstacle: &Obstacle) -> f64 {
    let offset = body.position - obstacle.position;
    let distance_sq = offset.length_squared();
    if distance_sq < TINY_SQ {
        return 0.0;
    }
    let relative_velocity = body.velocity - obstacle.velocity;
    (-relative_velocity.dot(offset) / distance_sq.sqrt()).max(0.0)
}

/// A sphere whose tangent courses are still to be tried, with the course that reaches it.
#[derive(Clone, Copy)]
struct Pending {
    index: usize,
    threat: Threat,
    course: Wanted,
    /// Whether the course reaches the sphere on the kept side of the first sphere, passing every
    /// sphere before it on the side it leans to.
    kept: bool,
}

/// Steers a wanted course clear of the obstacles, if it leads into one. The candidates are the
/// tangent courses of the sphere reached first, on the side the course leans to and on the
/// other; a candidate that another sphere blocks adds that sphere's tangents in turn. Of the
/// candidates that reach nothing within the horizon, the one turning least from the wanted
/// course is taken; if none is clear, the one that reaches an obstacle latest (the wanted course
/// included). The tangent is the one of Reynolds, "Steering Behaviors For Autonomous
/// Characters", 1999, obstacle avoidance.
///
/// The clearance kept from each sphere grows with the speed the agent closes on it at (see
/// [`AvoidanceTuning::clearance_radii`]), so a slow agent fits through gaps a fast one avoids.
///
/// The speed the resulting course closes at on every sphere that the wanted course, the new
/// course or the agent's own velocity leads into is also limited to the speed the agent can
/// brake from within the distance to the sphere's hull (its surface plus the agent's radius) less
/// a braking margin that grows with the agent's speed ([`BRAKING_REACTION_S`]): the curve of
/// [`arrive`](crate::behaviour::arrive). A fast agent turns slowly, so without the limit it
/// reaches the sphere before its velocity has followed the detour.
///
/// `commitment` is the agent's memory between steps: while the same sphere is the first one
/// reached, the side it passed it on is taken again if that course is clear, so the detour doesn't
/// swap sides as the agent's position or velocity shifts.
///
/// `peers` names the agents among `obstacles` that avoid this agent back, which it takes half of
/// the avoidance against, and the obstacle its behaviour tracks, which it leaves out.
///
/// `spheres` is scratch space for the per-obstacle data of this call, reused between calls so
/// that no call allocates. Spheres no course within the agent's top speed (or its wanted or
/// current speed, if larger) can come within reach of inside the horizon are left out of the
/// course search, which never changes its result.
#[expect(
    clippy::too_many_arguments,
    reason = "the agent's state, its goal, its memory, its neighbours and the scratch buffer are separate owners"
)]
pub(crate) fn avoid(
    body: &Body,
    profile: &Profile,
    wanted: Wanted,
    obstacles: &[Obstacle],
    tuning: &AvoidanceTuning,
    goal: Option<Goal>,
    peers: Peers,
    commitment: &mut Option<Commitment>,
    spheres: &mut Vec<Sphere>,
) -> Wanted {
    if wanted.direction == DVec3::ZERO {
        return wanted;
    }
    let max_speed = profile.max_speed();
    let probe = Scene {
        body,
        profile,
        tuning,
        goal,
        peers,
        spheres: &[],
    };
    let speed_bound = speed_bound(body, profile, wanted.velocity.length());
    probe.fill_spheres(obstacles, speed_bound, spheres);
    let scene = Scene {
        spheres: spheres.as_slice(),
        ..probe
    };
    let Some((index, first)) = scene.first_threat(wanted.velocity) else {
        return limit_closing_speeds(&scene, wanted, wanted);
    };
    let first_key = scene.spheres[index].key;
    let kept_side = commitment
        .filter(|kept| kept.obstacle == first_key)
        .map(|kept| kept.side);
    let course = if first.is_inside() {
        steer_around(&first, wanted, max_speed, false, kept_side)
    } else {
        search_courses(
            &scene,
            wanted,
            max_speed,
            Pending {
                index,
                threat: first,
                course: wanted,
                kept: kept_side.is_some(),
            },
            kept_side,
        )
    };
    *commitment = detour_side(&first, course).map(|side| Commitment {
        obstacle: first_key,
        side,
    });
    if first.is_inside() {
        return course;
    }
    limit_closing_speeds(&scene, wanted, course)
}

/// The side an agent passes a sphere on, remembered between steps.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Commitment {
    /// Key of the sphere, which outlives its place in the obstacle list.
    obstacle: ObstacleKey,
    /// Unit direction, across the line to the sphere's centre, the course leans to.
    side: DVec3,
}

/// The side `course` passes the threatening sphere on; `None` when it runs straight at the centre.
fn detour_side(threat: &Threat, course: Wanted) -> Option<DVec3> {
    let relative_velocity = course.velocity - threat.obstacle_velocity;
    let distance = threat.offset.length();
    if distance * distance < TINY_SQ {
        return None;
    }
    across(relative_velocity, -threat.offset / distance)
}

/// `course` with its closing speed limited on every sphere that `wanted`, `course` or the body's
/// velocity leads into.
fn limit_closing_speeds(scene: &Scene, wanted: Wanted, course: Wanted) -> Wanted {
    let margin = scene.braking_margin();
    let horizon_s = scene.tuning.horizon_s;
    scene.spheres.iter().fold(course, |course, sphere| {
        let leads_in = |velocity| sphere.threat(velocity, horizon_s).is_some();
        // The limited course can be faster than the speed bound, so it is always tested; the
        // wanted and the current velocity are within it, and can't reach an unreachable sphere.
        let is_threat = leads_in(course.velocity)
            || (sphere.reachable && (leads_in(wanted.velocity) || leads_in(scene.body.velocity)));
        if is_threat {
            limit_closing_speed(course, sphere, scene.profile, margin)
        } else {
            course
        }
    })
}

/// `course` with the speed it closes on the obstacle's sphere at reduced to the speed the agent
/// can brake from within its share of the distance to the sphere's hull less the braking margin.
/// The speed along the sphere is kept, so an agent passing the sphere isn't slowed for it.
fn limit_closing_speed(
    course: Wanted,
    sphere: &Sphere,
    profile: &Profile,
    braking_margin: f64,
) -> Wanted {
    let distance = sphere.distance;
    if distance * distance < TINY_SQ {
        return course;
    }
    let to_centre = -sphere.offset / distance;
    // Against an agent that brakes too, only the sphere's share of the room is this agent's.
    let room = (distance - sphere.hull - braking_margin).max(0.0) * sphere.share;
    let limit = stopping_speed(profile, room);
    let closing = (course.velocity - sphere.velocity).dot(to_centre);
    if closing <= limit {
        return course;
    }
    let velocity = course.velocity - to_centre * (closing - limit);
    let direction = if velocity.length_squared() < TINY_SQ {
        course.direction
    } else {
        velocity.normalize()
    };
    Wanted {
        velocity,
        direction,
    }
}

/// The best course found by expanding spheres breadth-first from `first`, which blocks `wanted`.
/// `kept_side` is the side the first sphere was passed on before: its course is preferred to the
/// other side's whenever it is clear. A course on that side blocked by a farther sphere is
/// expanded before a clear course on the other side is accepted, so the side doesn't flip for
/// an obstacle that can be passed.
fn search_courses(
    scene: &Scene,
    wanted: Wanted,
    max_speed: f64,
    first: Pending,
    kept_side: Option<DVec3>,
) -> Wanted {
    let mut queue = [None; MAX_EXPANDED_THREATS];
    queue[0] = Some(first);
    let mut queued = 1;
    let mut choice = Choice::new(wanted, first.threat.entry_time);
    let mut next = 0;
    while next < queued && !choice.is_settled(first.kept) {
        let Some(pending) = queue[next] else { break };
        let kept_side = kept_side.filter(|_| next == 0);
        next += 1;
        for flip in [false, true] {
            let course = steer_around(&pending.threat, pending.course, max_speed, flip, kept_side);
            let Some((index, threat)) = scene.first_threat(course.velocity) else {
                let kept = pending.kept && !flip;
                choice.offer_clear(course, course.direction.dot(wanted.direction), kept);
                continue;
            };
            choice.offer_blocked(course, threat.entry_time);
            let known = queue[..queued].iter().flatten().any(|p| p.index == index);
            if queued < MAX_EXPANDED_THREATS && !known {
                queue[queued] = Some(Pending {
                    index,
                    threat,
                    course,
                    kept: pending.kept && !flip,
                });
                queued += 1;
            }
        }
    }
    choice.best()
}

/// A candidate course with its score: higher is better.
#[derive(Clone, Copy)]
struct Scored {
    course: Wanted,
    score: f64,
    /// Whether the course passes on the side kept from the previous step.
    kept: bool,
}

/// The best clear candidate and the best blocked one so far. A later candidate replaces one
/// only with a strictly higher score, so ties keep the first offered.
struct Choice {
    clear: Option<Scored>,
    blocked: Scored,
}

impl Choice {
    /// Starts with the wanted course as the blocked candidate, reaching an obstacle at `entry_time`.
    fn new(wanted: Wanted, entry_time: f64) -> Self {
        Self {
            clear: None,
            blocked: Scored {
                course: wanted,
                score: entry_time,
                kept: false,
            },
        }
    }

    /// Offers a course that reaches nothing; `alignment` is the cosine of its turn from the
    /// wanted course, and `kept` whether it passes on the side kept from the previous step. A
    /// kept side beats any alignment.
    fn offer_clear(&mut self, course: Wanted, alignment: f64, kept: bool) {
        if self
            .clear
            .is_none_or(|best| (kept, alignment) > (best.kept, best.score))
        {
            self.clear = Some(Scored {
                course,
                score: alignment,
                kept,
            });
        }
    }

    /// Offers a course that reaches an obstacle after `entry_time` seconds.
    fn offer_blocked(&mut self, course: Wanted, entry_time: f64) {
        if entry_time > self.blocked.score {
            self.blocked = Scored {
                course,
                score: entry_time,
                kept: false,
            };
        }
    }

    /// Whether no further candidate can beat the clear one: any clear course, or with a kept
    /// side, a clear course on it.
    fn is_settled(&self, kept_side: bool) -> bool {
        self.clear.is_some_and(|best| best.kept || !kept_side)
    }

    fn best(self) -> Wanted {
        self.clear.unwrap_or(self.blocked).course
    }
}

/// The course from `wanted` that passes the threatening sphere tangent to its inflated surface,
/// on the side the course leans to, or on the other side when `flip`. An agent already inside
/// the reach slides out of it instead ([`slide_out`]), on either side.
///
/// `kept_side` replaces the side the course leans to, when the agent already chose one.
fn steer_around(
    threat: &Threat,
    wanted: Wanted,
    max_speed: f64,
    flip: bool,
    kept_side: Option<DVec3>,
) -> Wanted {
    let distance = threat.offset.length();
    // Unit direction from the agent to the obstacle's centre; any fixed one when they coincide.
    let to_centre = if distance * distance < TINY_SQ {
        DVec3::X
    } else {
        -threat.offset / distance
    };
    let leaning = kept_side
        .and_then(|side| across(side, to_centre))
        .unwrap_or_else(|| side_of(threat.relative_velocity, to_centre));
    let side = if flip { -leaning } else { leaning };
    let relative_speed = match threat.relative_velocity.length() {
        speed if speed * speed < TINY_SQ => wanted.velocity.length(),
        speed => speed,
    };

    let relative_direction = if threat.is_inside() {
        let depth = (1.0 - distance / threat.aim_reach).clamp(0.0, 1.0);
        slide_out(threat, -to_centre, side, relative_speed, depth)
    } else {
        // The cone from the agent that just touches the inflated sphere has half-angle α, with
        // sin α = radius / distance. The course aims along its edge on `side`.
        let sin_alpha = threat.aim_reach / distance;
        if sin_alpha >= 1.0 {
            side
        } else {
            to_centre * (1.0 - sin_alpha * sin_alpha).sqrt() + side * sin_alpha
        }
    };

    let velocity = (relative_direction * relative_speed + threat.obstacle_velocity)
        .clamp_length_max(max_speed);
    let direction = if velocity.length_squared() < TINY_SQ {
        wanted.direction
    } else {
        velocity.normalize()
    };
    Wanted {
        velocity,
        direction,
    }
}

/// The unit direction, in the obstacle's frame, for an agent inside the reach: along the sphere
/// as the wanted course does, so the agent keeps making for its goal, and out of it by the
/// larger of the wanted outward speed and `depth` (0 at the boundary, 1 at the centre) times the
/// relative speed. Where the course runs straight into the centre, `side` is the way along.
fn slide_out(
    threat: &Threat,
    outward: DVec3,
    side: DVec3,
    relative_speed: f64,
    depth: f64,
) -> DVec3 {
    let velocity = threat.relative_velocity;
    let wanted_out = velocity.dot(outward);
    let out = wanted_out.max(depth * relative_speed);
    let along = velocity - outward * wanted_out;
    let along = if along.length_squared() < TINY_SQ {
        side * (relative_speed - out).max(0.0)
    } else {
        along
    };
    let course = along + outward * out;
    if course.length_squared() < TINY_SQ {
        outward
    } else {
        course.normalize()
    }
}

/// The unit direction of `vector` across the unit `axis`; `None` when it runs along the axis.
fn across(vector: DVec3, axis: DVec3) -> Option<DVec3> {
    let across = vector - axis * vector.dot(axis);
    (across.length_squared() >= TINY_SQ).then(|| across.normalize())
}

/// The unit direction, across `axis`, the velocity leans to. A velocity along the axis has no
/// side: a fixed rule picks one, so that two agents meeting head-on both choose the same.
fn side_of(velocity: DVec3, axis: DVec3) -> DVec3 {
    if let Some(side) = across(velocity, axis) {
        return side;
    }
    let reference = if axis.y.abs() < 0.9 {
        DVec3::Y
    } else {
        DVec3::X
    };
    axis.cross(reference).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{Frame, MIN_ARRIVAL_RADII};

    const RADIUS: f64 = 1.0;
    const MAX_SPEED: f64 = 100.0;

    fn body_at(position: DVec3) -> Body {
        Body {
            position,
            velocity: DVec3::ZERO,
            angular_velocity: DVec3::ZERO,
            frame: Frame {
                forward: DVec3::NEG_Z,
                right: DVec3::X,
                up: DVec3::Y,
            },
            radius: RADIUS,
        }
    }

    fn wanted_toward(direction: DVec3, speed: f64) -> Wanted {
        Wanted {
            velocity: direction * speed,
            direction,
        }
    }

    /// A rock with key `Body(0)`; a scene of several rocks gives the others `keyed`.
    fn rock(position: DVec3, velocity: DVec3, radius: f64) -> Obstacle {
        Obstacle {
            key: ObstacleKey::Body(0),
            position,
            velocity,
            radius,
        }
    }

    fn keyed(obstacle: Obstacle, key: u64) -> Obstacle {
        Obstacle {
            key: ObstacleKey::Body(key),
            ..obstacle
        }
    }

    /// A drag so high that the braking limit never bites, for tests of the course alone.
    const UNLIMITED_DRAG: f64 = 1.0e6;

    /// A profile with the given top speed against the given drag.
    fn profile_with(max_speed: f64, linear_drag: f64) -> Profile {
        Profile {
            linear_drag,
            accel_forward: max_speed * linear_drag,
            ..Profile::TEST
        }
    }

    fn avoid_default(body: &Body, wanted: Wanted, obstacles: &[Obstacle]) -> Wanted {
        avoid(
            body,
            &profile_with(MAX_SPEED, UNLIMITED_DRAG),
            wanted,
            obstacles,
            &AvoidanceTuning::default(),
            None,
            Peers::alone(),
            &mut None,
            &mut Vec::new(),
        )
    }

    /// Where the agent comes closest to the obstacle's centre while flying `velocity`.
    fn closest_approach(body: &Body, velocity: DVec3, obstacle: &Obstacle) -> f64 {
        let offset = body.position - obstacle.position;
        let relative = velocity - obstacle.velocity;
        let t = (-offset.dot(relative) / relative.length_squared()).max(0.0);
        (offset + relative * t).length()
    }

    /// The reach of an obstacle for an agent at rest: it keeps the smallest clearance.
    fn reach_of(obstacle: &Obstacle) -> f64 {
        obstacle.radius + RADIUS * (1.0 + MIN_CLEARANCE_RADII)
    }

    #[test]
    fn course_with_no_obstacle_is_kept() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        assert_eq!(avoid_default(&body, wanted, &[]), wanted);
    }

    #[test]
    fn obstacle_beside_the_course_is_ignored() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let beside = rock(DVec3::new(100.0, 0.0, -200.0), DVec3::ZERO, 20.0);
        assert_eq!(avoid_default(&body, wanted, &[beside]), wanted);
    }

    #[test]
    fn obstacle_behind_is_ignored() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let behind = rock(DVec3::new(0.0, 0.0, 50.0), DVec3::ZERO, 20.0);
        assert_eq!(avoid_default(&body, wanted, &[behind]), wanted);
    }

    #[test]
    fn obstacle_beyond_the_horizon_is_ignored() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 10.0);
        // 10 units per second reaches it in 50 s, past the 10 s horizon.
        let far = rock(DVec3::new(0.0, 0.0, -520.0), DVec3::ZERO, 20.0);
        assert_eq!(avoid_default(&body, wanted, &[far]), wanted);
    }

    #[test]
    fn course_through_a_static_obstacle_passes_it_outside_its_reach() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let on_course = rock(DVec3::new(2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        let steered = avoid_default(&body, wanted, &[on_course]);

        let miss = closest_approach(&body, steered.velocity, &on_course);
        assert!(miss >= reach_of(&on_course), "misses by {miss}");
        assert!(
            steered.velocity.x < 0.0,
            "leans away from the side it was offset to"
        );
        assert!(
            (steered.velocity.length() - 80.0).abs() < 1e-9,
            "keeps its speed"
        );
        assert!((steered.direction - steered.velocity.normalize()).length() < 1e-12);
    }

    #[test]
    fn course_through_a_moving_obstacle_passes_it_outside_its_reach() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 100.0);
        // It crosses the course from the right, and meets the agent at z = -500 in 5 s.
        let crossing = rock(
            DVec3::new(200.0, 0.0, -500.0),
            DVec3::new(-40.0, 0.0, 0.0),
            30.0,
        );
        let steered = avoid_default(&body, wanted, &[crossing]);

        assert_ne!(steered, wanted);
        let miss = closest_approach(&body, steered.velocity, &crossing);
        assert!(miss >= reach_of(&crossing), "misses by {miss}");
    }

    #[test]
    fn a_far_obstacle_off_the_detour_does_not_change_it() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let near = rock(DVec3::new(0.0, 0.0, -200.0), DVec3::ZERO, 20.0);
        let far = rock(DVec3::new(0.0, 0.0, -600.0), DVec3::ZERO, 20.0);
        let both = avoid_default(&body, wanted, &[far, near]);
        assert_eq!(both, avoid_default(&body, wanted, &[near]));
    }

    /// Spheres of radius 40 in a row across the course at z = -300, centred at the given x.
    fn row(centres: &[f64]) -> Vec<Obstacle> {
        centres
            .iter()
            .map(|&x| rock(DVec3::new(x, 0.0, -300.0), DVec3::ZERO, 40.0))
            .collect()
    }

    /// The seconds until the agent flying `velocity` first reaches any of the obstacles.
    fn contact_time(body: &Body, velocity: DVec3, obstacles: &[Obstacle]) -> f64 {
        let tuning = AvoidanceTuning {
            horizon_s: f64::INFINITY,
            ..AvoidanceTuning::default()
        };
        let probe = Scene {
            body,
            profile: &profile_with(MAX_SPEED, UNLIMITED_DRAG),
            tuning: &tuning,
            goal: None,
            peers: Peers::alone(),
            spheres: &[],
        };
        let mut spheres = Vec::new();
        probe.fill_spheres(obstacles, velocity.length(), &mut spheres);
        let scene = Scene {
            spheres: &spheres,
            ..probe
        };
        scene
            .first_threat(velocity)
            .map_or(f64::INFINITY, |(_, threat)| threat.entry_time)
    }

    #[test]
    fn a_tangent_that_runs_into_a_neighbour_is_passed_over_for_the_other_side() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        // The course is 5 left of the first sphere's centre, so it leans right; the second
        // sphere sits so close on the right that no course passes between them.
        let squeeze = row(&[-5.0, 75.0]);
        let steered = avoid_default(&body, wanted, &squeeze);

        assert!(steered.velocity.x < 0.0, "goes round the left of the pair");
        for sphere in &squeeze {
            let miss = closest_approach(&body, steered.velocity, sphere);
            assert!(miss >= reach_of(sphere), "misses by {miss}");
        }
    }

    #[test]
    fn a_gap_wide_enough_is_still_taken_on_the_side_the_course_leans_to() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        // The second sphere is far enough right that the right tangent of the first misses it.
        let open = row(&[-5.0, 300.0]);
        let steered = avoid_default(&body, wanted, &open);
        assert!(steered.velocity.x > 0.0, "keeps to the side it leans to");
        for sphere in &open {
            assert!(closest_approach(&body, steered.velocity, sphere) >= reach_of(sphere));
        }
    }

    #[test]
    fn the_tangents_of_a_blocking_neighbour_are_tried_when_both_first_tangents_are_blocked() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        // Both tangents of the middle sphere run into the outer ones; only a course round an
        // end of the row is clear.
        let wall = row(&[-80.0, 0.0, 80.0]);
        let steered = avoid_default(&body, wanted, &wall);

        assert_eq!(contact_time(&body, steered.velocity, &wall), f64::INFINITY);
        assert!(steered.velocity.x.abs() > 1.0);
    }

    #[test]
    fn when_no_course_is_clear_the_one_reaching_an_obstacle_latest_is_taken() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        // More touching spheres than the search expands: no clear course is found.
        let centres: Vec<f64> = (-6..=6).map(|i| f64::from(i) * 80.0).collect();
        let wall = row(&centres);
        let steered = avoid_default(&body, wanted, &wall);

        let contact = contact_time(&body, steered.velocity, &wall);
        assert!(contact.is_finite(), "the row is wider than the search");
        assert!(steered.velocity.length() <= MAX_SPEED + 1e-9);
        assert!(contact >= contact_time(&body, wanted.velocity, &wall));
    }

    #[test]
    fn head_on_with_the_centre_picks_the_same_side_every_time() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let dead_ahead = rock(DVec3::new(0.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        let first = avoid_default(&body, wanted, &[dead_ahead]);
        let second = avoid_default(&body, wanted, &[dead_ahead]);

        assert_eq!(first, second);
        assert!(
            first.velocity.x.abs() + first.velocity.y.abs() > 1.0,
            "turns off the line"
        );
        assert!(closest_approach(&body, first.velocity, &dead_ahead) >= reach_of(&dead_ahead));
    }

    #[test]
    fn an_agent_inside_the_reach_is_pushed_out() {
        let body = body_at(DVec3::new(10.0, 0.0, 0.0));
        let wanted = wanted_toward(DVec3::NEG_Z, 50.0);
        let around = rock(DVec3::ZERO, DVec3::ZERO, 20.0);
        let steered = avoid_default(&body, wanted, &[around]);
        assert!(steered.velocity.x > 0.0, "moves away from the centre");
        assert!((steered.velocity.length() - 50.0).abs() < 1e-9);
    }

    #[test]
    fn an_agent_just_inside_the_reach_slides_along_it_toward_its_goal() {
        // 0.5 inside the reach of 21.5, with the goal ahead and a little inward.
        let body = body_at(DVec3::new(21.0, 0.0, 0.0));
        let wanted = wanted_toward(DVec3::new(-0.2, 0.0, -1.0).normalize(), 50.0);
        let around = rock(DVec3::ZERO, DVec3::ZERO, 20.0);
        let steered = avoid_default(&body, wanted, &[around]);

        assert!(steered.velocity.z < -48.0, "keeps making for the goal");
        assert!(steered.velocity.x > 0.0, "and does not go further in");
        assert!((steered.velocity.length() - 50.0).abs() < 1e-9);
    }

    #[test]
    fn an_agent_inside_the_reach_with_its_goal_straight_out_leaves_that_way() {
        let body = body_at(DVec3::new(10.0, 0.0, 0.0));
        let wanted = wanted_toward(DVec3::X, 50.0);
        let around = rock(DVec3::ZERO, DVec3::ZERO, 20.0);
        let steered = avoid_default(&body, wanted, &[around]);
        assert!((steered.direction - DVec3::X).length() < 1e-9);
    }

    #[test]
    fn the_steered_speed_is_capped() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, MAX_SPEED);
        // A fast obstacle receding along the course adds its velocity to the steered one.
        let fast = rock(
            DVec3::new(0.0, 0.0, -300.0),
            DVec3::new(0.0, 0.0, -60.0),
            40.0,
        );
        let steered = avoid_default(&body, wanted, &[fast]);
        assert!(steered.velocity.length() <= MAX_SPEED + 1e-9);
    }

    const DRAG: f64 = 2.0;

    /// The profile of the tests of the braking limits: the speed it can brake from within a
    /// distance is a few units per second.
    fn braking_profile() -> Profile {
        profile_with(MAX_SPEED, DRAG)
    }

    fn avoid_with_drag(body: &Body, wanted: Wanted, obstacles: &[Obstacle]) -> Wanted {
        avoid(
            body,
            &braking_profile(),
            wanted,
            obstacles,
            &AvoidanceTuning::default(),
            None,
            Peers::alone(),
            &mut None,
            &mut Vec::new(),
        )
    }

    #[test]
    fn a_threat_close_ahead_limits_the_closing_speed_to_what_braking_stops_short_of_the_surface() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, MAX_SPEED);
        let rock = rock(DVec3::new(0.0, 0.0, -30.0), DVec3::ZERO, 5.0);
        // The agent's centre may come as close as the rock's radius plus its own, plus the
        // smallest clearance.
        let room = 30.0 - (rock.radius + RADIUS) - RADIUS * MIN_CLEARANCE_RADII;
        let steered = avoid_with_drag(&body, wanted, &[rock]);
        let closing = steered.velocity.dot(DVec3::NEG_Z);
        let limit = stopping_speed(&braking_profile(), room);
        assert!(
            closing <= limit + 1e-9,
            "closing at {closing}, limit {limit}"
        );
        assert!(
            closing > DRAG * room,
            "thrust lets it close faster than drag alone"
        );
        assert!(steered.velocity.length() > 0.0);
    }

    #[test]
    fn a_body_flying_into_a_sphere_that_the_wanted_course_misses_has_its_closing_speed_limited() {
        let mut body = body_at(DVec3::ZERO);
        body.velocity = DVec3::new(0.0, 0.0, -50.0);
        // The wanted course passes the rock at 18 from its centre, outside the reach of 8, but
        // the body still leads straight into it.
        let wanted = wanted_toward(DVec3::new(0.6, 0.0, -0.8), MAX_SPEED);
        let rock = rock(DVec3::new(0.0, 0.0, -30.0), DVec3::ZERO, 5.0);
        let room = 30.0 - (rock.radius + RADIUS) - 50.0 * BRAKING_REACTION_S;
        let steered = avoid_with_drag(&body, wanted, &[rock]);
        let closing = steered.velocity.dot(DVec3::NEG_Z);
        let limit = stopping_speed(&braking_profile(), room);
        assert!(
            closing <= limit + 1e-9,
            "closing at {closing}, limit {limit}"
        );
        assert!(
            (steered.velocity.x - wanted.velocity.x).abs() < 1e-9,
            "the speed along the sphere is kept"
        );
    }

    fn braking_margin_with_velocity(velocity: DVec3) -> f64 {
        let mut body = body_at(DVec3::ZERO);
        body.velocity = velocity;
        let tuning = AvoidanceTuning::default();
        let scene = Scene {
            body: &body,
            profile: &braking_profile(),
            tuning: &tuning,
            goal: None,
            peers: Peers::alone(),
            spheres: &[],
        };
        scene.braking_margin()
    }

    #[test]
    fn the_braking_margin_grows_with_the_total_speed_from_the_smallest_clearance() {
        assert_eq!(
            braking_margin_with_velocity(DVec3::ZERO),
            RADIUS * MIN_CLEARANCE_RADII
        );
        // Speed in any direction counts, not only the speed toward a sphere.
        let sideways = braking_margin_with_velocity(DVec3::new(40.0, 0.0, 0.0));
        assert!((sideways - 40.0 * BRAKING_REACTION_S).abs() < 1e-9);
    }

    #[test]
    fn the_closing_speed_limit_is_taken_relative_to_a_moving_sphere() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, MAX_SPEED);
        // The sphere moves away along the course at 30: the agent may close on it by 30 more.
        let receding = rock(
            DVec3::new(0.0, 0.0, -30.0),
            DVec3::new(0.0, 0.0, -30.0),
            5.0,
        );
        let room = 30.0 - (receding.radius + RADIUS) - RADIUS * MIN_CLEARANCE_RADII;
        let steered = avoid_with_drag(&body, wanted, &[receding]);
        let relative_closing = (steered.velocity - receding.velocity).dot(DVec3::NEG_Z);
        let limit = stopping_speed(&braking_profile(), room);
        assert!(relative_closing <= limit + 1e-9, "{relative_closing}");
    }

    #[test]
    fn the_limit_removes_only_the_closing_component_of_the_detour() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, MAX_SPEED);
        let rock = rock(DVec3::new(0.0, 0.0, -30.0), DVec3::ZERO, 5.0);
        let free = avoid_default(&body, wanted, &[rock]);
        let limited = avoid_with_drag(&body, wanted, &[rock]);
        // The sideways part of the detour is kept whole, so the agent still gets round.
        let sideways = |velocity: DVec3| DVec3::new(velocity.x, velocity.y, 0.0);
        assert!(sideways(free.velocity).length() > 1.0);
        assert!((sideways(limited.velocity) - sideways(free.velocity)).length() < 1e-9);
        assert!(limited.velocity.z > free.velocity.z);
    }

    #[test]
    fn a_course_that_does_not_close_on_the_threat_keeps_its_speed() {
        // The rock is abeam and the agent flies past it: nothing to brake for.
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, MAX_SPEED);
        let rock = rock(DVec3::new(30.0, 0.0, 0.0), DVec3::ZERO, 5.0);
        let steered = avoid_with_drag(&body, wanted, &[rock]);
        assert!((steered.velocity - wanted.velocity).length() < 1e-9);
    }

    /// Radius 1 and the braking profile give clearances from 0.5 (at rest) up to 2 (from a
    /// closing speed of about 12).
    fn clearance_with_velocity(velocity: DVec3, obstacle: &Obstacle) -> f64 {
        let mut body = body_at(DVec3::ZERO);
        body.velocity = velocity;
        let tuning = AvoidanceTuning::default();
        let scene = Scene {
            body: &body,
            profile: &braking_profile(),
            tuning: &tuning,
            goal: None,
            peers: Peers::alone(),
            spheres: &[],
        };
        scene.clearance(obstacle)
    }

    #[test]
    fn the_clearance_grows_with_the_closing_speed_between_its_bounds() {
        let ahead = rock(DVec3::new(0.0, 0.0, -100.0), DVec3::ZERO, 10.0);
        let at_rest = clearance_with_velocity(DVec3::ZERO, &ahead);
        let slow = clearance_with_velocity(DVec3::new(0.0, 0.0, -10.0), &ahead);
        let fast = clearance_with_velocity(DVec3::new(0.0, 0.0, -50.0), &ahead);
        assert_eq!(at_rest, RADIUS * MIN_CLEARANCE_RADII);
        // Between the bounds, the clearance is the distance braking from the closing speed takes.
        let braking = stopping_distance(&braking_profile(), 10.0);
        assert!(at_rest < braking && braking < fast);
        assert!((slow - braking).abs() < 1e-9, "slow clearance {slow}");
        assert_eq!(fast, RADIUS * AvoidanceTuning::default().clearance_radii);
    }

    #[test]
    fn moving_away_or_alongside_keeps_the_smallest_clearance() {
        let ahead = rock(DVec3::new(0.0, 0.0, -100.0), DVec3::ZERO, 10.0);
        let away = clearance_with_velocity(DVec3::new(0.0, 0.0, 50.0), &ahead);
        let alongside = clearance_with_velocity(DVec3::new(50.0, 0.0, 0.0), &ahead);
        assert_eq!(away, RADIUS * MIN_CLEARANCE_RADII);
        assert_eq!(alongside, RADIUS * MIN_CLEARANCE_RADII);
    }

    #[test]
    fn the_closing_speed_is_taken_relative_to_the_obstacle() {
        // The obstacle flies the agent's own velocity: they don't close on each other.
        let along = rock(
            DVec3::new(0.0, 0.0, -100.0),
            DVec3::new(0.0, 0.0, -50.0),
            10.0,
        );
        let same = clearance_with_velocity(DVec3::new(0.0, 0.0, -50.0), &along);
        assert_eq!(same, RADIUS * MIN_CLEARANCE_RADII);
        // An obstacle coming at a resting agent does close on it.
        let coming = rock(
            DVec3::new(0.0, 0.0, -100.0),
            DVec3::new(0.0, 0.0, 50.0),
            10.0,
        );
        let closing = clearance_with_velocity(DVec3::ZERO, &coming);
        assert_eq!(closing, RADIUS * AvoidanceTuning::default().clearance_radii);
    }

    #[test]
    fn a_maximum_clearance_below_the_minimum_caps_the_clearance() {
        let ahead = rock(DVec3::new(0.0, 0.0, -100.0), DVec3::ZERO, 10.0);
        let mut body = body_at(DVec3::ZERO);
        body.velocity = DVec3::new(0.0, 0.0, -50.0);
        let tuning = AvoidanceTuning {
            clearance_radii: 0.25,
            ..AvoidanceTuning::default()
        };
        let scene = Scene {
            body: &body,
            profile: &braking_profile(),
            tuning: &tuning,
            goal: None,
            peers: Peers::alone(),
            spheres: &[],
        };
        assert_eq!(scene.clearance(&ahead), RADIUS * 0.25);
    }

    /// A profile that can't brake at all: no drag and no thrust.
    const NO_BRAKING: Profile = Profile {
        linear_drag: 0.0,
        accel_forward: 0.0,
        ..Profile::TEST
    };

    #[test]
    fn an_agent_that_cannot_brake_keeps_the_largest_clearance_while_it_closes() {
        let ahead = rock(DVec3::new(0.0, 0.0, -100.0), DVec3::ZERO, 10.0);
        let mut body = body_at(DVec3::ZERO);
        body.velocity = DVec3::new(0.0, 0.0, -1.0);
        let tuning = AvoidanceTuning::default();
        let scene = Scene {
            body: &body,
            profile: &NO_BRAKING,
            tuning: &tuning,
            goal: None,
            peers: Peers::alone(),
            spheres: &[],
        };
        assert_eq!(scene.clearance(&ahead), RADIUS * tuning.clearance_radii);
    }

    #[test]
    fn without_drag_the_clearance_is_the_distance_constant_braking_takes() {
        let ahead = rock(DVec3::new(0.0, 0.0, -100.0), DVec3::ZERO, 10.0);
        let mut body = body_at(DVec3::ZERO);
        body.velocity = DVec3::new(0.0, 0.0, -6.0);
        let profile = Profile {
            linear_drag: 0.0,
            ..braking_profile()
        };
        let tuning = AvoidanceTuning::default();
        let scene = Scene {
            body: &body,
            profile: &profile,
            tuning: &tuning,
            goal: None,
            peers: Peers::alone(),
            spheres: &[],
        };
        let braking = stopping_distance(&profile, 6.0);
        assert!(
            RADIUS * MIN_CLEARANCE_RADII < braking && braking < RADIUS * tuning.clearance_radii
        );
        assert!((scene.clearance(&ahead) - braking).abs() < 1e-12);
    }

    /// The clearance kept from `sphere` by an agent that can't brake, flying at the origin toward
    /// the sphere, heading for `goal`.
    fn clearance_toward(goal: Goal, sphere: &Obstacle) -> f64 {
        let mut body = body_at(DVec3::ZERO);
        body.velocity = DVec3::new(0.0, 0.0, -1.0);
        let tuning = AvoidanceTuning::default();
        let scene = Scene {
            body: &body,
            profile: &NO_BRAKING,
            tuning: &tuning,
            goal: Some(goal),
            peers: Peers::alone(),
            spheres: &[],
        };
        scene.clearance(sphere)
    }

    #[test]
    fn a_goal_next_to_a_sphere_limits_the_clearance_kept_from_it() {
        let sphere = rock(DVec3::new(0.0, 0.0, -20.0), DVec3::ZERO, 5.0);
        let arrival_radius = 3.0 * RADIUS;
        let goal_at = |x: f64| Goal {
            target: DVec3::new(x, 0.0, -20.0),
            arrival_radius,
        };
        let hull = 5.0 + RADIUS;
        // Room for the full clearance.
        assert_eq!(
            clearance_toward(goal_at(30.0), &sphere),
            RADIUS * AvoidanceTuning::default().clearance_radii
        );
        // 1 unit of room between the hull and the goal's arrival circle.
        assert!(
            (clearance_toward(goal_at(hull + arrival_radius + 1.0), &sphere) - 1.0).abs() < 1e-12
        );
        // A goal inside the hull leaves none.
        assert_eq!(clearance_toward(goal_at(hull - 1.0), &sphere), 0.0);
    }

    #[test]
    fn the_clearance_leaves_the_goals_arrival_circle_at_least_two_radii_wide_outside_the_reach() {
        let sphere = rock(DVec3::new(0.0, 0.0, -20.0), DVec3::ZERO, 5.0);
        let hull = 5.0 + RADIUS;
        let narrow = |x: f64| Goal {
            target: DVec3::new(x, 0.0, -20.0),
            arrival_radius: 0.1 * RADIUS,
        };
        // The circle is held to `MIN_ARRIVAL_RADII` radii, so 1 unit of room is left beyond it.
        let floor = RADIUS * MIN_ARRIVAL_RADII;
        assert!((clearance_toward(narrow(hull + floor + 1.0), &sphere) - 1.0).abs() < 1e-12);
    }

    /// `avoid_default` that keeps the side passed on in `commitment`.
    fn avoid_remembering(
        body: &Body,
        wanted: Wanted,
        obstacles: &[Obstacle],
        commitment: &mut Option<Commitment>,
    ) -> Wanted {
        avoid(
            body,
            &profile_with(MAX_SPEED, UNLIMITED_DRAG),
            wanted,
            obstacles,
            &AvoidanceTuning::default(),
            None,
            Peers::alone(),
            commitment,
            &mut Vec::new(),
        )
    }

    #[test]
    fn the_side_passed_on_is_kept_while_the_same_sphere_is_first() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let right_of_course = rock(DVec3::new(2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        let left_of_course = rock(DVec3::new(-2.0, 0.0, -300.0), DVec3::ZERO, 40.0);

        let mut memory = None;
        let first = avoid_remembering(&body, wanted, &[right_of_course], &mut memory);
        assert!(first.velocity.x < 0.0, "passes on the left");
        // The sphere is now left of the course, so a fresh choice passes on the right.
        assert!(avoid_default(&body, wanted, &[left_of_course]).velocity.x > 0.0);
        let kept = avoid_remembering(&body, wanted, &[left_of_course], &mut memory);
        assert!(kept.velocity.x < 0.0, "keeps the left");
    }

    #[test]
    fn the_memory_of_a_side_does_not_apply_to_another_sphere() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let right_of_course = rock(DVec3::new(2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        let left_of_course = rock(DVec3::new(-2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        // Beyond the horizon and the course: never a threat, but it comes first in the list.
        let distant = keyed(rock(DVec3::new(5000.0, 0.0, 5000.0), DVec3::ZERO, 10.0), 1);
        let other_sphere = keyed(left_of_course, 2);

        let mut memory = None;
        avoid_remembering(&body, wanted, &[right_of_course], &mut memory);
        let other = avoid_remembering(&body, wanted, &[distant, other_sphere], &mut memory);
        assert!(other.velocity.x > 0.0, "chooses freely");
    }

    #[test]
    fn the_side_passed_on_is_kept_when_the_sphere_changes_its_place_in_the_list() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let right_of_course = rock(DVec3::new(2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        let left_of_course = rock(DVec3::new(-2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        let distant = keyed(rock(DVec3::new(5000.0, 0.0, 5000.0), DVec3::ZERO, 10.0), 1);

        let mut memory = None;
        let first = avoid_remembering(&body, wanted, &[right_of_course], &mut memory);
        assert!(first.velocity.x < 0.0, "passes on the left");
        // A sphere enters the list ahead of it: the index changes, the key does not.
        let kept = avoid_remembering(&body, wanted, &[distant, left_of_course], &mut memory);
        assert!(kept.velocity.x < 0.0, "keeps the left");
    }

    #[test]
    fn the_sensing_range_covers_what_a_course_can_reach_inside_the_horizon() {
        let body = body_at(DVec3::ZERO);
        let tuning = AvoidanceTuning::default();
        let profile = profile_with(MAX_SPEED, UNLIMITED_DRAG);
        let range = sensing_range(&body, &profile, &tuning);
        let expected = MAX_SPEED * tuning.horizon_s + RADIUS + RADIUS * tuning.clearance_radii;
        assert!((range - expected).abs() < 1e-9, "range {range}");

        // The cull drops a sphere whose surface lies beyond the range, whatever clearance the
        // agent keeps from it, and keeps one just inside the reach of a body at rest.
        let still = |surface_distance: f64| {
            rock(
                DVec3::new(0.0, 0.0, -(surface_distance + 5.0)),
                DVec3::ZERO,
                5.0,
            )
        };
        let at_rest_limit = range - RADIUS * (tuning.clearance_radii - MIN_CLEARANCE_RADII);
        assert!(sphere_of(&body, &still(at_rest_limit - 1.0), MAX_SPEED).reachable);
        assert!(!sphere_of(&body, &still(range + 1.0), MAX_SPEED).reachable);
    }

    #[test]
    fn a_body_faster_than_the_top_speed_senses_further() {
        let tuning = AvoidanceTuning::default();
        let profile = profile_with(MAX_SPEED, UNLIMITED_DRAG);
        let mut body = body_at(DVec3::ZERO);
        let at_top = sensing_range(&body, &profile, &tuning);
        body.velocity = DVec3::NEG_Z * (2.0 * MAX_SPEED);
        assert!(sensing_range(&body, &profile, &tuning) > at_top);
    }

    #[test]
    fn a_kept_side_that_is_blocked_is_given_up() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let right_of_course = rock(DVec3::new(2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        let left_of_course = rock(DVec3::new(-2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        // Sits where the course passing on the left would go, clear of the straight course.
        let blocker = keyed(rock(DVec3::new(-60.0, 0.0, -330.0), DVec3::ZERO, 30.0), 1);

        let mut memory = None;
        avoid_remembering(&body, wanted, &[right_of_course], &mut memory);
        let chosen = avoid_remembering(&body, wanted, &[left_of_course, blocker], &mut memory);
        assert!(chosen.velocity.x > 0.0, "passes on the right instead");
        for sphere in [&left_of_course, &blocker] {
            let miss = closest_approach(&body, chosen.velocity, sphere);
            assert!(miss >= reach_of(sphere), "misses by {miss}");
        }
    }

    #[test]
    fn a_kept_side_blocked_by_a_farther_sphere_is_kept_when_it_can_be_passed() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let right_of_course = rock(DVec3::new(2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        let left_of_course = rock(DVec3::new(-2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        // Small, and on the tangent course on the left: that course is blocked, the one round
        // this sphere too is clear, and the right side is clear at once.
        let blocker = keyed(rock(DVec3::new(-40.0, 0.0, -330.0), DVec3::ZERO, 10.0), 1);

        let mut memory = None;
        avoid_remembering(&body, wanted, &[right_of_course], &mut memory);
        let chosen = avoid_remembering(&body, wanted, &[left_of_course, blocker], &mut memory);
        assert!(chosen.velocity.x < 0.0, "keeps the left");
        for sphere in [&left_of_course, &blocker] {
            let miss = closest_approach(&body, chosen.velocity, sphere);
            assert!(miss >= reach_of(sphere), "misses by {miss}");
        }
    }

    #[test]
    fn the_aim_radius_exceeds_the_reach_by_the_aim_factor_of_the_clearance_only() {
        let body = body_at(DVec3::ZERO);
        // Within the horizon at 80 per second, and far larger than the agent.
        let big = rock(DVec3::new(0.0, 0.0, -600.0), DVec3::ZERO, 300.0);
        let tuning = AvoidanceTuning::default();
        let scene = Scene {
            body: &body,
            profile: &braking_profile(),
            tuning: &tuning,
            goal: None,
            peers: Peers::alone(),
            spheres: &[],
        };
        let threat = scene
            .sphere(&big, 80.0)
            .threat(DVec3::NEG_Z * 80.0, tuning.horizon_s)
            .expect("the course leads into the rock");
        let band = threat.aim_reach - threat.reach;
        let clearance = scene.clearance(&big);
        assert!(
            (band - (tuning.aim_factor - 1.0) * clearance).abs() < 1e-9,
            "band {band}"
        );
        assert!(band < RADIUS, "the band doesn't grow with the rock");
    }

    /// Two spheres of radius 40 across the course at z = -300, `gap` apart between their surfaces.
    fn gate(gap: f64) -> [Obstacle; 2] {
        let half = 40.0 + gap / 2.0;
        [-half, half].map(|x| rock(DVec3::new(x, 0.0, -300.0), DVec3::ZERO, 40.0))
    }

    #[test]
    fn a_slow_agent_flies_through_a_gap_that_a_fast_one_goes_round() {
        // The agent needs half the gap to exceed its radius plus the clearance: 1.5 when slow, 3
        // when fast. 5 apart gives each side 2.5.
        let gate = gate(5.0);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);

        let slow = body_at(DVec3::ZERO);
        assert_eq!(avoid_with_drag(&slow, wanted, &gate), wanted);

        let mut fast = body_at(DVec3::ZERO);
        fast.velocity = wanted.velocity;
        let steered = avoid_with_drag(&fast, wanted, &gate);
        assert_ne!(steered.direction, wanted.direction);
        assert!(steered.velocity.x.abs() > 1.0, "turns off the line");
    }

    #[test]
    fn the_same_state_gives_the_same_course() {
        let mut body = body_at(DVec3::ZERO);
        body.velocity = DVec3::new(0.0, 0.0, -30.0);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let gate = gate(5.0);
        assert_eq!(
            avoid_with_drag(&body, wanted, &gate),
            avoid_with_drag(&body, wanted, &gate)
        );
    }

    /// The sphere of `obstacle` for the agent `body`, for courses no faster than `speed_bound`.
    fn sphere_of(body: &Body, obstacle: &Obstacle, speed_bound: f64) -> Sphere {
        let tuning = AvoidanceTuning::default();
        Scene {
            body,
            profile: &profile_with(MAX_SPEED, UNLIMITED_DRAG),
            tuning: &tuning,
            goal: None,
            peers: Peers::alone(),
            spheres: &[],
        }
        .sphere(obstacle, speed_bound)
    }

    #[test]
    fn a_sphere_is_reachable_only_within_what_the_horizon_lets_a_course_cover() {
        let body = body_at(DVec3::ZERO);
        let horizon_s = AvoidanceTuning::default().horizon_s;
        let speed = 80.0;
        let still = |z: f64| rock(DVec3::new(0.0, 0.0, z), DVec3::ZERO, 10.0);
        let reach = reach_of(&still(0.0));
        let covered = speed * horizon_s;
        assert!(sphere_of(&body, &still(-(reach + covered - 1.0)), speed).reachable);
        assert!(!sphere_of(&body, &still(-(reach + covered + 1.0)), speed).reachable);

        // A sphere flying at the agent closes the gap with its own speed as well.
        let coming = |z: f64| rock(DVec3::new(0.0, 0.0, z), DVec3::new(0.0, 0.0, 50.0), 10.0);
        let covered = (speed + 50.0) * horizon_s;
        assert!(sphere_of(&body, &coming(-(reach + covered - 1.0)), speed).reachable);
        assert!(!sphere_of(&body, &coming(-(reach + covered + 1.0)), speed).reachable);
    }

    #[test]
    fn a_sphere_the_agent_is_inside_of_is_reachable_at_any_speed() {
        let body = body_at(DVec3::ZERO);
        let around = rock(DVec3::new(0.0, 0.0, -5.0), DVec3::ZERO, 20.0);
        assert!(sphere_of(&body, &around, 0.0).reachable);
    }

    #[test]
    fn an_unreachable_sphere_is_never_a_threat_to_a_course_within_the_speed_bound() {
        let body = body_at(DVec3::ZERO);
        let tuning = AvoidanceTuning::default();
        let speed = 80.0;
        let mut unreachable = 0;
        for obstacle_speed in [-50.0, 0.0, 50.0] {
            for step in 0..400 {
                let z = -(step as f64) * 5.0;
                let obstacle = rock(
                    DVec3::new(3.0, 0.0, z),
                    DVec3::new(0.0, 0.0, -obstacle_speed),
                    10.0,
                );
                let sphere = sphere_of(&body, &obstacle, speed);
                let threat = sphere.threat(DVec3::NEG_Z * speed, tuning.horizon_s);
                if !sphere.reachable {
                    unreachable += 1;
                    assert!(threat.is_none(), "z {z}, obstacle speed {obstacle_speed}");
                }
            }
        }
        assert!(unreachable > 100, "the sweep covers unreachable spheres");
    }

    #[test]
    fn spheres_that_cannot_be_reached_do_not_change_the_course() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let blocker = rock(DVec3::new(2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        let far = [
            rock(DVec3::new(0.0, 0.0, -50_000.0), DVec3::ZERO, 40.0),
            rock(DVec3::new(9_000.0, 0.0, -100.0), DVec3::ZERO, 40.0),
        ];
        let alone = avoid_with_drag(&body, wanted, &[blocker]);
        let with_far = avoid_with_drag(&body, wanted, &[far[0], blocker, far[1]]);
        assert_ne!(alone, wanted, "the blocker is a threat");
        assert_eq!(with_far, alone);
    }

    #[test]
    fn a_threat_far_ahead_does_not_limit_the_speed() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, MAX_SPEED);
        let far = rock(DVec3::new(0.0, 0.0, -900.0), DVec3::ZERO, 5.0);
        let steered = avoid_with_drag(&body, wanted, &[far]);
        assert!(steered.velocity.length() > MAX_SPEED - 1e-6);
    }

    const OWN: AgentId = AgentId::new(0);
    const OTHER: AgentId = AgentId::new(1);

    /// An agent obstacle, which `other` is in the peers of the tests below.
    fn agent_sphere(position: DVec3, velocity: DVec3, radius: f64) -> Obstacle {
        Obstacle {
            key: ObstacleKey::Agent(OTHER),
            position,
            velocity,
            radius,
        }
    }

    /// The agent `OWN` avoiding `obstacles` with `other` as the peer `OTHER`, tracking `tracked`.
    fn avoid_among(
        profile: &Profile,
        body: &Body,
        wanted: Wanted,
        obstacles: &[Obstacle],
        other: Peer,
        tracked: Option<ObstacleKey>,
    ) -> Wanted {
        let own = Peer {
            steering: true,
            tracks: tracked,
        };
        avoid(
            body,
            profile,
            wanted,
            obstacles,
            &AvoidanceTuning::default(),
            None,
            Peers {
                own: OWN,
                tracked,
                all: &[own, other],
            },
            &mut None,
            &mut Vec::new(),
        )
    }

    const STEERING: Peer = Peer {
        steering: true,
        tracks: None,
    };

    /// A head-on pair at 20 units per second each, 150 apart, avoiding with `other` as the peer.
    /// The apex of a pair that avoids back is at rest, so it is inside the horizon too.
    fn head_on_course(other: Peer) -> (Wanted, Wanted) {
        let mut body = body_at(DVec3::ZERO);
        body.velocity = DVec3::NEG_Z * 20.0;
        let wanted = wanted_toward(DVec3::NEG_Z, 20.0);
        let oncoming = agent_sphere(DVec3::new(0.0, 0.0, -150.0), DVec3::Z * 20.0, 10.0);
        let profile = profile_with(MAX_SPEED, UNLIMITED_DRAG);
        (
            avoid_among(&profile, &body, wanted, &[oncoming], other, None),
            wanted,
        )
    }

    #[test]
    fn an_agent_that_avoids_back_gets_half_of_the_detour_to_take() {
        let (shared, wanted) = head_on_course(STEERING);
        let (alone, _) = head_on_course(Peer {
            steering: false,
            tracks: None,
        });
        let sideways = |course: Wanted| course.velocity.cross(wanted.velocity).length();
        assert!(sideways(alone) > 1.0, "the full detour turns the course");
        let ratio = sideways(shared) / sideways(alone);
        assert!((ratio - RECIPROCAL_SHARE).abs() < 1e-9, "ratio {ratio}");
    }

    #[test]
    fn an_agent_that_tracks_this_one_is_avoided_in_full() {
        let tracking = Peer {
            steering: true,
            tracks: Some(ObstacleKey::Agent(OWN)),
        };
        let (tracked_by, _) = head_on_course(tracking);
        let (resting, _) = head_on_course(Peer {
            steering: false,
            tracks: None,
        });
        assert_eq!(tracked_by, resting);
    }

    #[test]
    fn the_obstacle_an_agent_tracks_is_left_out_of_its_scene() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let profile = profile_with(MAX_SPEED, UNLIMITED_DRAG);
        let ahead = rock(DVec3::new(2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
        let among = |tracked| avoid_among(&profile, &body, wanted, &[ahead], STEERING, tracked);
        assert_ne!(among(None), wanted, "the rock is a threat");
        assert_eq!(among(Some(ObstacleKey::Body(0))), wanted);
        assert_ne!(
            among(Some(ObstacleKey::Body(1))),
            wanted,
            "another key leaves the rock in"
        );
    }

    #[test]
    fn an_agent_that_avoids_back_leaves_half_of_the_braking_room_to_it() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, MAX_SPEED);
        let profile = braking_profile();
        let ahead = agent_sphere(DVec3::new(0.0, 0.0, -30.0), DVec3::ZERO, 5.0);
        let closing = |other| {
            avoid_among(&profile, &body, wanted, &[ahead], other, None)
                .velocity
                .dot(DVec3::NEG_Z)
        };
        let room = 30.0 - (ahead.radius + RADIUS) - RADIUS * MIN_CLEARANCE_RADII;
        // The sphere is at rest and so is the apex, so only the room differs.
        let full = closing(Peer {
            steering: false,
            tracks: None,
        });
        assert!((full - stopping_speed(&profile, room)).abs() < 1e-9);
        let shared = closing(STEERING);
        let half = stopping_speed(&profile, room * RECIPROCAL_SHARE);
        assert!(
            (shared - half).abs() < 1e-9,
            "closing {shared}, half {half}"
        );
        assert!(shared < full);
    }

    #[test]
    fn only_an_agent_key_can_avoid_back() {
        let peers = Peers {
            own: OWN,
            tracked: None,
            all: &[STEERING, STEERING],
        };
        assert_eq!(peers.share_of(ObstacleKey::Body(1)), 1.0);
        assert_eq!(peers.share_of(ObstacleKey::Agent(OTHER)), RECIPROCAL_SHARE);
        // An agent the peers don't list is a plain sphere.
        assert_eq!(peers.share_of(ObstacleKey::Agent(AgentId::new(7))), 1.0);
    }
}
