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
fn a_detour_stays_under_the_speed_cap() {
    let body = body_at(DVec3::ZERO);
    let cap = MAX_SPEED * 0.5;
    let profile = Profile {
        speed_cap: cap,
        ..profile_with(MAX_SPEED, UNLIMITED_DRAG)
    };
    // A course at the cap, through a rock: the detour must not exceed the cap.
    let wanted = wanted_toward(DVec3::NEG_Z, cap);
    let on_course = rock(DVec3::new(2.0, 0.0, -300.0), DVec3::ZERO, 40.0);
    let steered = avoid(
        &body,
        &profile,
        wanted,
        &[on_course],
        &AvoidanceTuning::default(),
        None,
        Peers::alone(),
        &mut None,
        &mut Vec::new(),
    );

    assert_ne!(steered, wanted, "it detours");
    assert!(
        steered.velocity.length() <= cap + 1e-9,
        "flies {} against a cap of {cap}",
        steered.velocity.length()
    );
    assert!(closest_approach(&body, steered.velocity, &on_course) >= reach_of(&on_course));
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
    probe.fill_spheres(obstacles, velocity, velocity.length(), &mut spheres);
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
    assert!(RADIUS * MIN_CLEARANCE_RADII < braking && braking < RADIUS * tuning.clearance_radii);
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
    let goal_at = |x: f64| Goal::fixed(DVec3::new(x, 0.0, -20.0), arrival_radius);
    let hull = 5.0 + RADIUS;
    // Room for the full clearance.
    assert_eq!(
        clearance_toward(goal_at(30.0), &sphere),
        RADIUS * AvoidanceTuning::default().clearance_radii
    );
    // 1 unit of room between the hull and the goal's arrival circle.
    assert!((clearance_toward(goal_at(hull + arrival_radius + 1.0), &sphere) - 1.0).abs() < 1e-12);
    // A goal inside the hull leaves none.
    assert_eq!(clearance_toward(goal_at(hull - 1.0), &sphere), 0.0);
}

#[test]
fn the_clearance_leaves_the_goals_arrival_circle_at_least_two_radii_wide_outside_the_reach() {
    let sphere = rock(DVec3::new(0.0, 0.0, -20.0), DVec3::ZERO, 5.0);
    let hull = 5.0 + RADIUS;
    let narrow = |x: f64| Goal::fixed(DVec3::new(x, 0.0, -20.0), 0.1 * RADIUS);
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

/// The agent that is avoiding in the tests below is agent 0, and the other agent is this one.
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
const RESTING: Peer = Peer {
    steering: false,
    tracks: None,
};
/// A steering agent that tracks the agent `OWN`.
const TRACKING_OWN: Peer = Peer {
    steering: true,
    tracks: Some(ObstacleKey::Agent(OWN)),
};

/// The agent `OWN` flying 20 units per second along -z, wanted to fly `wanted_speed` that
/// way, avoiding an agent `OTHER` of kind `other` 40 units ahead of it (`behind` false) or
/// behind it, flying `other_speed` along -z. Returns the course and the wanted course.
fn near_an_agent(
    behind: bool,
    wanted_speed: f64,
    other_speed: f64,
    other: Peer,
) -> (Wanted, Wanted) {
    let mut body = body_at(DVec3::ZERO);
    body.velocity = DVec3::NEG_Z * 20.0;
    let wanted = wanted_toward(DVec3::NEG_Z, wanted_speed);
    let z = if behind { 40.0 } else { -40.0 };
    let neighbour = agent_sphere(DVec3::new(0.0, 0.0, z), DVec3::NEG_Z * other_speed, 5.0);
    let profile = profile_with(MAX_SPEED, UNLIMITED_DRAG);
    (
        avoid_among(&profile, &body, wanted, &[neighbour], other, None),
        wanted,
    )
}

#[test]
fn an_agent_that_tracks_this_one_and_lies_behind_is_left_out_of_its_scene() {
    // Braking from 20 to 5 closes the 40 to an agent behind that keeps its 20.
    let (tracker, wanted) = near_an_agent(true, 5.0, 20.0, TRACKING_OWN);
    assert_eq!(tracker, wanted);
    let (other, _) = near_an_agent(true, 5.0, 20.0, STEERING);
    assert_ne!(
        other, wanted,
        "an agent behind that isn't a tracker is a threat"
    );
}

#[test]
fn an_agent_that_tracks_this_one_and_lies_ahead_is_avoided_like_any_steering_agent() {
    // Flying 20 closes the 40 to an agent ahead that flies 5.
    let (tracker, wanted) = near_an_agent(false, 20.0, 5.0, TRACKING_OWN);
    let (other, _) = near_an_agent(false, 20.0, 5.0, STEERING);
    assert_ne!(tracker, wanted, "an agent ahead is a threat");
    assert_eq!(tracker, other);
}

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
    let (alone, _) = head_on_course(RESTING);
    let sideways = |course: Wanted| course.velocity.cross(wanted.velocity).length();
    assert!(sideways(alone) > 1.0, "the full detour turns the course");
    let ratio = sideways(shared) / sideways(alone);
    assert!((ratio - RECIPROCAL_SHARE).abs() < 1e-9, "ratio {ratio}");
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
    let full = closing(RESTING);
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
    let resting = Peers {
        own: OWN,
        tracked: None,
        all: &[STEERING, RESTING],
    };
    assert_eq!(resting.share_of(ObstacleKey::Agent(OTHER)), 1.0);
    // An agent the peers don't list is a plain sphere.
    assert_eq!(peers.share_of(ObstacleKey::Agent(AgentId::new(7))), 1.0);
}
