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

/// The obstacle an agent reaches first on its wanted course, in the obstacle's frame.
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

/// Steers a wanted course around the obstacle it reaches first, if any: the new course passes
/// the obstacle's inflated sphere on the side the course already leans to, tangent to it
/// (Reynolds, "Steering Behaviors For Autonomous Characters", 1999, obstacle avoidance).
pub(crate) fn avoid(
    body: &Body,
    max_speed: f64,
    wanted: Wanted,
    obstacles: &[Obstacle],
    tuning: &AvoidanceTuning,
) -> Wanted {
    if wanted.direction == DVec3::ZERO {
        return wanted;
    }
    let reach_margin = body.radius * (1.0 + tuning.clearance_radii);
    let first = obstacles
        .iter()
        .filter_map(|o| {
            threat(
                body.position,
                wanted.velocity,
                reach_margin,
                o,
                tuning.horizon_s,
            )
        })
        .min_by(|a, b| a.entry_time.total_cmp(&b.entry_time));
    match first {
        Some(threat) => steer_around(&threat, wanted, max_speed, tuning.aim_factor),
        None => wanted,
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

fn steer_around(threat: &Threat, wanted: Wanted, max_speed: f64, aim_factor: f64) -> Wanted {
    let distance = threat.offset.length();
    // Unit direction from the agent to the obstacle's centre; any fixed one when they coincide.
    let to_centre = if distance * distance < TINY_SQ {
        DVec3::X
    } else {
        -threat.offset / distance
    };
    let side = side_of(threat.relative_velocity, to_centre);
    let relative_speed = match threat.relative_velocity.length() {
        speed if speed * speed < TINY_SQ => wanted.velocity.length(),
        speed => speed,
    };

    let relative_direction = if threat.is_inside() {
        -to_centre
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

    fn avoid_default(body: &Body, wanted: Wanted, obstacles: &[Obstacle]) -> Wanted {
        avoid(
            body,
            MAX_SPEED,
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
    fn the_obstacle_reached_first_decides() {
        let body = body_at(DVec3::ZERO);
        let wanted = wanted_toward(DVec3::NEG_Z, 80.0);
        let near = rock(DVec3::new(0.0, 0.0, -200.0), DVec3::ZERO, 20.0);
        let far = rock(DVec3::new(0.0, 0.0, -600.0), DVec3::ZERO, 20.0);
        let both = avoid_default(&body, wanted, &[far, near]);
        assert_eq!(both, avoid_default(&body, wanted, &[near]));
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
}
