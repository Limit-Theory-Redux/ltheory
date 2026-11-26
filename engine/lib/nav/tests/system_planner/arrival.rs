use super::*;

// ---- The arrival radius ----

/// How far before a driven leg's end the drive is released, which is where its flat tail starts.
fn drive_release() -> f64 {
    HANDOVER_MULT * BRAKING_LAG_MARGIN * (1.0 / DRIVE.ramp_down + 1.0 / SHIP.drag) * SPEED
}

/// Plans the one leg from the origin to `[length, 0, 0]` with the arrival radius `arrival`.
fn plan_leg_arriving(
    planner: &Planner,
    length: f64,
    drive: Option<Drive>,
    arrival: Option<f64>,
) -> Plan {
    planner.run_arrival(
        &[],
        [0.0; 3],
        SYSTEM_FRAME,
        [length, 0.0, 0.0],
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        drive,
        arrival,
    )
}

#[test]
fn the_goals_time_without_the_drive_is_when_the_ship_is_within_the_radius() {
    let planner = Planner::new();
    let (length, radius) = (5000.0, 800.0);
    let far = plan_leg_arriving(&planner, length, None, None);
    let near = plan_leg_arriving(&planner, length, None, Some(radius));
    assert_eq!((near.status, far.status), (STATUS_OK, STATUS_OK));
    assert_eq!(far.times, vec![length / SPEED]);
    assert_close(near.times[0], (length - radius) / SPEED, 1e-12, "time");
    assert_eq!(near.points, far.points, "the path does not change");
}

#[test]
fn a_radius_within_the_drives_flat_tail_cuts_the_time_by_its_flying_time() {
    let planner = Planner::new();
    let length = 1e8;
    let radius = 0.5 * drive_release();
    let far = plan_leg_arriving(&planner, length, Some(DRIVE), None);
    let near = plan_leg_arriving(&planner, length, Some(DRIVE), Some(radius));
    assert_eq!(far.drives, vec![true]);
    assert_eq!(near.drives, far.drives);
    assert_eq!(near.points, far.points);
    assert_close(far.times[0] - near.times[0], radius / SPEED, 1e-9, "cut");
}

#[test]
fn a_radius_beyond_the_flat_tail_cuts_into_the_braking_with_the_drive_on() {
    let planner = Planner::new();
    let length = 1e8;
    let radius = 3.0 * drive_release();
    let far = plan_leg_arriving(&planner, length, Some(DRIVE), None);
    let near = plan_leg_arriving(&planner, length, Some(DRIVE), Some(radius));
    assert_eq!((near.status, near.drives.clone()), (STATUS_OK, vec![true]));
    assert_eq!(near.points, far.points);
    assert_eq!(near.throughs, far.throughs);
    let cut = far.times[0] - near.times[0];
    // The braking stretch is flown faster than the planning speed, so it saves less than that
    // speed would over the radius, and the flat tail inside it saves its flying time.
    assert!(cut > drive_release() / SPEED, "cut {cut}");
    assert!(cut < radius / SPEED, "cut {cut}");
}

#[test]
fn a_radius_longer_than_the_last_leg_gives_the_time_the_leg_starts() {
    let planner = Planner::new();
    let bodies = vec![planet_with_zone([5e8, 0.0, 0.0])];
    let (start, goal) = ([0.0; 3], [1e9, 0.0, 0.0]);
    let run = |arrival| {
        planner.run_arrival(
            &bodies,
            start,
            SYSTEM_FRAME,
            goal,
            SPEED,
            SHIP,
            DEPTH,
            CAPACITY,
            Some(DRIVE),
            arrival,
        )
    };
    let far = run(None);
    assert_eq!(far.drives, vec![true, true]);
    let near = run(Some(1e12));
    assert_eq!(near.status, STATUS_OK);
    assert_eq!(near.points.len(), 2);
    assert_eq!(near.points, far.points);
    assert_eq!(near.times[0], far.times[0]);
    assert_eq!(
        near.times[1], near.times[0],
        "the goal is reached at the corner"
    );
}

#[test]
fn the_radius_cuts_the_framed_leg_of_a_run_and_leaves_the_entry() {
    let planner = Planner::new();
    let planet = standing_big_planet();
    let radius = 1.0e4;
    let run = |arrival| {
        planner.run_arrival(
            &[planet],
            RUN_START,
            0,
            RUN_GOAL_OFFSET,
            SPEED,
            SHIP,
            DEPTH,
            CAPACITY,
            Some(DRIVE),
            arrival,
        )
    };
    let far = run(None);
    let near = run(Some(radius));
    assert_eq!((near.status, far.status), (STATUS_OK, STATUS_OK));
    assert_eq!(far.throughs, vec![true, false]);
    assert_eq!(near.throughs, far.throughs);
    assert_eq!(near.points, far.points);
    assert_eq!(near.offsets, far.offsets);
    assert_eq!(near.drives, far.drives);
    assert_eq!(near.times[0], far.times[0], "the entry is not moved");
    let cut = far.times[1] - near.times[1];
    assert!(cut > 0.0 && cut <= radius / SPEED, "cut {cut}");
}

#[test]
fn a_negative_or_non_finite_radius_is_invalid_input() {
    let planner = Planner::new();
    for radius in [-1.0, f64::NAN, f64::INFINITY] {
        let plan = plan_leg_arriving(&planner, 5000.0, None, Some(radius));
        assert_eq!(plan.status, STATUS_INVALID_INPUT, "radius {radius}");
    }
    let plan = plan_leg_arriving(&planner, 5000.0, None, Some(0.0));
    assert_eq!(plan.status, STATUS_OK);
}
