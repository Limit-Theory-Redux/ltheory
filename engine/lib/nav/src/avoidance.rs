use glam::DVec3;

use crate::agent::Body;
use crate::behaviour::{TINY_SQ, Wanted};
use crate::obstacle::Obstacle;

/// How far ahead and how widely an agent avoids obstacles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AvoidanceTuning {
    /// Obstacles the agent would reach later than this are ignored, in seconds.
    pub horizon_s: f64,
    /// Distance kept from an obstacle's surface besides the agent's own radius, in agent radii.
    pub clearance_radii: f64,
    /// The new course passes the inflated sphere at this multiple of its radius. A value above 1
    /// keeps the course off the boundary the threat test uses, so it doesn't flip every step.
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
    obstacle_velocity: DVec3,
}

impl Threat {
    fn is_inside(&self) -> bool {
        self.entry_time == 0.0 && self.offset.length_squared() <= self.reach * self.reach
    }
}

/// The agent and the obstacles its courses are tested against.
struct Scene<'a> {
    position: DVec3,
    /// The agent's radius plus its clearance.
    reach_margin: f64,
    horizon_s: f64,
    obstacles: &'a [Obstacle],
}

impl Scene<'_> {
    /// The obstacle (its index and threat) the agent reaches first flying `velocity`.
    fn first_threat(&self, velocity: DVec3) -> Option<(usize, Threat)> {
        self.obstacles
            .iter()
            .enumerate()
            .filter_map(|(index, obstacle)| {
                threat(
                    self.position,
                    velocity,
                    self.reach_margin,
                    obstacle,
                    self.horizon_s,
                )
                .map(|threat| (index, threat))
            })
            .min_by(|a, b| a.1.entry_time.total_cmp(&b.1.entry_time))
    }
}

/// A sphere whose tangent courses are still to be tried, with the course that reaches it.
#[derive(Clone, Copy)]
struct Pending {
    index: usize,
    threat: Threat,
    course: Wanted,
}

/// Steers a wanted course clear of the obstacles, if it leads into one. The candidates are the
/// tangent courses of the sphere reached first, on the side the course leans to and on the
/// other; a candidate that another sphere blocks adds that sphere's tangents in turn. Of the
/// candidates that reach nothing within the horizon, the one turning least from the wanted
/// course is taken; if none is clear, the one that reaches an obstacle latest (the wanted course
/// included). The tangent is the one of Reynolds, "Steering Behaviors For Autonomous
/// Characters", 1999, obstacle avoidance.
///
/// While the wanted course leads into a sphere, the speed is also limited to `linear_drag` times
/// the distance to the sphere's surface plus the agent's radius, the speed from which drag alone
/// stops the agent there (the rule of [`arrive`](crate::behaviour::arrive)). A fast agent turns
/// slowly, so without the limit it reaches the sphere before its velocity has followed the
/// detour.
pub(crate) fn avoid(
    body: &Body,
    max_speed: f64,
    linear_drag: f64,
    wanted: Wanted,
    obstacles: &[Obstacle],
    tuning: &AvoidanceTuning,
) -> Wanted {
    if wanted.direction == DVec3::ZERO {
        return wanted;
    }
    let scene = Scene {
        position: body.position,
        reach_margin: body.radius * (1.0 + tuning.clearance_radii),
        horizon_s: tuning.horizon_s,
        obstacles,
    };
    let Some((index, first)) = scene.first_threat(wanted.velocity) else {
        return wanted;
    };
    if first.is_inside() {
        return steer_around(&first, wanted, max_speed, tuning.aim_factor, false);
    }
    let stopping_distance =
        first.entry_time * wanted.velocity.length() + body.radius * tuning.clearance_radii;
    let course = search_courses(
        &scene,
        wanted,
        max_speed,
        tuning.aim_factor,
        Pending {
            index,
            threat: first,
            course: wanted,
        },
    );
    limit_speed(course, linear_drag * stopping_distance)
}

/// `course` with its speed reduced to `limit` if it is faster; the direction is kept.
fn limit_speed(course: Wanted, limit: f64) -> Wanted {
    let speed = course.velocity.length();
    if speed <= limit {
        return course;
    }
    Wanted {
        velocity: course.velocity * (limit / speed),
        direction: course.direction,
    }
}

