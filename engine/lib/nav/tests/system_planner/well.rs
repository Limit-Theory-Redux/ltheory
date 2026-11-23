use super::*;

// ---- The goal body's gravity well ----

#[test]
fn a_body_faster_than_the_ship_is_reached_through_an_entry_in_its_well() {
    let planner = Planner::new();
    let planet = well_planet(3.0 * SPEED);
    let offset = [3000.0, 0.0, 500.0];
    let start = [-100000.0, 0.0, 50000.0];

    // Without a well the same planet outruns the ship, which is the failure this fixes.
    let bare = Body {
        zone_radius: 0.0,
        zone_cap: 0.0,
        ..planet
    };
    let plan = plan_in_well(&planner, bare, start, offset, Some(DRIVE));
    assert_eq!(plan.status, STATUS_GOAL_OUTRUN);

    let plan = plan_in_well(&planner, planet, start, offset, Some(DRIVE));
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.points);
    let n = plan.frames.len();
    assert_eq!(plan.frames[n - 1], 0, "the goal rides on the planet");
    assert_eq!(plan.offsets[n - 1], offset, "the goal keeps its offset");
    let e = entry_index(&plan).expect("the entry, in the planet's frame at the entry radius");
    assert_eq!(e, 0, "the system part is one leg");
    assert_eq!(plan.radii[e], 0.0, "the entry corner is not rounded");
    assert!(plan.speeds[e] > 0.0, "and not passed at rest");
    assert!(plan.drives[e], "the long leg to the well is driven");
    assert!(
        !plan.throughs[e],
        "the part in the well is shorter than the release distance, so the drive is released"
    );
    assert!(plan.times[e] > 0.0 && plan.times[n - 1] > plan.times[e]);
    // The entry is further than the planning speed gets in its time: the drive reached it.
    assert!(
        distance(start, plan.points[e]) > SPEED * plan.times[e],
        "the entry is further than the planning speed gets in its time"
    );
}

#[test]
fn a_ship_inside_the_goals_well_is_planned_in_the_frame_whatever_the_bodys_speed() {
    let planner = Planner::new();
    // The planet moves ten times the ship's speed, which the ship in its well does not feel.
    let planet = well_planet(10.0 * SPEED);
    let centre = body_position(&[planet], 0, 0.0);
    let start_offset = [-6000.0, 0.0, 2000.0];
    let start = plus(centre, start_offset);
    // The line from the start to the goal passes within the planet's safe sphere.
    let offset = [4000.0, 0.0, 0.0];
    let plan = plan_in_well(&planner, planet, start, offset, None);
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.offsets);
    assert!(
        plan.frames.iter().all(|frame| *frame == 0),
        "{:?}",
        plan.frames
    );
    assert_eq!(plan.offsets.last(), Some(&offset));
    assert!(
        plan.detours() >= 1,
        "the planet is in the way: {:?}",
        plan.offsets
    );

    // The path is the same as for a planet that stands still: lengths, times and clearance are
    // those of the offsets.
    let mut corners = vec![start_offset];
    corners.extend(plan.offsets.iter().copied());
    let mut length = 0.0;
    for (k, time) in plan.times.iter().enumerate() {
        length += distance(corners[k], corners[k + 1]);
        assert_close(
            *time,
            length / SPEED,
            1e-6 * (1.0 + time) + GOAL_TIME_SLACK,
            "time",
        );
        assert!(
            segment_distance_to_origin(corners[k], corners[k + 1])
                >= planet.safe * CLEARANCE_TOLERANCE,
            "leg {k} cuts the planet's safe sphere"
        );
    }
}

