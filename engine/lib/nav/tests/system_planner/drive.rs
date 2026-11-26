use super::*;

// ---- The travel drive ----

#[test]
fn the_zone_cap_is_the_games() {
    let planner = Planner::new();
    let math = planner
        .vm
        .require("Shared.Helpers.TravelDriveMath")
        .expect("load the drive maths");
    let function: Function = math.get("zoneCap").unwrap();
    for distance in [0.0, 100.0, 1500.0, 4999.0, 5000.0, 9999.0, 10000.0, 1e6] {
        let lua: f64 = function.call((40000.0, 2000.0, 5000.0, distance)).unwrap();
        assert_eq!(lua, zone_cap(40000.0, 2000.0, 5000.0, distance));
    }
    let influence: f64 = math.get("INFLUENCE_RADIUS_FACTOR").unwrap();
    let handover: f64 = math.get("HANDOVER_MULT").unwrap();
    assert_eq!(
        (influence, handover),
        (INFLUENCE_RADIUS_FACTOR, HANDOVER_MULT)
    );
}

#[test]
fn a_leg_is_driven_when_that_is_sooner() {
    let planner = Planner::new();
    let braking_time = BRAKING_LAG_MARGIN * (1.0 / DRIVE.ramp_down + 1.0 / SHIP.drag);
    let shortest = SPEED * (DRIVE.charge + HANDOVER_MULT * braking_time);
    let start = [0.0; 3];
    for length in [1e3, 0.5 * shortest, 0.99 * shortest] {
        let plan = planner.plan_driven(&[], start, [length, 0.0, 0.0]);
        assert_eq!(plan.status, STATUS_OK);
        assert_eq!(plan.drives, vec![false], "length {length}");
        assert_eq!(plan.times, vec![length / SPEED]);
    }
    for length in [1.2 * shortest, 3.0 * shortest, 1e6, 1e8, 1e10] {
        let goal = [length, 0.0, 0.0];
        let plan = planner.plan_driven(&[], start, goal);
        assert_eq!(plan.status, STATUS_OK, "length {length}");
        assert_eq!(plan.drives, vec![true], "length {length}");
        assert!(plan.times[0] < length / SPEED);
        assert_close(
            plan.times[0],
            oracle_time(&[], start, goal),
            DRIVE_TIME_TOLERANCE,
            &format!("time of {length}"),
        );
    }
}

/// Plans the one leg from the origin to `[length, 0, 0]` under `drive`.
fn plan_leg_under(planner: &Planner, length: f64, drive: Drive) -> Plan {
    planner.run_driven(
        &[],
        [0.0; 3],
        SYSTEM_FRAME,
        [length, 0.0, 0.0],
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        Some(drive),
    )
}

#[test]
fn a_leg_shorter_than_the_minimum_is_not_driven() {
    let planner = Planner::new();
    let length = 1e8;
    let free = plan_leg_under(&planner, length, DRIVE);
    assert_eq!((free.status, free.drives.clone()), (STATUS_OK, vec![true]));

    // The same leg, with a minimum just above it: flown at the planning speed.
    let held = Drive {
        min_length: length * 1.001,
        ..DRIVE
    };
    let plan = plan_leg_under(&planner, length, held);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.drives, vec![false]);
    assert_eq!(plan.times, vec![length / SPEED]);

    // A minimum at the leg's own length still drives it.
    let exact = Drive {
        min_length: length,
        ..DRIVE
    };
    let plan = plan_leg_under(&planner, length, exact);
    assert_eq!(plan.drives, vec![true]);
    assert_eq!(plan.times, free.times);
}

#[test]
fn the_minimum_applies_to_each_leg_of_a_detour() {
    let planner = Planner::new();
    let bodies = vec![planet_with_zone([5e8, 0.0, 0.0])];
    let start = [0.0; 3];
    let goal = [1e9, 0.0, 0.0];
    let both = planner.plan_driven(&bodies, start, goal);
    assert_eq!(both.drives, vec![true, true]);
    // Each leg is a little over half the way; a minimum above that leaves both at normal speed.
    let long_legs = Drive {
        min_length: 0.6e9,
        ..DRIVE
    };
    let none = planner.run_driven(
        &bodies,
        start,
        SYSTEM_FRAME,
        goal,
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        Some(long_legs),
    );
    assert_eq!(none.status, STATUS_OK);
    assert_eq!(none.drives, vec![false, false]);
}

#[test]
fn a_bad_drive_minimum_is_invalid_input() {
    let planner = Planner::new();
    for (what, min_length) in [
        ("negative", -1.0),
        ("NaN", f64::NAN),
        ("infinite", f64::INFINITY),
    ] {
        let drive = Drive {
            min_length,
            ..DRIVE
        };
        let plan = plan_leg_under(&planner, 1e8, drive);
        assert_eq!(plan.status, STATUS_INVALID_INPUT, "{what}");
    }
}