/// The best course found by expanding spheres breadth-first from `first`, which blocks `wanted`.
fn search_courses(
    scene: &Scene,
    wanted: Wanted,
    max_speed: f64,
    aim_factor: f64,
    first: Pending,
) -> Wanted {
    let mut queue = [None; MAX_EXPANDED_THREATS];
    queue[0] = Some(first);
    let mut queued = 1;
    let mut choice = Choice::new(wanted, first.threat.entry_time);
    let mut next = 0;
    while next < queued && choice.clear.is_none() {
        let Some(pending) = queue[next] else { break };
        next += 1;
        for flip in [false, true] {
            let course = steer_around(&pending.threat, pending.course, max_speed, aim_factor, flip);
            let Some((index, threat)) = scene.first_threat(course.velocity) else {
                choice.offer_clear(course, course.direction.dot(wanted.direction));
                continue;
            };
            choice.offer_blocked(course, threat.entry_time);
            let known = queue[..queued].iter().flatten().any(|p| p.index == index);
            if queued < MAX_EXPANDED_THREATS && !known {
                queue[queued] = Some(Pending {
                    index,
                    threat,
                    course,
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
            },
        }
    }

    /// Offers a course that reaches nothing; `alignment` is the cosine of its turn from the
    /// wanted course.
    fn offer_clear(&mut self, course: Wanted, alignment: f64) {
        if self.clear.is_none_or(|best| alignment > best.score) {
            self.clear = Some(Scored {
                course,
                score: alignment,
            });
        }
    }

    /// Offers a course that reaches an obstacle after `entry_time` seconds.
    fn offer_blocked(&mut self, course: Wanted, entry_time: f64) {
        if entry_time > self.blocked.score {
            self.blocked = Scored {
                course,
                score: entry_time,
            };
        }
    }

    fn best(self) -> Wanted {
        self.clear.unwrap_or(self.blocked).course
    }
}

/// The time the agent, flying `wanted_velocity`, first comes within `reach_margin` of the
/// obstacle's surface; `None` when it doesn't within the horizon.
fn threat(
    position: DVec3,
    wanted_velocity: DVec3,
    reach_margin: f64,
    obstacle: &Obstacle,
    horizon_s: f64,
) -> Option<Threat> {
    let offset = position - obstacle.position;
    let relative_velocity = wanted_velocity - obstacle.velocity;
    let reach = obstacle.radius + reach_margin;
    let make = |entry_time| Threat {
        entry_time,
        offset,
        relative_velocity,
        reach,
        obstacle_velocity: obstacle.velocity,
    };

    // Solve |offset + relative_velocity·t| = reach for the first t ≥ 0: a·t² + 2b·t + c = 0.
    let c = offset.length_squared() - reach * reach;
    if c <= 0.0 {
        return Some(make(0.0));
    }
    let a = relative_velocity.length_squared();
    let b = offset.dot(relative_velocity);
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

/// The course from `wanted` that passes the threatening sphere tangent to its inflated surface,
/// on the side the course leans to, or on the other side when `flip`. An agent already inside
/// the reach slides out of it instead ([`slide_out`]), on either side.
fn steer_around(
    threat: &Threat,
    wanted: Wanted,
    max_speed: f64,
    aim_factor: f64,
    flip: bool,
) -> Wanted {
    let distance = threat.offset.length();
    // Unit direction from the agent to the obstacle's centre; any fixed one when they coincide.
    let to_centre = if distance * distance < TINY_SQ {
        DVec3::X
    } else {
        -threat.offset / distance
    };
    let leaning = side_of(threat.relative_velocity, to_centre);
    let side = if flip { -leaning } else { leaning };
    let relative_speed = match threat.relative_velocity.length() {
        speed if speed * speed < TINY_SQ => wanted.velocity.length(),
        speed => speed,
    };

    let relative_direction = if threat.is_inside() {
        let depth = (1.0 - distance / (threat.reach * aim_factor)).clamp(0.0, 1.0);
        slide_out(threat, -to_centre, side, relative_speed, depth)
    } else {
        // The cone from the agent that just touches the inflated sphere has half-angle α, with
        // sin α = radius / distance. The course aims along its edge on `side`.
        let sin_alpha = threat.reach * aim_factor / distance;
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

/// The unit direction, across `axis`, the velocity leans to. A velocity along the axis has no
/// side: a fixed rule picks one, so that two agents meeting head-on both choose the same.
fn side_of(velocity: DVec3, axis: DVec3) -> DVec3 {
    let across = velocity - axis * velocity.dot(axis);
    if across.length_squared() >= TINY_SQ {
        return across.normalize();
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
    use crate::agent::Frame;

    const RADIUS: f64 = 1.0;
    const MAX_SPEED: f64 = 100.0;

    fn body_at(position: DVec3) -> Body {
        Body {
            position,
            velocity: DVec3::ZERO,
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

    fn rock(position: DVec3, velocity: DVec3, radius: f64) -> Obstacle {
        Obstacle {
            position,
            velocity,
            radius,
        }
    }

    /// A drag so high that the braking limit never bites, for tests of the course alone.
    const UNLIMITED_DRAG: f64 = 1.0e6;

    fn avoid_default(body: &Body, wanted: Wanted, obstacles: &[Obstacle]) -> Wanted {
        avoid(
            body,
            MAX_SPEED,
            UNLIMITED_DRAG,
            wanted,
            obstacles,
            &AvoidanceTuning::default(),
        )
    }

    /// Where the agent comes closest to the obstacle's centre while flying `velocity`.
    fn closest_approach(body: &Body, velocity: DVec3, obstacle: &Obstacle) -> f64 {
        let offset = body.position - obstacle.position;
        let relative = velocity - obstacle.velocity;
        let t = (-offset.dot(relative) / relative.length_squared()).max(0.0);
        (offset + relative * t).length()
    }

    fn reach_of(obstacle: &Obstacle) -> f64 {
        obstacle.radius + RADIUS * (1.0 + AvoidanceTuning::default().clearance_radii)
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
        let margin = RADIUS * (1.0 + AvoidanceTuning::default().clearance_radii);
        obstacles
            .iter()
            .filter_map(|o| threat(body.position, velocity, margin, o, f64::INFINITY))
            .map(|t| t.entry_time)
            .fold(f64::INFINITY, f64::min)
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
        // 0.5 inside the reach of 23, with the goal ahead and a little inward.
        let body = body_at(DVec3::new(22.5, 0.0, 0.0));
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

    fn avoid_with_drag(body: &Body, wanted: Wanted, obstacles: &[Obstacle]) -> Wanted {
        avoid(
            body,
            MAX_SPEED,
            DRAG,
            wanted,
            obstacles,
            &AvoidanceTuning::default(),
        )
    }

    #[test]
    fn a_threat_close_ahead_limits_the_speed_to_what_drag_stops_at_the_surface() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, MAX_SPEED);
        let rock = rock(DVec3::new(0.0, 0.0, -30.0), DVec3::ZERO, 5.0);
        // The agent's centre may come as close as the rock's radius plus its own.
        let stopping = 30.0 - (rock.radius + RADIUS);
        let steered = avoid_with_drag(&body, wanted, &[rock]);
        assert!(steered.velocity.length() <= DRAG * stopping + 1e-9);
        assert!(steered.velocity.length() > 0.0);
    }

    #[test]
    fn the_limit_keeps_the_direction_of_the_detour() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, MAX_SPEED);
        let rock = rock(DVec3::new(0.0, 0.0, -30.0), DVec3::ZERO, 5.0);
        let free = avoid_default(&body, wanted, &[rock]);
        let limited = avoid_with_drag(&body, wanted, &[rock]);
        assert!(limited.velocity.length() < free.velocity.length());
        assert!(limited.velocity.normalize().dot(free.velocity.normalize()) > 1.0 - 1e-9);
    }

    #[test]
    fn a_threat_far_ahead_does_not_limit_the_speed() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, MAX_SPEED);
        let far = rock(DVec3::new(0.0, 0.0, -900.0), DVec3::ZERO, 5.0);
        let steered = avoid_with_drag(&body, wanted, &[far]);
        assert!(steered.velocity.length() > MAX_SPEED - 1e-6);
    }
}