#[test]
fn a_body_that_outruns_the_part_before_its_well_is_an_error() {
    let planner = Planner::new();
    let offset = [3000.0, 0.0, 500.0];
    let start = [-100000.0, 0.0, 50000.0];
    // Faster than the planning speed, with no drive to catch it.
    let plan = plan_in_well(&planner, well_planet(3.0 * SPEED), start, offset, None);
    assert_eq!(plan.status, STATUS_GOAL_OUTRUN);
    assert!(plan.points.is_empty());
    // Faster than the drive's top speed.
    let fast = well_planet(2.0 * DRIVE.open_cap * SPEED);
    let plan = plan_in_well(&planner, fast, start, offset, Some(DRIVE));
    assert_eq!(plan.status, STATUS_GOAL_OUTRUN);
    assert!(plan.points.is_empty());
}

#[test]
fn a_part_before_the_well_that_is_too_short_to_drive_cannot_catch_a_fast_body() {
    let planner = Planner::new();
    let never = Drive {
        min_length: 1e9,
        ..DRIVE
    };
    let planet = well_planet(5.0 * SPEED);
    let start = [-100000.0, 0.0, 50000.0];
    let offset = [3000.0, 0.0, 500.0];
    // The part before the well is flown at the planning speed, which the planet outruns: the
    // arrival at the entry does not settle, and the ship cannot close on it.
    let plan = plan_in_well(&planner, planet, start, offset, Some(never));
    assert_eq!(plan.status, STATUS_GOAL_OUTRUN, "{:?}", plan.offsets);
    assert!(plan.points.is_empty());
    // The drive that the same part is flown under catches it.
    let plan = plan_in_well(&planner, planet, start, offset, Some(DRIVE));
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.offsets);
}

#[test]
fn a_goal_outside_the_bodys_well_is_planned_as_before() {
    let planner = Planner::new();
    let start = [-100000.0, 0.0, 50000.0];
    // Beyond the well: nothing changes for a body that has one.
    let outside = [2.0 * WELL_RADIUS, 0.0, 0.0];
    let slow = well_planet(0.5 * SPEED);
    let bare = Body {
        zone_radius: 0.0,
        zone_cap: 0.0,
        ..slow
    };
    let with_well = plan_in_well(&planner, slow, start, outside, None);
    let without = plan_in_well(&planner, bare, start, outside, None);
    assert_eq!(with_well.status, STATUS_OK);
    assert_eq!(with_well.status, without.status);
    assert_eq!(with_well.frames, without.frames);
    assert_eq!(with_well.offsets, without.offsets);
    assert_eq!(with_well.times, without.times);
    // And a fast body still outruns the ship there.
    let plan = plan_in_well(
        &planner,
        well_planet(3.0 * SPEED),
        start,
        outside,
        Some(DRIVE),
    );
    assert_eq!(plan.status, STATUS_GOAL_OUTRUN);
}

#[test]
fn a_slow_body_with_a_well_gets_the_same_entry_without_the_drive() {
    let planner = Planner::new();
    let planet = well_planet(0.4 * SPEED);
    let offset = [3000.0, 0.0, 500.0];
    let plan = plan_in_well(&planner, planet, [-100000.0, 0.0, 50000.0], offset, None);
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.points);
    let n = plan.frames.len();
    assert_eq!(plan.offsets[n - 1], offset);
    let e = entry_index(&plan).expect("the entry");
    assert_eq!(e, 0, "the system part is one leg");
    // The system part is flown at the planning speed, to where the planet is on entering.
    assert_close(
        distance([-100000.0, 0.0, 50000.0], plan.points[e]) / SPEED,
        plan.times[e],
        1e-6 * plan.times[e],
        "entry time",
    );
    // The part in the well is a line of offsets, timed as one.
    let mut length = 0.0;
    for k in e + 1..n {
        length += distance(plan.offsets[k - 1], plan.offsets[k]);
    }
    assert_close(
        length / SPEED,
        plan.times[n - 1] - plan.times[e],
        1e-6 * plan.times[n - 1] + GOAL_TIME_SLACK,
        "time in the well",
    );
}

// ---- A drive run through the well's entry ----