#[test]
fn a_long_free_leg_gains_time_as_the_autopilots_eta_law_says() {
    let planner = Planner::new();
    let math = planner
        .vm
        .require("Shared.Helpers.TravelDriveMath")
        .expect("load the drive maths");
    let eta: Function = math.get("travelTime").unwrap();
    let law = |length: f64| -> f64 {
        eta.call((
            length,
            0.0,
            SPEED,
            DRIVE.ramp_down,
            SHIP.drag,
            DRIVE.open_cap,
            true,
        ))
        .unwrap()
    };
    let kernel = |length: f64| planner.plan_driven(&[], [0.0; 3], [length, 0.0, 0.0]).times[0];
    // Charging and the climb to the cap are the same for both lengths: the cruise between them
    // is what the law has to match.
    let (near, far) = (1e9, 3e9);
    assert_close(
        kernel(far) - kernel(near),
        law(far) - law(near),
        DRIVE_TIME_TOLERANCE,
        "extra cruise time",
    );
}

#[test]
fn a_leg_that_needs_too_many_steps_is_planned_without_the_drive() {
    let planner = Planner::new();
    // A drive that falls a million times a second is held to steps of microseconds while its
    // multiplier climbs, so the leg's timeline does not fit in the step limit.
    let jittery = Drive {
        ramp_down: 1e6,
        ..DRIVE
    };
    let length = 1e8;
    let plan = planner.run_driven(
        &[],
        [0.0; 3],
        SYSTEM_FRAME,
        [length, 0.0, 0.0],
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        Some(jittery),
    );
    assert_eq!(plan.status, STATUS_DRIVE_NOT_TIMED);
    assert_eq!(plan.drives, vec![false]);
    assert_eq!(plan.times, vec![length / SPEED]);
}

#[test]
fn a_leg_through_a_zone_is_slower_and_the_oracle_agrees() {
    let planner = Planner::new();
    let start = [0.0; 3];
    let goal = [1e9, 0.0, 0.0];
    let in_zone = vec![planet_with_zone([5e8, 0.0, 3e6])];
    let open = vec![fixed([5e8, 0.0, 3e6], 5e5, 1e6)];
    let slow = planner.plan_driven(&in_zone, start, goal);
    let free = planner.plan_driven(&open, start, goal);
    assert_eq!((slow.status, slow.detours()), (STATUS_OK, 0));
    assert_eq!(slow.drives, vec![true]);
    assert!(
        slow.times[0] > free.times[0] + 0.5,
        "{} against {}",
        slow.times[0],
        free.times[0]
    );
    assert_close(
        slow.times[0],
        oracle_time(&in_zone, start, goal),
        DRIVE_TIME_TOLERANCE,
        "time through the zone",
    );
    assert_close(
        free.times[0],
        oracle_time(&open, start, goal),
        DRIVE_TIME_TOLERANCE,
        "time past the body",
    );
}

