use super::*;

#[test]
fn a_clear_line_is_the_goal_alone() {
    let planner = Planner::new();
    let bodies = [fixed([0.0, 0.0, 9000.0], 500.0, 750.0)];
    let plan = planner.plan(&bodies, [-10000.0, 0.0, 0.0], [10000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.points, vec![[10000.0, 0.0, 0.0]]);
    assert!((plan.times[0] - 20.0).abs() < 1e-9);
}

#[test]
fn no_bodies_is_the_goal_alone() {
    let plan = Planner::new().plan(&[], [0.0; 3], [3000.0, 4000.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.points, vec![[3000.0, 4000.0, 0.0]]);
    assert!((plan.times[0] - 5.0).abs() < 1e-9);
}

#[test]
fn start_equal_to_goal_is_the_goal_alone() {
    let plan = Planner::new().plan(&[], [5.0; 3], [5.0; 3]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.points, vec![[5.0; 3]]);
    assert_eq!(plan.times, vec![0.0]);
}

#[test]
fn a_sphere_on_the_line_gets_one_symmetric_detour() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let start = [-10000.0, 0.0, 0.0];
    let goal = [10000.0, 0.0, 0.0];
    let plan = planner.plan(&bodies, start, goal);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    let waypoint = plan.points[0];
    assert!(waypoint[0].abs() < 1.0, "symmetric: {waypoint:?}");
    let off_line = (waypoint[1].powi(2) + waypoint[2].powi(2)).sqrt();
    assert!(off_line > 1500.0, "outside the safe sphere: {off_line}");
    assert_eq!(plan.points[1], goal);
    assert!(plan.times[1] > 20.0, "the detour is longer than the line");
    assert_times(&plan, SPEED);
    assert_clear(&plan, &bodies, SPEED);
}

#[test]
fn the_kernels_turn_speed_is_the_circle_speed_of_nav() {
    let planner = Planner::new();
    let share = turn_share(&planner);
    let turn_speed: Function = planner.kernel.get("turnSpeed").unwrap();
    for ship in [
        SHIP,
        Ship {
            accel: 17_600.0,
            drag: 2.0,
        },
        Ship {
            accel: 40.0,
            drag: 0.02,
        },
        // Without drag the top speed is infinite and the turn speed is `√(share · a · r)`.
        Ship {
            accel: 300.0,
            drag: 0.0,
        },
    ] {
        let profile = profile_of(ship);
        let top = if ship.drag > 0.0 {
            ship.accel / ship.drag
        } else {
            f64::INFINITY
        };
        // From a radius that the top speed caps to one that the thrust does.
        for radius in [0.0, 1.0, 30.0, 500.0, 4_000.0, 90_000.0, 5.0e6] {
            let kernel: f64 = turn_speed
                .call((ship.accel, ship.drag, radius, top))
                .unwrap();
            let expected = circle_speed(&profile, radius, share);
            assert!(
                (kernel - expected).abs() <= 1e-9 * (1.0 + expected),
                "radius {radius}, ship {} / {}: kernel {kernel}, nav {expected}",
                ship.accel,
                ship.drag
            );
        }
    }
}

#[test]
fn a_line_meets_a_sphere_where_it_first_crosses_it() {
    let planner = Planner::new();
    let sphere_entry: Function = planner.kernel.get("sphereEntry").unwrap();
    let entry = |a: [f64; 3], g: [f64; 3], radius: f64| -> [f64; 3] {
        let (x, y, z): (f64, f64, f64) = sphere_entry
            .call((a[0], a[1], a[2], g[0], g[1], g[2], radius))
            .unwrap();
        [x, y, z]
    };
    let close = |got: [f64; 3], want: [f64; 3]| {
        assert!(
            distance(got, want) < 1e-9,
            "the entry is at {got:?}, wanted {want:?}"
        );
    };

    // A line through the sphere is met on the side it comes from, not where it leaves.
    close(
        entry([-10.0, 0.0, 0.0], [10.0, 0.0, 0.0], 2.0),
        [-2.0, 0.0, 0.0],
    );
    // A goal inside the sphere: the line ends there, so it meets the sphere once, short of the goal.
    let a = [-10.0, 0.5, 0.2];
    let hit = entry(a, [0.0, 0.0, 0.0], 2.0);
    assert!((distance(hit, [0.0; 3]) - 2.0).abs() < 1e-9);
    // The point is on the line: its distance from `a` and from the goal add up to the line's length.
    assert!((distance(a, hit) + distance(hit, [0.0; 3]) - distance(a, [0.0; 3])).abs() < 1e-9);
    // A start already inside the sphere enters at once.
    close(
        entry([0.5, 0.0, 0.0], [10.0, 0.0, 0.0], 2.0),
        [0.5, 0.0, 0.0],
    );
    // A line that misses the sphere meets it at its closest approach.
    close(
        entry([-10.0, 3.0, 0.0], [10.0, 3.0, 0.0], 1.0),
        [0.0, 3.0, 0.0],
    );
}

#[test]
fn a_corner_gets_an_arc_the_ship_can_hold() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let plan = planner.plan(&bodies, [-10000.0, 0.0, 0.0], [10000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    let radius = plan.radii[0];
    assert!(radius > 0.0, "the detour corner is rounded");
    // The arc is passed at the speed `nav` holds a circle of that radius at, below the planning
    // speed here.
    let held = circle_speed(&profile_of(SHIP), radius, turn_share(&planner));
    assert!(held < SPEED, "the turn is the limit: {held}");
    assert!((plan.speeds[0] - held).abs() <= 1e-9 * held);
    // The goal is where the ship stops.
    assert_eq!((plan.radii[1], plan.speeds[1]), (0.0, 0.0));
    assert_times(&plan, SPEED);
    assert_clear(&plan, &bodies, SPEED);
}

#[test]
fn a_corner_that_barely_turns_is_passed_at_the_planning_speed() {
    let planner = Planner::new();
    // A sphere that the line only just enters: the detour is a shallow bend.
    let bodies = [fixed([0.0, 1400.0, 0.0], 1000.0, 1500.0)];
    let plan = planner.run(
        &bodies,
        [-10000.0, 0.0, 0.0],
        [10000.0, 0.0, 0.0],
        SPEED,
        DEPTH,
        CAPACITY,
    );
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    assert!(plan.radii[0] > 0.0 || plan.speeds[0] == SPEED);
    assert_clear(&plan, &bodies, SPEED);
}

#[test]
fn a_plan_without_a_detour_has_no_corner() {
    let plan = Planner::new().plan(&[], [0.0; 3], [3000.0, 4000.0, 0.0]);
    assert_eq!(plan.radii, vec![0.0]);
    assert_eq!(plan.speeds, vec![0.0]);
}

#[test]
fn a_ship_without_drag_gets_an_arc_without_a_knee() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let ship = Ship {
        accel: 300.0,
        drag: 0.0,
    };
    let plan = planner.run_framed(
        &bodies,
        [-10000.0, 0.0, 0.0],
        SYSTEM_FRAME,
        [10000.0, 0.0, 0.0],
        SPEED,
        ship,
        DEPTH,
        CAPACITY,
    );
    assert_eq!(plan.status, STATUS_OK);
    assert!(plan.radii[0] > 0.0);
    let held = circle_speed(&profile_of(ship), plan.radii[0], turn_share(&planner));
    assert!((plan.speeds[0] - held.min(SPEED)).abs() <= 1e-9 * SPEED);
    assert_clear(&plan, &bodies, SPEED);
}

const NARROW_START: Vec3 = [-3000.0, 0.0, 0.0];
const NARROW_GOAL: Vec3 = [3000.0, 0.0, 0.0];

/// A ship whose turns are wide: its full-size arc at the narrow detour's corner is larger than
/// the sphere's clearance from the corner.
const WIDE_TURNING_SHIP: Ship = Ship {
    accel: 5000.0,
    drag: 0.5,
};

/// One sphere of safe radius `safe` between a start and a goal, with one corner just outside it.
fn narrow_detour(planner: &Planner, safe: f64, ship: Ship) -> (Plan, Vec<Body>) {
    let bodies = vec![fixed([0.0; 3], safe / 1.5, safe)];
    let plan = planner.run_framed(
        &bodies,
        NARROW_START,
        SYSTEM_FRAME,
        NARROW_GOAL,
        SPEED,
        ship,
        DEPTH,
        CAPACITY,
    );
    (plan, bodies)
}

#[test]
fn an_arc_that_cuts_into_a_body_is_made_smaller() {
    let planner = Planner::new();
    let (plan, bodies) = narrow_detour(&planner, 600.0, WIDE_TURNING_SHIP);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    let full = unconstrained_radius(&plan, 0, turn_share(&planner), WIDE_TURNING_SHIP);
    assert!(plan.radii[0] > 0.0, "a smaller arc is clear");
    assert!(plan.radii[0] < full, "the full arc was blocked");
    assert_clear(&plan, &bodies, SPEED);
}

#[test]
fn a_corner_with_no_clear_arc_is_left_sharp() {
    let planner = Planner::new();
    let (plan, sphere) = narrow_detour(&planner, 600.0, SHIP);
    assert_eq!(plan.detours(), 1);
    // A body just inside the corner, as far from the legs as it is wide: the legs clear it by a
    // hair, and every arc, however small, passes closer.
    const INSIDE_DISTANCE: f64 = 50.0;
    const LEG_CLEARANCE: f64 = 1.005;
    let corner = plan.points[0];
    let to_start = unit(sub(NARROW_START, corner));
    let to_goal = unit(sub(NARROW_GOAL, corner));
    let inside = unit(plus(to_start, to_goal));
    let leg_distance = INSIDE_DISTANCE * (1.0 - dot(inside, to_start).powi(2)).sqrt();
    let blocker = fixed(
        plus(corner, scaled(inside, INSIDE_DISTANCE)),
        1.0,
        leg_distance / LEG_CLEARANCE,
    );
    let bodies = [sphere[0], blocker];
    let plan = planner.plan(&bodies, NARROW_START, NARROW_GOAL);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1, "the blocker leaves the legs alone");
    assert_eq!(
        (plan.radii[0], plan.speeds[0]),
        (0.0, 0.0),
        "the corner is sharp"
    );
    assert_clear(&plan, &bodies, SPEED);
}