/// The checks that a plan of `RUN_START` to `RUN_GOAL_OFFSET` under `drive` is one drive run, and
/// takes the time that one leg takes under the same drive.
fn assert_one_run_to_the_goal(drive: Drive) {
    let planner = Planner::new();
    let planet = standing_big_planet();
    let plan = plan_in_well(&planner, planet, RUN_START, RUN_GOAL_OFFSET, Some(drive));
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.offsets);
    let n = plan.frames.len();
    let e = entry_index_in(&plan, BIG_WELL_RADIUS).expect("the entry");
    assert_eq!((e, n), (0, 2), "a system leg and a framed leg");
    assert_eq!(
        plan.throughs,
        vec![true, false],
        "the run goes through the entry"
    );
    assert!(plan.drives[e] && plan.drives[n - 1], "both legs are driven");
    assert!(
        plan.speeds[e] > SPEED,
        "the entry is passed at the run's speed: {}",
        plan.speeds[e]
    );
    // One leg from the start to the goal, timed again from the oracle's own integration, takes the
    // time of the run. A second charge, or a release at the entry, would add much more.
    let along = drive_leg(&[planet], RUN_START, RUN_GOAL_OFFSET, 0.0, drive);
    assert_close(
        plan.times[n - 1],
        along.last().unwrap().0,
        DRIVE_TIME_TOLERANCE,
        "arrival",
    );
    let framed = plan.times[n - 1] - plan.times[e];
    let framed_length = distance(plan.offsets[e], RUN_GOAL_OFFSET);
    assert!(
        framed < 0.5 * framed_length / SPEED,
        "the framed leg is driven: {framed} s over {framed_length}"
    );
}

#[test]
fn a_drive_run_goes_on_through_the_entry_to_the_goal() {
    assert_one_run_to_the_goal(DRIVE);
}

#[test]
fn the_framed_leg_of_a_run_has_no_minimum_length_and_no_charge() {
    // The framed leg is about 90 000 long and the system leg is about 1.8 million.
    assert_one_run_to_the_goal(Drive {
        min_length: 5.0e5,
        ..DRIVE
    });
}

#[test]
fn a_detour_in_the_well_ends_the_run_at_the_entry() {
    let planner = Planner::new();
    // A rock on the line, inside the well and past the entry.
    let rock = fixed([-1.5e5, 0.0, 32632.0], 2000.0, 3000.0);
    let bodies = [standing_big_planet(), rock];
    let plan = planner.run_driven(
        &bodies,
        RUN_START,
        0,
        RUN_GOAL_OFFSET,
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        Some(DRIVE),
    );
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.offsets);
    assert!(
        plan.detours() >= 2,
        "the entry and a detour: {:?}",
        plan.offsets
    );
    let e = entry_index_in(&plan, BIG_WELL_RADIUS).expect("the entry");
    assert_eq!(e, 0, "the system part is one leg");
    // Only the entry ends the run: a detour corner round the rock may keep the drive.
    assert!(!plan.throughs[e], "{:?}", plan.throughs);
    assert_eq!(
        plan.speeds[e], SPEED,
        "the entry is passed at the planning speed"
    );
    // The system leg is released before the entry, as a leg that ends there always was.
    let released = drive_leg(&bodies, RUN_START, plan.points[e], 0.0, DRIVE);
    assert_close(
        plan.times[e],
        released.last().unwrap().0,
        DRIVE_TIME_TOLERANCE,
        "time at the entry",
    );
}

