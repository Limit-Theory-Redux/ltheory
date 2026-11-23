use super::*;

// Cost zones.

const ZONE_START: Vec3 = [0.0; 3];
const ZONE_GOAL: Vec3 = [1.0e5, 0.0, 0.0];
const ZONE_RADIUS: f64 = 1.0e4;
/// A cost per metre that no detour of these tests is dearer than.
const DEAR: f64 = 1.0e3;

fn plan_with_zones(planner: &Planner, bodies: &[Body], zones: &[Zone]) -> Plan {
    planner.run_zoned(
        bodies,
        ZONE_START,
        SYSTEM_FRAME,
        ZONE_GOAL,
        SPEED,
        DEPTH,
        None,
        zones,
    )
}

fn path_length(plan: &Plan) -> f64 {
    plan.corners()
        .windows(2)
        .map(|leg| distance(leg[0], leg[1]))
        .sum()
}

/// How much longer the path is than the straight line to its last point.
fn detour_extra(plan: &Plan) -> f64 {
    path_length(plan) - distance(plan.start, *plan.points.last().unwrap())
}

/// The length of a straight line inside a sphere of `radius` whose centre is `offset` from it.
fn chord(radius: f64, offset: f64) -> f64 {
    2.0 * (radius * radius - offset * offset).sqrt()
}

/// The path keeps clear of the zone's sphere, a fixed one or one that rides on a body.
fn assert_clear_of_zone(plan: &Plan, bodies: &[Body], z: &Zone) {
    let ratio = min_sphere_ratio(plan, SPEED, z.radius, |t| zone_position(bodies, z, t));
    assert!(
        ratio >= CLEARANCE_TOLERANCE,
        "the path enters the zone to {ratio} of its radius"
    );
}

#[test]
fn a_costly_zone_across_the_line_is_gone_around_and_cleared() {
    let planner = Planner::new();
    let z = zone([5.0e4, 0.0, 0.0], ZONE_RADIUS, DEAR);
    let plan = plan_with_zones(&planner, &[], &[z]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1, "one waypoint round the zone");
    assert_eq!(plan.points.last(), Some(&ZONE_GOAL));
    assert_clear_of_zone(&plan, &[], &z);
    assert_times(&plan, SPEED);
}

#[test]
fn a_zone_that_costs_nothing_is_crossed() {
    let planner = Planner::new();
    let free = plan_with_zones(&planner, &[], &[zone([5.0e4, 0.0, 0.0], ZONE_RADIUS, 0.0)]);
    let bare = plan_with_zones(&planner, &[], &[]);
    assert_eq!(free.status, STATUS_OK);
    assert_eq!(free.points, bare.points);
    assert_eq!(free.times, bare.times);
}

#[test]
fn the_break_even_is_the_detours_extra_length_over_the_length_inside() {
    let planner = Planner::new();
    let centre = [5.0e4, 0.0, 0.0];
    let around = plan_with_zones(&planner, &[], &[zone(centre, ZONE_RADIUS, DEAR)]);
    assert_eq!(around.detours(), 1);
    // The leg goes through the zone's centre, so the length inside is the diameter.
    let break_even = detour_extra(&around) / chord(ZONE_RADIUS, 0.0);
    assert!(break_even > 0.0);
    let below = plan_with_zones(
        &planner,
        &[],
        &[zone(centre, ZONE_RADIUS, 0.9 * break_even)],
    );
    let above = plan_with_zones(
        &planner,
        &[],
        &[zone(centre, ZONE_RADIUS, 1.1 * break_even)],
    );
    assert_eq!(below.status, STATUS_OK);
    assert_eq!(below.detours(), 0, "crossed below the break-even");
    assert_eq!(above.detours(), 1, "gone around above it");
    assert_eq!(above.points, around.points);
}

#[test]
fn a_chord_that_grazes_a_zone_is_gone_around_at_a_cost_that_crosses_a_deep_one() {
    let planner = Planner::new();
    let deep = zone([5.0e4, 0.0, 0.0], ZONE_RADIUS, DEAR);
    let offset = 0.9 * ZONE_RADIUS;
    let graze = zone([5.0e4, 0.0, offset], ZONE_RADIUS, DEAR);
    let deep_ratio =
        detour_extra(&plan_with_zones(&planner, &[], &[deep])) / chord(ZONE_RADIUS, 0.0);
    let graze_ratio =
        detour_extra(&plan_with_zones(&planner, &[], &[graze])) / chord(ZONE_RADIUS, offset);
    assert!(
        graze_ratio < deep_ratio,
        "a shallow chord is the cheaper to avoid: {graze_ratio} against {deep_ratio}"
    );
    let cost = (graze_ratio * deep_ratio).sqrt();
    let deep_plan = plan_with_zones(&planner, &[], &[Zone { cost, ..deep }]);
    let graze_plan = plan_with_zones(&planner, &[], &[Zone { cost, ..graze }]);
    assert_eq!(deep_plan.detours(), 0, "the deep chord is crossed");
    assert_eq!(graze_plan.detours(), 1, "the shallow chord is gone around");
    assert_clear_of_zone(&graze_plan, &[], &Zone { cost, ..graze });
}