#[test]
fn a_gentle_detour_keeps_the_drive_run() {
    let planner = Planner::new();
    let bodies = vec![planet_with_zone([5e8, 0.0, 0.0])];
    let plan = planner.plan_driven(&bodies, [0.0; 3], [1e9, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    assert_eq!(plan.drives, vec![true, true]);
    assert_eq!(plan.throughs, vec![true, false], "the corner keeps the run");
    assert_eq!(plan.radii[0], 0.0, "no arc under the drive");
    assert!(
        plan.speeds[0] > HANDOVER_MULT * SPEED,
        "passed at the run's speed: {}",
        plan.speeds[0]
    );
    // One charge and no release at the corner: the run's time is the chain of its legs, and it is
    // sooner than the two legs each charged and released.
    let corner = plan.points[0];
    let goal = plan.points[1];
    let to_goal = distance(corner, goal);
    let (_, state) = drive_run_leg(&bodies, plan.start, corner, 0.0, DRIVE, to_goal, None);
    let (run, _) = drive_run_leg(
        &bodies,
        corner,
        goal,
        plan.times[0],
        DRIVE,
        0.0,
        Some(state),
    );
    assert_close(
        plan.times[1],
        run.last().unwrap().0,
        DRIVE_TIME_TOLERANCE,
        "arrival",
    );
    let first = drive_leg(&bodies, plan.start, corner, 0.0, DRIVE);
    let second = drive_leg(&bodies, corner, goal, first.last().unwrap().0, DRIVE);
    assert!(
        plan.times[1] < second.last().unwrap().0,
        "{} against {} released",
        plan.times[1],
        second.last().unwrap().0
    );
}

#[test]
fn a_corner_whose_turn_does_not_fit_the_leg_after_it_is_released() {
    let planner = Planner::new();
    let bodies = vec![planet_with_zone([5e8, 0.0, 0.0])];
    // The goal is 3e6 past the planet's centre: the leg after the detour is too short for the
    // settling of a turn at the speed the run has there (the braking law slows it to about 3e5).
    let plan = planner.plan_driven(&bodies, [0.0; 3], [5e8 + 3e6, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    assert_eq!(plan.drives, vec![true, true], "both legs are driven");
    assert_eq!(plan.throughs, vec![false, false], "the corner is released");
    assert!(plan.radii[0] > 0.0, "and rounded as in normal flight");
}

/// The time constants a drive turn would need to settle if it did not depend on the turn: the
/// settle of a turn of about a right angle.
const TURN_INDEPENDENT_SETTLE: f64 = 4.6;

#[test]
fn a_slight_kink_keeps_the_run_where_a_settle_that_ignores_the_turn_would_not_fit() {
    let planner = Planner::new();
    // The planet's safe sphere is entered by a tenth of its radius, so the detour is a kink of a
    // few degrees; the goal is a few million past it, where the braking law holds the run's speed
    // to a few times the leg (the same legs as the sharper turn above, which is released).
    let bodies = vec![fixed([5e8, 0.0, 9e5], 5e5, 1e6).with_zone(5e7, 2000.0)];
    for tail in [3e6, 5e6, 8e6] {
        let plan = planner.plan_driven(&bodies, [0.0; 3], [5e8 + tail, 0.0, 0.0]);
        assert_eq!(plan.status, STATUS_OK);
        assert_eq!(plan.detours(), 1, "tail {tail}");
        assert_eq!(
            plan.throughs,
            vec![true, false],
            "tail {tail}: the corner keeps the run"
        );
        assert_eq!(plan.radii[0], 0.0, "no arc under the drive");
        let corner = plan.points[0];
        let before = unit(sub(corner, plan.start));
        let after = unit(sub(plan.points[1], corner));
        let turn = dot(before, after).clamp(-1.0, 1.0).acos();
        assert!(
            turn < 5.0_f64.to_radians(),
            "tail {tail}: a slight kink: {turn}"
        );
        // The case is the interesting one: the settle after the corner, were it the right angle's,
        // would be more than the share of the leg after it that a turn may take.
        let lead = plan.speeds[0] / SHIP.drag;
        let leg_after = distance(corner, plan.points[1]);
        assert!(
            lead * (TURN_INDEPENDENT_SETTLE - 1.0) > ARC_LEG_SHARE * leg_after,
            "tail {tail}: settle {} against room {}",
            lead * (TURN_INDEPENDENT_SETTLE - 1.0),
            ARC_LEG_SHARE * leg_after
        );
    }
}

#[test]
fn a_kink_is_kept_up_to_the_turn_whose_settle_fits_the_leg_after_it() {
    let planner = Planner::new();
    let tail = 3e6;
    let mut kept = Vec::new();
    // From a graze of the safe sphere to a deep cut: the turn grows as the body comes onto the line.
    for side in [9.5e5, 9e5, 8e5, 6e5, 3e5, 0.0] {
        let bodies = vec![fixed([5e8, 0.0, side], 5e5, 1e6).with_zone(5e7, 2000.0)];
        let plan = planner.plan_driven(&bodies, [0.0; 3], [5e8 + tail, 0.0, 0.0]);
        assert_eq!(plan.detours(), 1, "side {side}");
        kept.push(plan.throughs[0]);
    }
    assert!(kept[0], "a graze keeps the run: {kept:?}");
    assert!(!kept[kept.len() - 1], "a deep cut releases it: {kept:?}");
    // One switch from kept to released, and none back.
    assert_eq!(
        kept.windows(2).filter(|pair| pair[0] != pair[1]).count(),
        1,
        "{kept:?}"
    );
}

#[test]
fn a_corner_that_a_later_detour_leaves_too_tight_a_leg_after_it_is_released() {
    let planner = Planner::new();
    // A second planet just behind the first: the leg after the first detour is bent again soon, so
    // the first corner fits the straight leg to the goal but not the short one it gets.
    let bodies = vec![
        planet_with_zone([3e8, 0.0, 0.0]),
        fixed([3.04e8, 0.0, 1.5e6], 5e5, 1e6),
    ];
    let plan = planner.plan_driven(&bodies, [0.0; 3], [1e9, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert!(plan.detours() >= 2, "{:?}", plan.points);
    assert!(
        plan.throughs.iter().all(|through| !through),
        "{:?}",
        plan.throughs
    );
    assert_through_corners_hold(&plan, &bodies, "too tight");
    for i in 0..bodies.len() {
        assert!(min_driven_clearance_ratio(&plan, &bodies, i) >= CLEARANCE_TOLERANCE);
    }
}

/// The turn of corner `k` (a waypoint index), in radians.
fn corner_turn(plan: &Plan, k: usize) -> f64 {
    let corners = plan.corners();
    let before = unit(sub(corners[k + 1], corners[k]));
    let after = unit(sub(corners[k + 2], corners[k + 1]));
    dot(before, after).clamp(-1.0, 1.0).acos()
}

const MIN_ARC_ANGLE: f64 = 0.02;
const STALE_SAFE: f64 = 2e7;
const STALE_BODY_X: f64 = 1.0e8;
const STALE_BODY_Z: f64 = 6e6;
const STALE_SECOND_X: f64 = 1.25e8;

/// A big body a little off the line, near the start, and a smaller one behind it, placed so that
/// the second detour's waypoint lies on the ray through the first one: the first corner is judged
/// against the straight leg to the goal, which it fails, and ends up almost straight. `second_z`
/// and `second_safe` place the small body.
fn stale_turn_bodies(second_z: f64, second_safe: f64) -> Vec<Body> {
    vec![
        fixed(
            [STALE_BODY_X, 0.0, STALE_BODY_Z],
            0.5 * STALE_SAFE,
            STALE_SAFE,
        ),
        fixed(
            [STALE_SECOND_X, 0.0, second_z],
            0.5 * second_safe,
            second_safe,
        ),
    ]
}

#[test]
fn a_corner_released_on_a_stale_turn_keeps_the_run_when_the_finished_path_holds_it() {
    let planner = Planner::new();
    let bodies = stale_turn_bodies(-1.48e7, 5e6);
    // The big body alone bends the path by more than the drive turn keeps. It is near the start,
    // so the failure is the leg before the corner being short for the turn's lead, which no push
    // of the waypoint changes.
    let alone = planner.plan_driven(&bodies[..1], [0.0; 3], [1e9, 0.0, 0.0]);
    assert_eq!(alone.detours(), 1);
    assert!(!alone.throughs[0], "{:?}", alone.throughs);
    assert!(corner_turn(&alone, 0) > MIN_ARC_ANGLE);

    let plan = planner.plan_driven(&bodies, [0.0; 3], [1e9, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 2, "{:?}", plan.points);
    assert!(
        corner_turn(&plan, 0) < MIN_ARC_ANGLE,
        "the second detour straightens the first corner: {}",
        corner_turn(&plan, 0)
    );
    assert!(plan.throughs[0], "{:?}", plan.throughs);
    assert!(plan.drives[1], "{:?}", plan.drives);
    for i in 0..bodies.len() {
        assert!(min_driven_clearance_ratio(&plan, &bodies, i) >= CLEARANCE_TOLERANCE);
        assert!(min_drive_turn_clearance_ratio(&plan, &bodies, i) >= CLEARANCE_TOLERANCE);
    }
}

#[test]
fn a_corner_released_on_a_stale_turn_stays_released_when_the_finished_path_fails_it_too() {
    let planner = Planner::new();
    // A smaller second body bends the first corner by a large angle again.
    let bodies = stale_turn_bodies(-1.55e7, 2e6);
    let plan = planner.plan_driven(&bodies, [0.0; 3], [1e9, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 2, "{:?}", plan.points);
    assert!(corner_turn(&plan, 0) > MIN_ARC_ANGLE);
    assert!(!plan.throughs[0], "{:?}", plan.throughs);
    for i in 0..bodies.len() {
        assert!(min_driven_clearance_ratio(&plan, &bodies, i) >= CLEARANCE_TOLERANCE);
    }
}

/// A body off the line, far enough from both ends for a long leg before and after its detour, so
/// that a corner there fails the drive turn's clearance alone: the turn cuts the corner inward by
/// more than the detour's margin.
fn clearance_only_body() -> Body {
    fixed([1.6e8, 0.0, 6e6], 0.5 * STALE_SAFE, STALE_SAFE)
}

#[test]
fn a_corner_that_fails_only_on_clearance_is_kept_with_its_waypoint_pushed_out() {
    let planner = Planner::new();
    let bodies = vec![clearance_only_body()];
    let plan = planner.plan_driven(&bodies, [0.0; 3], [1e9, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1);
    assert!(corner_turn(&plan, 0) > MIN_ARC_ANGLE);
    assert!(plan.throughs[0], "{:?}", plan.throughs);
    assert!(min_drive_turn_clearance_ratio(&plan, &bodies, 0) >= CLEARANCE_TOLERANCE);
    assert!(min_driven_clearance_ratio(&plan, &bodies, 0) >= CLEARANCE_TOLERANCE);
}

#[test]
fn a_pushed_corner_that_another_body_blocks_is_released() {
    const GOAL: Vec3 = [1e9, 0.0, 0.0];
    // Time constants into the turn: far enough for the blocker to lie off the unpushed waypoint
    // and the legs, near enough to lie on the turn.
    const BLOCKER_TAU: f64 = 2.0;
    let planner = Planner::new();
    let kept = planner.plan_driven(&[clearance_only_body()], [0.0; 3], GOAL);
    assert!(kept.throughs[0], "{:?}", kept.throughs);
    // A small body on the pushed corner's turn path.
    let corners = kept.corners();
    let before = unit(sub(corners[1], corners[0]));
    let after = unit(sub(corners[2], corners[1]));
    let lead = kept.speeds[0] / SHIP.drag;
    let on_turn = plus(
        corners[1],
        plus(
            scaled(after, lead * (BLOCKER_TAU - 1.0 + (-BLOCKER_TAU).exp())),
            scaled(before, -lead * (-BLOCKER_TAU).exp()),
        ),
    );
    let reach = distance(on_turn, corners[1]);
    let bodies = vec![
        clearance_only_body(),
        fixed(on_turn, 0.01 * reach, 0.02 * reach),
    ];
    let plan = planner.plan_driven(&bodies, [0.0; 3], GOAL);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.detours(), 1, "{:?}", plan.points);
    assert!(
        !plan.throughs[0],
        "{:?} {:?} {:?}",
        plan.throughs, plan.points, kept.points
    );
    for i in 0..bodies.len() {
        assert!(min_driven_clearance_ratio(&plan, &bodies, i) >= CLEARANCE_TOLERANCE);
    }
}

#[test]
fn the_drive_turn_path_starts_a_lead_before_the_corner_and_meets_the_new_leg() {
    let planner = Planner::new();
    let lead_of: Function = planner.kernel.get("driveTurnLead").unwrap();
    let point_on: Function = planner.kernel.get("pointOnDriveTurn").unwrap();
    let (speed, drag) = (400.0, 2.0);
    let lead: f64 = lead_of.call((speed, drag)).unwrap();
    assert_eq!(lead, speed / drag);
    let corner = [10.0, -4.0, 7.0];
    let angle: f64 = 0.6;
    let before = [1.0, 0.0, 0.0];
    let after = [angle.cos(), angle.sin(), 0.0];
    let at = |tau: f64| -> Vec3 {
        let (x, y, z): (f64, f64, f64) = point_on
            .call((
                corner[0], corner[1], corner[2], before[0], before[1], before[2], after[0],
                after[1], after[2], lead, tau,
            ))
            .unwrap();
        [x, y, z]
    };
    let start = at(0.0);
    assert!(distance(start, sub(corner, scaled(before, lead))) < 1e-9);
    // Integrate the ship: the nose snaps to the new leg, and the velocity follows it under drag.
    let (mut position, mut velocity) = (start, scaled(before, speed));
    let dt = 1e-5;
    let target = scaled(after, speed);
    let mut tau = 0.0;
    for tau_check in [0.5, 1.0, 2.0, 4.6] {
        while tau < tau_check {
            position = plus(position, scaled(velocity, dt));
            velocity = plus(velocity, scaled(sub(target, velocity), drag * dt));
            tau += drag * dt;
        }
        let model = at(tau_check);
        assert!(
            distance(position, model) < 1e-3 * lead,
            "tau {tau_check}: {position:?} against {model:?}"
        );
    }
    // After the settling the ship is within a hair of the new leg's line.
    let offset = sub(position, corner);
    let across = sub(offset, scaled(after, dot(offset, after)));
    assert!(dot(offset, after) > 0.0);
    assert!(
        (across[0].powi(2) + across[1].powi(2) + across[2].powi(2)).sqrt() < 0.02 * lead,
        "{across:?}"
    );
}

#[test]
fn a_detour_is_planned_for_where_the_body_is_when_the_drive_gets_there() {
    const PERIOD: f64 = 200.0;
    const CROSSING: f64 = 6e8;
    let planner = Planner::new();
    let start = [0.0; 3];
    let goal = [1e9, 0.0, 0.0];
    // The ship under the drive is at the crossing point at this time.
    let timeline = drive_leg(&[], start, goal, 0.0, DRIVE);
    let k = timeline.partition_point(|&(_, s)| s < CROSSING);
    let arrival = timeline[k].0;
    let angular_speed = std::f64::consts::TAU / PERIOD;
    // A planet on a circle about [5e8, 0, 0] that is at [6e8, 0, 0], on the line, at that time. At
    // the time a ship flying at the planning speed would be there it is somewhere else.
    let mut planet = orbiting(-1, 1e8, -angular_speed * arrival, angular_speed, 1e6);
    planet.position = [5e8, 0.0, 0.0];
    planet.safe = 5e6;
    let bodies = vec![planet];
    let normal_arrival = CROSSING / SPEED;
    let there = body_position(&bodies, 0, normal_arrival);
    assert!(distance(there, [CROSSING, 0.0, 0.0]) > 10.0 * planet.safe);

    let plan = planner.plan_driven(&bodies, start, goal);
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(
        plan.detours(),
        1,
        "the planet is on the line when the drive gets there"
    );
    assert!(plan.drives.iter().all(|&driven| driven));
    let ratio = min_driven_clearance_ratio(&plan, &bodies, 0);
    assert!(ratio >= CLEARANCE_TOLERANCE, "clearance {ratio}");
}

#[test]
fn a_goal_on_a_slow_moving_body_settles_under_the_drive() {
    const CASES: usize = 30;
    let planner = Planner::new();
    let mut rng = Rng(0x00D0_17ED_600D_5EED);
    // A planet whose orbit speed, 400 game units per second, is below the planning speed.
    let planet = orbiting(-1, 2e8, 0.0, 2e-6, 1e6);
    let bodies = vec![planet];
    let offset = [4e6, 0.0, 0.0];
    let (mut settled, mut driven) = (0, 0);
    for _ in 0..CASES {
        let start = rng.point(1.5e9);
        if distance(start, body_position(&bodies, 0, 0.0)) < 1e8 {
            continue;
        }
        let plan = planner.run_driven(
            &bodies,
            start,
            0,
            offset,
            SPEED,
            SHIP,
            DEPTH,
            CAPACITY,
            Some(DRIVE),
        );
        assert!(
            matches!(plan.status, STATUS_OK | STATUS_GOAL_NOT_SETTLED),
            "status {}",
            plan.status
        );
        assert_eq!(plan.frames.last(), Some(&0));
        assert_eq!(plan.offsets.last(), Some(&offset));
        // The goal's world point at the time the plan arrives is the body then plus the offset.
        let arrival = *plan.times.last().unwrap();
        let there = plus(body_position(&bodies, 0, arrival), offset);
        assert!(
            distance(*plan.points.last().unwrap(), there)
                < 1e-6 * (1.0 + distance(there, [0.0; 3]))
        );
        if plan.status == STATUS_OK {
            settled += 1;
        }
        if plan.drives.iter().any(|&leg| leg) {
            driven += 1;
        }
    }
    println!("moving goal under the drive: settled {settled}, driven {driven}");
    assert!(settled >= CASES / 2, "most plans settle: {settled}");
    assert!(driven >= CASES / 2, "most plans drive: {driven}");
}

/// The time a plan's leg `k` starts: the previous waypoint's, zero for the first leg.
fn leg_start_time(plan: &Plan, k: usize) -> f64 {
    if k == 0 { 0.0 } else { plan.times[k - 1] }
}

/// The smallest ratio of distance to safe radius between the ship and body `i` along the corner
/// path, with each driven leg timed by the oracle and the others at the planning speed.
fn min_driven_clearance_ratio(plan: &Plan, bodies: &[Body], i: usize) -> f64 {
    let corners = plan.corners();
    let mut smallest = f64::INFINITY;
    let mut carried: Option<RunState> = None;
    for (k, leg) in corners.windows(2).enumerate() {
        let t0 = leg_start_time(plan, k);
        let length = distance(leg[0], leg[1]);
        let timeline = if plan.drives[k] {
            // The run goes on past this leg's end by the legs up to the first corner that ends it.
            let mut beyond = 0.0;
            let mut end = k;
            while plan.throughs[end] {
                beyond += distance(corners[end + 1], corners[end + 2]);
                end += 1;
            }
            let (samples, state) =
                drive_run_leg(bodies, leg[0], leg[1], t0, DRIVE, beyond, carried);
            carried = plan.throughs[k].then_some(state);
            samples
        } else {
            carried = None;
            vec![(t0, 0.0), (t0 + length / SPEED, length)]
        };
        let direction = unit(sub(leg[1], leg[0]));
        for &(t, s) in &timeline {
            let position = plus(leg[0], scaled(direction, s.min(length)));
            let ratio = distance(position, body_position(bodies, i, t)) / bodies[i].safe;
            smallest = smallest.min(ratio);
        }
    }
    smallest
}

/// The smallest ratio of distance to safe radius between the drive turns of the plan's kept corners
/// and body `i`: the path a ship takes that turns its nose to the new leg a lead before the corner,
/// written out again from its differential equation's solution, sampled along the turn's time.
fn min_drive_turn_clearance_ratio(plan: &Plan, bodies: &[Body], i: usize) -> f64 {
    const SAMPLES: usize = 200;
    let corners = plan.corners();
    let mut smallest = f64::INFINITY;
    for k in (0..plan.points.len()).filter(|&k| plan.throughs[k]) {
        let before = unit(sub(corners[k + 1], corners[k]));
        let after = unit(sub(corners[k + 2], corners[k + 1]));
        let lead = plan.speeds[k] / SHIP.drag;
        // The turn is modelled until it settles, which grows with the turn: past that the ship is
        // on the leg after the corner, whose own path the other checks cover.
        let settle = DRIVE_TURN_LEAD_TAUS
            .max((2.0 * (corner_turn(plan, k) / 2.0).sin() / DRIVE_TURN_TOLERANCE).ln());
        for j in 0..=SAMPLES {
            let tau = settle * j as f64 / SAMPLES as f64;
            let along = lead * (tau - 1.0 + (-tau).exp());
            let back = lead * (-tau).exp();
            let position = plus(
                corners[k + 1],
                plus(scaled(after, along), scaled(before, -back)),
            );
            let t = plan.times[k] + (tau - 1.0) / SHIP.drag;
            let ratio = distance(position, body_position(bodies, i, t)) / bodies[i].safe;
            smallest = smallest.min(ratio);
        }
    }
    smallest
}

/// How far a drive turn's velocity may be off the new heading when it counts as settled, as a share
/// of the run's speed: the kernel's `DRIVE_TURN_TOLERANCE`.
const DRIVE_TURN_TOLERANCE: f64 = 0.01;
/// The time constants a drive turn spends before its corner: the kernel's `DRIVE_TURN_LEAD_TAUS`.
const DRIVE_TURN_LEAD_TAUS: f64 = 1.0;
/// Slack for comparing a kernel decision with the same formula written out again.
const ROUNDING_SLACK: f64 = 1e-9;

/// Asserts that every corner the plan's drive run goes through holds the drive on the finished
/// path: the leg after it is driven, the run's speed there is not slowed below the hand-over by the
/// turn, the lead and the settling fit their legs, and the turn clears every body. The rules are
/// written out again from the plan's own legs, so a corner judged on other neighbours fails here.
fn assert_through_corners_hold(plan: &Plan, bodies: &[Body], what: &str) {
    let corners = plan.corners();
    for k in (0..plan.points.len()).filter(|&k| plan.throughs[k]) {
        assert!(
            plan.drives[k + 1],
            "{what}: corner {k}: the next leg is driven"
        );
        let turn = corner_turn(plan, k);
        if turn < MIN_ARC_ANGLE {
            continue;
        }
        let speed = plan.speeds[k];
        assert!(
            speed * (turn / 2.0).cos() >= HANDOVER_MULT * SPEED * (1.0 - ROUNDING_SLACK),
            "{what}: corner {k}: turn {turn} slows the run below the hand-over"
        );
        let lead = speed / SHIP.drag;
        let settle =
            DRIVE_TURN_LEAD_TAUS.max((2.0 * (turn / 2.0).sin() / DRIVE_TURN_TOLERANCE).ln());
        let before = distance(corners[k], corners[k + 1]);
        let after = distance(corners[k + 1], corners[k + 2]);
        assert!(
            lead <= ARC_LEG_SHARE * before * (1.0 + ROUNDING_SLACK),
            "{what}: corner {k}: lead {lead} against the leg before, {before}"
        );
        assert!(
            lead * (settle - DRIVE_TURN_LEAD_TAUS)
                <= ARC_LEG_SHARE * after * (1.0 + ROUNDING_SLACK),
            "{what}: corner {k}: settle {} against the leg after, {after}",
            lead * (settle - DRIVE_TURN_LEAD_TAUS)
        );
    }
    for i in 0..bodies.len() {
        let ratio = min_drive_turn_clearance_ratio(plan, bodies, i);
        assert!(
            ratio >= CLEARANCE_TOLERANCE,
            "{what}: body {i}: a drive turn enters the safe sphere to {ratio} of its radius"
        );
    }
}

/// Eight planets with zones on circles, a moon for each of the first two, scaled up so that a
/// leg is long enough to drive and a body crosses it in the seconds a drive flight takes.
fn drive_scale_system(rng: &mut Rng) -> Vec<Body> {
    const SCALE: f64 = 1e4;
    let mut bodies = random_orbiting_system(rng);
    for body in &mut bodies {
        body.orbit_radius *= SCALE;
        body.radius *= SCALE;
        body.safe *= SCALE;
        // A moon's period is short next to a planet's; the planets' linear speed stays below the
        // drive's.
        let cap = if body.parent >= 0 { 200.0 } else { 2000.0 };
        *body = body.with_zone(body.safe * 3.0, cap);
    }
    bodies
}

#[test]
fn random_drive_plans_clear_the_bodies_at_the_times_the_drive_gives() {
    const CASES: usize = 60;
    const EXTENT: f64 = 2.5e8;
    let planner = Planner::new();
    let mut rng = Rng(0x5EED_D41E_0B1D_C0DE);
    let (mut cases, mut driven, mut driven_detours, mut ok, mut kept) = (0, 0, 0, 0, 0);
    let mut closest = f64::INFINITY;
    while cases < CASES {
        let bodies = drive_scale_system(&mut rng);
        let start = rng.point(EXTENT);
        let goal = rng.point(EXTENT);
        let straight_time = distance(start, goal) / DRIVE.open_cap / SPEED;
        if !clear_of_all(&bodies, start, 0.0) || !clear_of_all(&bodies, goal, straight_time) {
            continue;
        }
        cases += 1;
        let plan = planner.plan_driven(&bodies, start, goal);
        assert!(!plan.points.is_empty());
        if plan.status != STATUS_OK {
            continue;
        }
        ok += 1;
        if plan.drives.iter().any(|&driven| driven) {
            driven += 1;
            if plan.detours() >= 1 {
                driven_detours += 1;
            }
        }
        kept += plan.throughs.iter().filter(|&&through| through).count();
        assert_through_corners_hold(&plan, &bodies, &format!("case {cases}"));
        for i in 0..bodies.len() {
            let ratio = min_driven_clearance_ratio(&plan, &bodies, i);
            closest = closest.min(ratio);
            assert!(
                ratio >= CLEARANCE_TOLERANCE,
                "case {cases}, body {i}: path enters the safe sphere to {ratio} of its radius"
            );
            let turn_ratio = min_drive_turn_clearance_ratio(&plan, &bodies, i);
            assert!(
                turn_ratio >= CLEARANCE_TOLERANCE,
                "case {cases}, body {i}: a drive turn enters the safe sphere to {turn_ratio} of its radius"
            );
        }
    }
    println!(
        "drive: ok {ok}, driven {driven}, driven with detours {driven_detours}, kept corners {kept}, closest {closest}"
    );
    assert!(ok * 10 >= CASES * 5, "most cases plan");
    assert!(driven * 10 >= CASES * 5, "most plans use the drive");
    assert!(driven_detours >= 5, "detours under the drive occur");
    assert!(kept >= 3, "corners that keep the drive occur");
}

#[test]
fn random_chained_detours_keep_only_corners_that_hold_on_the_finished_path() {
    const CASES: usize = 200;
    const GOAL: Vec3 = [1e9, 0.0, 0.0];
    let planner = Planner::new();
    let mut rng = Rng(0xC0DE_CA11_5EED_0032);
    let (mut ok, mut chained, mut kept, mut kept_in_chains, mut released_in_chains) =
        (0, 0, 0, 0, 0);
    for case in 0..CASES {
        // Bodies strewn along the line to the goal, so that detours follow one another.
        let count = 3 + (rng.next() * 3.0) as usize;
        let bodies: Vec<Body> = (0..count)
            .map(|_| {
                let safe = rng.range(0.5 * STALE_SAFE, 1.5 * STALE_SAFE);
                let x = rng.range(1e8, 9e8);
                let z = rng.range(-0.8 * safe, 0.8 * safe);
                fixed([x, 0.0, z], 0.5 * safe, safe)
            })
            .collect();
        let plan = planner.plan_driven(&bodies, [0.0; 3], GOAL);
        if plan.status != STATUS_OK {
            continue;
        }
        ok += 1;
        let what = format!("case {case}");
        assert_through_corners_hold(&plan, &bodies, &what);
        for i in 0..bodies.len() {
            let ratio = min_driven_clearance_ratio(&plan, &bodies, i);
            assert!(
                ratio >= CLEARANCE_TOLERANCE,
                "{what}, body {i}: path enters the safe sphere to {ratio} of its radius"
            );
        }
        let through = plan.throughs.iter().filter(|&&through| through).count();
        kept += through;
        if plan.detours() >= 2 {
            chained += 1;
            kept_in_chains += through;
            released_in_chains += plan.detours() - through;
        }
    }
    println!(
        "chained: ok {ok}, chained {chained}, kept {kept}, kept in chains {kept_in_chains}, released in chains {released_in_chains}"
    );
    assert!(ok * 10 >= CASES * 5, "most cases plan");
    assert!(chained >= CASES / 2, "chained detours occur");
    assert!(kept_in_chains >= CASES / 4, "kept corners in chains occur");
    assert!(
        released_in_chains >= CASES / 4,
        "released corners in chains occur"
    );
}