#[test]
fn a_run_starts_at_a_detour_corner_of_the_system_part_and_goes_through_the_entry() {
    let planner = Planner::new();
    // A rock half way along the system leg, a little off the line.
    let rock = fixed([-1.05e6, 1000.0, 2.6e5], 2000.0, 3000.0);
    let bodies = [standing_big_planet(), rock];
    let plan = planner.run_driven(
        &bodies,
        RUN_START,
        0,
        RUN_GOAL_OFFSET,
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        Some(DRIVE),
    );
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.offsets);
    let n = plan.frames.len();
    let e = entry_index_in(&plan, BIG_WELL_RADIUS).expect("the entry");
    assert_eq!(
        (e, n),
        (1, 3),
        "a detour, the entry and the goal: {:?}",
        plan.offsets
    );
    assert_eq!(
        plan.throughs,
        vec![true, true, false],
        "the run goes through the gentle detour corner and the entry"
    );
    assert!(
        plan.drives.iter().all(|drive| *drive),
        "every leg is driven"
    );
    assert!(
        plan.speeds[0] > SPEED && plan.speeds[e] > SPEED,
        "at the run's speed: {:?}",
        plan.speeds
    );
    // One run from the start: one charge, and the legs chained as the oracle flies them.
    let (corner, entry) = (plan.points[0], plan.points[e]);
    let to_goal = distance(entry, RUN_GOAL_OFFSET);
    let to_entry = distance(corner, entry);
    let (_, first) = drive_run_leg(
        &bodies,
        RUN_START,
        corner,
        0.0,
        DRIVE,
        to_entry + to_goal,
        None,
    );
    let (second, second_end) = drive_run_leg(
        &bodies,
        corner,
        entry,
        plan.times[0],
        DRIVE,
        to_goal,
        Some(first),
    );
    let (third, _) = drive_run_leg(
        &bodies,
        entry,
        RUN_GOAL_OFFSET,
        second.last().unwrap().0,
        DRIVE,
        0.0,
        Some(second_end),
    );
    assert_close(
        plan.times[n - 1],
        third.last().unwrap().0,
        DRIVE_TIME_TOLERANCE,
        "arrival",
    );
}

#[test]
fn a_plan_without_a_driven_system_leg_or_room_for_a_run_has_none() {
    let planner = Planner::new();
    let planet = standing_big_planet();
    let no_run = |plan: &Plan, what: &str| {
        assert_eq!(plan.status, STATUS_OK, "{what}: {:?}", plan.offsets);
        assert!(plan.throughs.iter().all(|through| !through), "{what}");
        let e = entry_index_in(plan, BIG_WELL_RADIUS).expect("the entry");
        assert_eq!(plan.speeds[e], SPEED, "{what}");
    };
    // No drive at all.
    let plan = plan_in_well(&planner, planet, RUN_START, RUN_GOAL_OFFSET, None);
    no_run(&plan, "no drive");
    // A system leg shorter than the drive's minimum is flown at the planning speed.
    let never = Drive {
        min_length: 1.0e9,
        ..DRIVE
    };
    let plan = plan_in_well(&planner, planet, RUN_START, RUN_GOAL_OFFSET, Some(never));
    no_run(&plan, "system leg below the minimum");
    // A goal so near the entry that the release distance would fall before it.
    let plan = plan_in_well(
        &planner,
        planet,
        RUN_START,
        [-1.8e5, 0.0, 3.0e4],
        Some(DRIVE),
    );
    no_run(&plan, "framed leg below the release distance");
}