#[test]
fn a_zone_on_an_orbiting_body_is_passed_where_the_body_is_when_the_ship_gets_there() {
    let planner = Planner::new();
    // The body is on the line, half way, at the time the ship is (50 s in).
    let body = orbiting(-1, 5.0e4, -0.5, 0.01, 10.0);
    let z = Zone {
        frame: 0,
        centre: [0.0; 3],
        radius: 5.0e3,
        cost: DEAR,
    };
    let plan = plan_with_zones(&planner, &[body], &[z]);
    assert_eq!(plan.status, STATUS_OK);
    assert!(plan.detours() >= 1, "the zone is gone around");
    assert_eq!(
        plan.frames[0], 0,
        "the waypoint is in the zone's body's frame"
    );
    assert_clear_of_zone(&plan, &[body], &z);
    let ratio = min_clearance_ratio(&plan, &[body], 0, SPEED);
    assert!(ratio >= CLEARANCE_TOLERANCE, "the body itself: {ratio}");
}

#[test]
fn a_zone_that_holds_the_start_or_the_goal_is_ignored() {
    let planner = Planner::new();
    let bare = plan_with_zones(&planner, &[], &[]);
    let holds_start = zone([1.0e3, 0.0, 0.0], ZONE_RADIUS, DEAR);
    let holds_goal = zone([9.5e4, 0.0, 0.0], ZONE_RADIUS, DEAR);
    for held in [holds_start, holds_goal] {
        let plan = plan_with_zones(&planner, &[], &[held]);
        assert_eq!(plan.status, STATUS_OK);
        assert_eq!(plan.points, bare.points);
    }
    // A zone that holds the start does not hide a second one on the way.
    let both = plan_with_zones(
        &planner,
        &[],
        &[holds_start, zone([5.0e4, 0.0, 0.0], ZONE_RADIUS, DEAR)],
    );
    assert_eq!(both.detours(), 1);
}

#[test]
fn a_zone_is_crossed_at_the_depth_limit_where_a_body_is_a_failure() {
    let planner = Planner::new();
    let run = |bodies: &[Body], zones: &[Zone]| {
        planner.run_zoned(
            bodies,
            ZONE_START,
            SYSTEM_FRAME,
            ZONE_GOAL,
            SPEED,
            0,
            None,
            zones,
        )
    };
    let by_zone = run(&[], &[zone([5.0e4, 0.0, 0.0], ZONE_RADIUS, DEAR)]);
    assert_eq!(by_zone.status, STATUS_OK);
    assert_eq!(by_zone.detours(), 0);
    let by_body = run(&[fixed([5.0e4, 0.0, 0.0], 5.0e3, ZONE_RADIUS)], &[]);
    assert_eq!(by_body.status, STATUS_DEPTH_EXCEEDED);
}

#[test]
fn a_zone_that_cannot_be_gone_around_on_either_side_is_crossed() {
    let planner = Planner::new();
    let z = zone([5.0e4, 0.0, 0.0], ZONE_RADIUS, DEAR);
    let around = plan_with_zones(&planner, &[], &[z]);
    assert_eq!(around.detours(), 1);
    // A body holds the detour point on each side, so no leg to it can be planned.
    let w = around.points[0];
    let blockers = [
        fixed(w, 1.0e3, 4.0e3),
        fixed([w[0], w[1], -w[2]], 1.0e3, 4.0e3),
    ];
    let plan = plan_with_zones(&planner, &blockers, &[z]);
    assert_eq!(plan.status, STATUS_OK, "a zone never fails a plan");
    assert_eq!(plan.detours(), 0, "the zone is crossed");
    for i in 0..blockers.len() {
        let ratio = min_clearance_ratio(&plan, &blockers, i, SPEED);
        assert!(ratio >= CLEARANCE_TOLERANCE, "blocker {i}: {ratio}");
    }
}

#[test]
fn a_zone_around_a_body_is_gone_around_without_a_hull_hit() {
    let planner = Planner::new();
    let star = fixed([5.0e4, 0.0, 0.0], 1.0e3, 1.5e3);
    let z = zone([5.0e4, 0.0, 0.0], 8.0e3, DEAR);
    let plain = plan_with_zones(&planner, &[star], &[]);
    let plan = plan_with_zones(&planner, &[star], &[z]);
    assert_eq!((plain.status, plan.status), (STATUS_OK, STATUS_OK));
    assert_clear_of_zone(&plan, &[star], &z);
    assert!(
        detour_extra(&plan) > detour_extra(&plain),
        "the zone widens the way round the body"
    );
}