#[test]
fn a_sphere_on_a_line_along_any_axis_is_cleared() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    for (start, goal) in [
        ([0.0, -9000.0, 0.0], [0.0, 12000.0, 0.0]),
        ([0.0, 0.0, 9000.0], [0.0, 0.0, -9000.0]),
        ([-5000.0, -5000.0, -5000.0], [5000.0, 5000.0, 5000.0]),
    ] {
        let plan = planner.plan(&bodies, start, goal);
        assert_eq!(plan.status, STATUS_OK, "{start:?} to {goal:?}");
        assert_eq!(plan.detours(), 1);
        assert_times(&plan, SPEED);
        assert_clear(&plan, &bodies, SPEED);
    }
}

#[test]
fn a_line_that_only_grazes_the_sphere_is_still_cleared() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let plan = planner.plan(&bodies, [-10000.0, 1400.0, 0.0], [10000.0, 1400.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    assert_clear(&plan, &bodies, SPEED);
}

#[test]
fn a_body_on_the_line_now_that_will_have_left_costs_no_detour() {
    let planner = Planner::new();
    // On the line at the start, 10 km ahead; it circles off before the ship gets there.
    let moving = [orbiting(-1, 10000.0, 0.0, 0.1, 400.0)];
    let plan = planner.plan(&moving, [-20000.0, 0.0, 0.0], [20000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.points.len(), 1, "no detour: {:?}", plan.points);
    assert_clear(&plan, &moving, SPEED);

    // The same body held still where it is now is in the way.
    let still = [fixed([10000.0, 0.0, 0.0], 400.0, 600.0)];
    let plan = planner.plan(&still, [-20000.0, 0.0, 0.0], [20000.0, 0.0, 0.0]);
    assert_eq!(plan.detours(), 1);
}

#[test]
fn a_body_off_the_line_now_that_will_be_on_it_costs_a_detour() {
    let planner = Planner::new();
    // A moon of a small planet beside the line: it crosses the origin 20 s from now, when the
    // ship (starting at x = -20 km at 1 km/s) is there.
    let angular_speed = 0.02;
    let planet = [-10000.0, 0.0, 8000.0];
    let orbit_radius = (10000.0f64.powi(2) + 8000.0f64.powi(2)).sqrt();
    let phase = (-8000.0f64).atan2(10000.0) - angular_speed * 20.0;
    let moon = orbiting(0, orbit_radius, phase, angular_speed, 600.0);
    let system = [fixed(planet, 10.0, 10.0), moon];
    let start = [-20000.0, 0.0, 0.0];
    let goal = [20000.0, 0.0, 0.0];
    let plan = planner.plan(&system, start, goal);
    assert_eq!(plan.status, STATUS_OK);
    assert!(plan.detours() >= 1, "the moon is in the way at t = 20");
    assert_times(&plan, SPEED);
    assert_clear(&plan, &system, SPEED);

    // Frozen at its position now, the moon is nowhere near the line: no detour.
    let mut frozen = system;
    frozen[1] = fixed(body_position(&system, 1, 0.0), 600.0, 900.0);
    let plan = planner.plan(&frozen, start, goal);
    assert_eq!(plan.points.len(), 1);
}

#[test]
fn a_moon_follows_a_moving_parent() {
    let planner = Planner::new();
    let system = [
        orbiting(-1, 20000.0, 0.5, 0.01, 1500.0),
        orbiting(0, 4000.0, 1.0, 0.2, 300.0),
    ];
    // Aim at the moon where it will be 40 s from now.
    let target = body_position(&system, 1, 40.0);
    let start = [target[0] - 40000.0, target[1], target[2]];
    let goal = [target[0] + 40000.0, target[1], target[2]];
    let plan = planner.plan(&system, start, goal);
    assert_eq!(plan.status, STATUS_OK);
    assert!(plan.detours() >= 1);
    assert_times(&plan, SPEED);
    assert_clear(&plan, &system, SPEED);
}

#[test]
fn an_endpoint_inside_a_safe_sphere_shrinks_it() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 2000.0)];

    // The start is inside the safe sphere (1.5 km from the centre) and outside the hull.
    let plan = planner.plan(&bodies, [-1500.0, 0.0, 0.0], [10000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    let shrunk = 1500.0 * 0.99 / bodies[0].safe;
    assert!(min_clearance_ratio(&plan, &bodies, 0, SPEED) >= shrunk * CLEARANCE_TOLERANCE);

    // The goal is inside it, on the far side.
    let plan = planner.plan(&bodies, [-10000.0, 0.0, 0.0], [1500.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    assert!(min_clearance_ratio(&plan, &bodies, 0, SPEED) >= shrunk * CLEARANCE_TOLERANCE);
}

#[test]
fn an_endpoint_inside_a_hull_is_an_error() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let plan = planner.plan(&bodies, [500.0, 0.0, 0.0], [10000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_ENDPOINT_INSIDE_BODY);
    assert!(plan.points.is_empty());
    let plan = planner.plan(&bodies, [-10000.0, 0.0, 0.0], [0.0, 999.0, 0.0]);
    assert_eq!(plan.status, STATUS_ENDPOINT_INSIDE_BODY);
}

#[test]
fn the_depth_limit_is_reported_and_the_leg_left_straight() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let plan = planner.run(
        &bodies,
        [-10000.0, 0.0, 0.0],
        [10000.0, 0.0, 0.0],
        SPEED,
        0,
        CAPACITY,
    );
    assert_eq!(plan.status, STATUS_DEPTH_EXCEEDED);
    assert_eq!(plan.points, vec![[10000.0, 0.0, 0.0]]);
}

#[test]
fn a_full_output_buffer_is_reported() {
    let planner = Planner::new();
    let bodies = [fixed([0.0; 3], 1000.0, 1500.0)];
    let plan = planner.run(
        &bodies,
        [-10000.0, 0.0, 0.0],
        [10000.0, 0.0, 0.0],
        SPEED,
        DEPTH,
        1,
    );
    assert_eq!(plan.status, STATUS_CAPACITY_EXCEEDED);
    assert_eq!(plan.points.len(), 1);
}

#[test]
fn a_body_that_arrives_on_the_goal_in_time_is_an_error() {
    let planner = Planner::new();
    // The goal is outside the body now, but the body is on it when the straight flight ends.
    let moon = orbiting(-1, 10000.0, 0.0, 0.05, 500.0);
    let goal = body_position(&[moon], 0, 20.0);
    let start = [goal[0] - 20000.0, goal[1], goal[2]];
    let plan = planner.plan(&[moon], start, goal);
    assert_eq!(plan.status, STATUS_ENDPOINT_INSIDE_BODY);
}

#[test]
fn a_body_that_crosses_the_leg_is_passed_behind() {
    let planner = Planner::new();
    // The body crosses the line at the origin going toward +z; the ship passes on the -z side.
    let moon = orbiting(-1, 10000.0, -0.05 * 20.0, 0.05, 600.0);
    let crossing = body_position(&[moon], 0, 20.0);
    let start = [crossing[0] - 20000.0, crossing[1], crossing[2]];
    let goal = [crossing[0] + 20000.0, crossing[1], crossing[2]];
    let plan = planner.plan(&[moon], start, goal);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    let waypoint = plan.points[0];
    let body_then = body_position(&[moon], 0, plan.times[0]);
    assert!(
        distance(waypoint, body_then) > moon.safe,
        "the waypoint is clear of the body when the ship gets there"
    );
    assert_clear(&plan, &[moon], SPEED);
}

#[test]
fn bad_input_is_rejected() {
    let planner = Planner::new();
    let good = [fixed([0.0, 0.0, 5000.0], 100.0, 150.0)];
    let start = [-1000.0, 0.0, 0.0];
    let goal = [1000.0, 0.0, 0.0];

    let rejected = |plan: Plan, what: &str| {
        assert_eq!(plan.status, STATUS_INVALID_INPUT, "{what}");
        assert!(plan.points.is_empty(), "{what}");
    };
    rejected(
        planner.run(&good, start, goal, 0.0, DEPTH, CAPACITY),
        "zero speed",
    );
    rejected(
        planner.run(&good, start, goal, SPEED, -1, CAPACITY),
        "negative depth",
    );
    rejected(
        planner.run(&good, start, goal, SPEED, DEPTH, 0),
        "no capacity",
    );
    rejected(planner.plan(&good, [f64::NAN, 0.0, 0.0], goal), "NaN start");
    rejected(
        planner.plan(&good, start, [0.0, f64::INFINITY, 0.0]),
        "infinite goal",
    );

    let mut bad = good;
    bad[0].safe = 50.0;
    rejected(
        planner.plan(&bad, start, goal),
        "safe radius under the radius",
    );
    let mut bad = good;
    bad[0].radius = 0.0;
    rejected(planner.plan(&bad, start, goal), "zero radius");
    let mut bad = good;
    bad[0].parent = 0;
    rejected(
        planner.plan(&bad, start, goal),
        "a body that is its own parent",
    );
    let mut bad = good;
    bad[0].phase = f64::NAN;
    rejected(planner.plan(&bad, start, goal), "NaN phase");
}

#[test]
fn a_detour_waypoint_is_in_the_frame_of_the_body_it_goes_around() {
    let planner = Planner::new();
    // The moon of the earlier test: off the line now, on it at t = 20.
    let angular_speed = 0.02;
    let planet = [-10000.0, 0.0, 8000.0];
    let orbit_radius = (10000.0f64.powi(2) + 8000.0f64.powi(2)).sqrt();
    let phase = (-8000.0f64).atan2(10000.0) - angular_speed * 20.0;
    let moon = orbiting(0, orbit_radius, phase, angular_speed, 600.0);
    let system = [fixed(planet, 10.0, 10.0), moon];
    let goal = [20000.0, 0.0, 0.0];
    let plan = planner.plan(&system, [-20000.0, 0.0, 0.0], goal);
    assert_eq!(plan.status, STATUS_OK);
    assert!(plan.detours() >= 1);

    let (goal_frame, detour_frames) = plan.frames.split_last().unwrap();
    assert_eq!(*goal_frame, SYSTEM_FRAME, "the goal keeps its frame");
    assert_eq!(*plan.offsets.last().unwrap(), goal);
    assert!(
        detour_frames.iter().all(|frame| *frame == 1),
        "{detour_frames:?}"
    );
    // Whenever the ship gets there, a point at this offset is as far from the body as planned.
    for offset in &plan.offsets[..plan.detours()] {
        let length = distance(*offset, [0.0; 3]);
        assert!(
            length >= moon.safe * CLEARANCE_TOLERANCE,
            "offset {length} against the safe radius {}",
            moon.safe
        );
    }
}

#[test]
fn a_goal_on_an_orbiting_body_keeps_its_offset_and_meets_the_body() {
    let planner = Planner::new();
    let moon = orbiting(-1, 10000.0, 0.0, 0.05, 600.0);
    let offset = [3000.0, 500.0, 0.0];
    let start = [-20000.0, 0.0, 0.0];
    let plan = planner.plan_framed(&[moon], start, 0, offset);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.frames.last(), Some(&0));
    assert_eq!(plan.offsets.last(), Some(&offset));
    let goal = *plan.points.last().unwrap();
    assert_ne!(
        goal,
        [offset[0] + 10000.0, offset[1], offset[2]],
        "the body has moved on from where it is now"
    );
    // The times are the path's length over the speed, to the goal where the body is on arrival.
    assert_times(&plan, SPEED);
    assert_clear(&plan, &[moon], SPEED);
}

#[test]
fn a_goal_on_a_static_body_is_the_body_plus_the_offset() {
    let planner = Planner::new();
    let system = [fixed([5000.0, 0.0, 0.0], 500.0, 750.0)];
    let plan = planner.plan_framed(&system, [-5000.0, 0.0, 0.0], 0, [0.0, 2000.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.points, vec![[5000.0, 2000.0, 0.0]]);
    assert_eq!(plan.frames, vec![0]);
    let expected = distance([-5000.0, 0.0, 0.0], [5000.0, 2000.0, 0.0]) / SPEED;
    assert!((plan.times[0] - expected).abs() < 1e-9);
}

#[test]
fn a_goal_in_the_system_frame_keeps_its_world_point() {
    let planner = Planner::new();
    let goal = [3000.0, 4000.0, 0.0];
    let plan = planner.plan_framed(&[], [0.0; 3], SYSTEM_FRAME, goal);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.frames, vec![SYSTEM_FRAME]);
    assert_eq!(plan.offsets, vec![goal]);
}

#[test]
fn a_detour_toward_a_moving_goal_is_planned_for_the_later_arrival() {
    let planner = Planner::new();
    // A sphere sits on the line to a goal beside a moon that circles it; the detour makes the
    // flight longer, so the moon is somewhere else when the ship gets there than at the
    // straight flight's end.
    let moon = orbiting(-1, 10000.0, -1.5, 0.05, 300.0);
    let system = [fixed([0.0; 3], 1000.0, 1500.0), moon];
    let start = [-20000.0, 0.0, 0.0];
    let plan = planner.plan_framed(&system, start, 1, [2000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.points);
    assert!(
        plan.detours() >= 1,
        "the sphere is in the way: {:?}",
        plan.points
    );
    let arrival = *plan.times.last().unwrap();
    let straight = distance(start, *plan.points.last().unwrap()) / SPEED;
    assert!(
        arrival > straight,
        "the detour is longer: {arrival} against {straight}"
    );
    assert_times(&plan, SPEED);
    assert_clear(&plan, &system, SPEED);
}

#[test]
fn a_goal_on_a_body_that_can_outrun_the_ship_is_an_error() {
    let planner = Planner::new();
    // The moon moves at 2 km/s along its orbit, the ship at 1 km/s.
    let moon = orbiting(-1, 10000.0, 0.0, 0.2, 300.0);
    let plan = planner.plan_framed(&[moon], [-20000.0, 0.0, 0.0], 0, [2000.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_GOAL_OUTRUN);
    assert!(plan.points.is_empty());
}