#[test]
fn random_runs_through_wide_wells_hold_their_contract() {
    const CASES: usize = 150;
    const EXTENT: f64 = 3.0e6;
    let planner = Planner::new();
    let mut rng = Rng(0x2D6B_41C9_73E5_A810);
    let (mut planned, mut runs, mut driven_entries) = (0, 0, 0);
    let (mut outrun, mut not_settled) = (0, 0);
    for _ in 0..CASES {
        let planet = Body {
            phase: rng.range(0.0, std::f64::consts::TAU),
            ..big_well_planet(rng.range(0.2, 6.0) * SPEED)
        };
        let centre = body_position(&[planet], 0, 0.0);
        let start = rng.point(EXTENT);
        let direction = rng.point(1.0);
        let norm = distance(direction, [0.0; 3]);
        if norm < 0.1 || distance(start, centre) < 1.2 * BIG_WELL_RADIUS {
            continue;
        }
        let reach = rng.range(0.1, 0.8) * BIG_WELL_RADIUS / norm;
        let offset = direction.map(|component| component * reach);
        let plan = plan_in_well(&planner, planet, start, offset, Some(DRIVE));
        match plan.status {
            STATUS_OK => planned += 1,
            STATUS_GOAL_OUTRUN => {
                outrun += 1;
                continue;
            }
            // A run that is kept at one arrival time and dropped at the next would not settle.
            STATUS_GOAL_NOT_SETTLED => {
                not_settled += 1;
                continue;
            }
            other => panic!("status {other}: {:?}", plan.offsets),
        }
        let n = plan.frames.len();
        let entry = entry_index_in(&plan, BIG_WELL_RADIUS).expect("the entry");
        driven_entries += usize::from(plan.drives[entry]);
        for (k, through) in plan.throughs.iter().enumerate() {
            if !*through {
                continue;
            }
            runs += 1;
            assert_eq!(
                k, entry,
                "a run goes through the entry only: {:?}",
                plan.offsets
            );
            assert_eq!(
                k + 2,
                n,
                "and then is one leg to the goal: {:?}",
                plan.offsets
            );
            assert!(plan.drives[k] && plan.drives[k + 1], "both legs are driven");
            assert!(
                plan.speeds[k] > SPEED,
                "at the run's speed: {}",
                plan.speeds[k]
            );
        }
        assert!(!plan.throughs[n - 1], "the goal ends the run");
    }
    println!(
        "wide wells: planned {planned}, driven entries {driven_entries}, runs {runs}, \
         outrun {outrun}, not settled {not_settled}"
    );
    assert!(planned >= CASES / 3, "many cases plan: {planned}");
    assert!(runs >= planned / 4, "runs occur: {runs} of {planned}");
    assert!(
        not_settled * 10 <= planned,
        "few cases do not settle: {not_settled}"
    );
}

#[test]
fn random_goals_in_a_wells_are_reached_or_the_status_says_why() {
    const CASES: usize = 200;
    const EXTENT: f64 = 250000.0;
    let planner = Planner::new();
    let mut rng = Rng(0x7C21_90AE_5B33_F0D1);
    let (mut ok, mut inside, mut entered, mut outrun, mut not_settled) = (0, 0, 0, 0, 0);
    let mut cases = 0;
    while cases < CASES {
        let planet = Body {
            phase: rng.range(0.0, std::f64::consts::TAU),
            ..well_planet(rng.range(0.2, 6.0) * SPEED)
        };
        let centre = body_position(&[planet], 0, 0.0);
        // One start in three is inside the well.
        let start = if rng.next() < 1.0 / 3.0 {
            plus(centre, rng.point(0.5 * WELL_RADIUS))
        } else {
            rng.point(EXTENT)
        };
        let direction = rng.point(1.0);
        let norm = distance(direction, [0.0; 3]);
        let to_planet = distance(start, centre);
        if norm < 0.1 || to_planet < 2.0 * planet.safe {
            continue;
        }
        let reach = rng.range(2.0, 0.9 * WELL_RADIUS / planet.safe) * planet.safe / norm;
        let offset = direction.map(|component| component * reach);
        let drive = (rng.next() < 0.5).then_some(DRIVE);
        let plan = plan_in_well(&planner, planet, start, offset, drive);
        cases += 1;
        match plan.status {
            STATUS_OK => {
                ok += 1;
                let n = plan.frames.len();
                assert_eq!(plan.frames[n - 1], 0);
                assert_eq!(plan.offsets[n - 1], offset);
                assert!(
                    drive.is_some() || plan.throughs.iter().all(|through| !through),
                    "no run without a drive"
                );
                let started_inside = to_planet < WELL_RADIUS;
                if started_inside {
                    inside += 1;
                    assert!(plan.frames.iter().all(|frame| *frame == 0));
                } else {
                    entered += 1;
                    assert!(
                        entry_index(&plan).is_some(),
                        "the entry: {:?}",
                        plan.offsets
                    );
                }
                // Every leg after the entry, or from the start inside the well, keeps clear of
                // the planet in its frame.
                let first_framed = entry_index(&plan).map_or(0, |e| e + 1);
                for k in first_framed.max(1)..n {
                    assert!(
                        segment_distance_to_origin(plan.offsets[k - 1], plan.offsets[k])
                            >= planet.safe * CLEARANCE_TOLERANCE,
                        "leg {k} cuts the planet: {:?}",
                        plan.offsets
                    );
                }
            }
            STATUS_GOAL_OUTRUN => outrun += 1,
            STATUS_GOAL_NOT_SETTLED => not_settled += 1,
            other => panic!("status {other}: {:?}", plan.offsets),
        }
    }
    println!(
        "well goals: ok {ok} (inside {inside}, entered {entered}), outrun {outrun}, not settled {not_settled}"
    );
    assert!(ok * 10 >= CASES * 5, "many cases plan: {ok}");
    assert!(entered >= 20, "entries occur: {entered}");
    assert!(outrun >= 1, "a body that is too fast occurs");
    assert!(
        not_settled * 10 <= CASES,
        "few cases do not settle: {not_settled}"
    );
}