#[test]
fn a_zone_that_is_invalid_is_invalid_input() {
    let planner = Planner::new();
    let good = zone([5.0e4, 0.0, 0.0], ZONE_RADIUS, DEAR);
    let bad = [
        Zone {
            radius: 0.0,
            ..good
        },
        Zone {
            radius: -1.0,
            ..good
        },
        Zone {
            radius: f64::INFINITY,
            ..good
        },
        Zone { cost: -1.0, ..good },
        Zone {
            cost: f64::NAN,
            ..good
        },
        Zone {
            centre: [f64::NAN, 0.0, 0.0],
            ..good
        },
        Zone { frame: 0, ..good },
        Zone { frame: -2, ..good },
    ];
    for (i, z) in bad.iter().enumerate() {
        let plan = plan_with_zones(&planner, &[], &[*z]);
        assert_eq!(plan.status, STATUS_INVALID_INPUT, "zone {i}");
        assert!(plan.points.is_empty());
    }
    // A count without a buffer, and a buffer with a count of zero.
    let counted = planner.run_zone_buffer(
        &[],
        ZONE_START,
        SYSTEM_FRAME,
        ZONE_GOAL,
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        None,
        None,
        None,
        Some(1),
    );
    assert_eq!(counted.status, STATUS_INVALID_INPUT);
    let uncounted = planner.run_zone_buffer(
        &[],
        ZONE_START,
        SYSTEM_FRAME,
        ZONE_GOAL,
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        None,
        None,
        Some(&flatten_zones(&[good])),
        Some(0),
    );
    assert_eq!(uncounted.status, STATUS_OK);
    assert_eq!(uncounted.detours(), 0);
}

#[test]
fn a_zone_in_the_system_is_passed_inside_a_wells_frame_where_it_is_when_the_ship_gets_there() {
    const SAMPLES: usize = 2000;
    let planner = Planner::new();
    let planet = big_well_planet(500.0);
    // The ship starts inside the planet's well, so it is carried with the planet and flies the
    // whole plan as offsets from it. The zone stands in the system, where the planet is half way.
    let start_offset = [-1.0e5, 0.0, 0.0];
    let goal_offset = [1.0e5, 0.0, 0.0];
    let at_start = body_position(&[planet], 0, 0.0);
    let start = plus(at_start, start_offset);
    let halfway = 0.5 * distance(start_offset, goal_offset) / SPEED;
    let z = zone(body_position(&[planet], 0, halfway), ZONE_RADIUS, DEAR);
    let plan = planner.run_zoned(&[planet], start, 0, goal_offset, SPEED, DEPTH, None, &[z]);
    assert_eq!(plan.status, STATUS_OK);
    assert!(plan.detours() >= 1, "the zone is gone around");
    assert!(plan.frames.iter().all(|&frame| frame == 0));

    // The path is straight in the offsets from the planet; the zone is clear of it at every time.
    let mut corners = vec![start_offset];
    corners.extend(plan.offsets.iter().copied());
    let mut times = vec![0.0];
    times.extend(plan.times.iter().copied());
    let mut smallest = f64::INFINITY;
    for k in 0..corners.len() - 1 {
        for step in 0..=SAMPLES {
            let fraction = step as f64 / SAMPLES as f64;
            let offset = plus(
                corners[k],
                scaled(sub(corners[k + 1], corners[k]), fraction),
            );
            let t = times[k] + (times[k + 1] - times[k]) * fraction;
            let world = plus(body_position(&[planet], 0, t), offset);
            smallest = smallest.min(distance(world, z.centre) / z.radius);
        }
    }
    assert!(
        smallest >= CLEARANCE_TOLERANCE,
        "the path enters the zone to {smallest} of its radius"
    );
}

#[test]
fn no_zones_is_the_plan_without_the_argument() {
    let planner = Planner::new();
    let bodies = [
        fixed([3.0e4, 2.0e3, 0.0], 2.0e3, 3.0e3),
        orbiting(-1, 6.0e4, 0.0, 0.01, 1.0e3),
    ];
    let without = planner.plan(&bodies, ZONE_START, ZONE_GOAL);
    let empty = plan_with_zones(&planner, &bodies, &[]);
    assert_eq!(empty.status, without.status);
    assert_eq!(empty.offsets, without.offsets);
    assert_eq!(empty.frames, without.frames);
    assert_eq!(empty.times, without.times);
    assert_eq!(empty.radii, without.radii);
    assert_eq!(empty.speeds, without.speeds);
}
