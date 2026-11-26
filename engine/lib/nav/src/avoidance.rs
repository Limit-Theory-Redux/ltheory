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

/// The share of the avoidance an agent takes against an agent that steers: half, so that a pair
/// that avoids each other takes all of it (the reciprocal velocity obstacle).
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

impl Peer {
    /// What a free slot is to the others: it neither steers nor tracks.
    pub const ABSENT: Self = Self {
        steering: false,
        tracks: None,
    };
}

/// The agents around the agent being steered, for [`avoid`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct Peers<'a> {
    /// The agent being steered.
    pub own: AgentId,
    /// The obstacle its behaviour tracks; it is left out of the scene.
    pub tracked: Option<ObstacleKey>,
    /// Every slot of the world, in agent order.
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

    /// Whether the agent leaves `key` out of its scene: the obstacle it tracks, and an agent that
    /// tracks it and lies behind it (`behind`, the dot product of the vector to that agent with
    /// the agent's wanted direction, is negative). A tracker behind is the one that keeps the
    /// distance: braking would close on it, so avoiding it forbids the agent's own braking.
    fn leaves_out(&self, key: ObstacleKey, behind: f64) -> bool {
        if Some(key) == self.tracked {
            return true;
        }
        let ObstacleKey::Agent(id) = key else {
            return false;
        };
        let tracks_this = self
            .all
            .get(id.index())
            .is_some_and(|peer| peer.tracks == Some(ObstacleKey::Agent(self.own)));
        tracks_this && behind < 0.0
    }

    /// The share of the avoidance of `key` the agent takes: [`RECIPROCAL_SHARE`] when `key` is a
    /// steering agent, which takes its half of the detour; 1 otherwise.
    fn share_of(&self, key: ObstacleKey) -> f64 {
        let ObstacleKey::Agent(id) = key else {
            return 1.0;
        };
        let avoids_back = self.all.get(id.index()).is_some_and(|peer| peer.steering);
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
    /// One per obstacle the agent doesn't leave out ([`Peers::leaves_out`]), in the order of the
    /// obstacles.
    spheres: &'a [Sphere],
}

impl Scene<'_> {
    /// Replaces `out` with the spheres of `obstacles` but the ones the agent leaves out when it
    /// heads `heading`, for courses no faster than `speed_bound`.
    fn fill_spheres(
        &self,
        obstacles: &[Obstacle],
        heading: DVec3,
        speed_bound: f64,
        out: &mut Vec<Sphere>,
    ) {
        out.clear();
        out.extend(
            obstacles
                .iter()
                .filter(|obstacle| {
                    let behind = (obstacle.position - self.body.position).dot(heading);
                    !self.peers.leaves_out(obstacle.key, behind)
                })
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
    probe.fill_spheres(obstacles, wanted.direction, speed_bound, spheres);
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

impl Commitment {
    /// Whether the remembered side is about the sphere `key`.
    pub(crate) fn is_about(&self, key: ObstacleKey) -> bool {
        self.obstacle == key
    }
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
mod tests;