#[test]
fn a_goal_on_the_safe_sphere_of_its_body_is_reached_straight_whatever_the_rounding() {
    const DIRECTIONS: usize = 200;
    const SAFE: f64 = 150.0;
    const START_DISTANCE: f64 = 600.0;
    let planner = Planner::new();
    // The numbers of a ship flying to a planet's stand-off point, which is its safe radius: the
    // positions are large next to the offsets, so the rounding of the world positions differs from
    // that of the offsets.
    let ship = Ship {
        accel: 1.76,
        drag: 2.0,
    };
    let star = fixed([0.0; 3], 1.0, 2.0);
    // A planet that outruns the ship, with a well that holds the start, and a slow one with none.
    let fast = orbiting(0, 10000.0, 0.0, 1.76 / 10000.0, 100.0).with_zone(1000.0, 2000.0);
    let slow = orbiting(0, 10000.0, 0.0, 0.1 / 10000.0, 100.0);
    assert_eq!(fast.safe, SAFE);
    let mut rng = Rng(0x51A7_0E22_4D9B_C3F5);
    let mut cases = 0;
    while cases < DIRECTIONS {
        let direction = rng.point(1.0);
        if distance(direction, [0.0; 3]) < 0.1 {
            continue;
        }
        cases += 1;
        let direction = unit(direction);
        let goal = scaled(direction, SAFE);
        for planet in [fast, slow] {
            let system = [star, planet];
            let start = plus(
                body_position(&system, 1, 0.0),
                scaled(direction, START_DISTANCE),
            );
            let plan = planner.run_driven(&system, start, 1, goal, 0.88, ship, 5, 32, None);
            assert_eq!(plan.status, STATUS_OK, "direction {direction:?}");
            assert_eq!(plan.detours(), 0, "a straight line to the sphere's surface");
        }
    }
}

#[test]
fn a_frame_that_is_not_a_body_is_rejected() {
    let planner = Planner::new();
    let system = [fixed([0.0, 0.0, 5000.0], 100.0, 150.0)];
    for frame in [-2, 1, 7] {
        let plan = planner.plan_framed(&system, [0.0; 3], frame, [100.0, 0.0, 0.0]);
        assert_eq!(plan.status, STATUS_INVALID_INPUT, "frame {frame}");
        assert!(plan.points.is_empty(), "frame {frame}");
    }
    let plan = planner.plan_framed(&[], [0.0; 3], 0, [100.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_INVALID_INPUT, "a frame with no bodies");
}

#[test]
fn a_goal_offset_inside_the_frame_bodys_hull_is_an_error() {
    let planner = Planner::new();
    let system = [fixed([5000.0, 0.0, 0.0], 500.0, 750.0)];
    let plan = planner.plan_framed(&system, [-5000.0, 0.0, 0.0], 0, [100.0, 0.0, 0.0]);
    assert_eq!(plan.status, STATUS_ENDPOINT_INSIDE_BODY);
}
