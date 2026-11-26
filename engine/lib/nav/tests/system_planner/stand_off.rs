use super::*;

// ---- A goal with a stand-off ----

const STAND_OFF: f64 = 400.0;
const STAND_START: Vec3 = [-10000.0, 0.0, 0.0];
const STAND_CENTRE: Vec3 = [20000.0, 3000.0, 0.0];
/// A stand-off outside the planet's safe sphere and inside its well.
const WELL_STAND_OFF: f64 = 3.0 * PLANET_RADIUS;
/// The largest angle (radians) between the last leg and the line to the goal's centre.
const RADIAL_TOLERANCE: f64 = 1e-6;

/// A moon near the line from `STAND_START` to `STAND_CENTRE`, so a detour is needed.
fn stand_off_moon() -> Body {
    fixed([5000.0, 800.0, 0.0], 1000.0, 1500.0)
}

fn plan_with_stand_off(
    planner: &Planner,
    bodies: &[Body],
    start: Vec3,
    centre: Vec3,
    stand_off: Option<f64>,
) -> Plan {
    planner.run_all(
        bodies,
        start,
        SYSTEM_FRAME,
        centre,
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        None,
        None,
        None,
        None,
        stand_off,
    )
}

/// The angle between the last leg of `plan` and the line from its last corner to `centre`.
fn last_leg_angle_to(plan: &Plan, centre: Vec3) -> f64 {
    let corners = plan.corners();
    let n = corners.len();
    let leg = unit(sub(corners[n - 1], corners[n - 2]));
    let towards = unit(sub(centre, corners[n - 2]));
    dot(leg, towards).clamp(-1.0, 1.0).acos()
}

#[test]
fn the_last_leg_of_a_goal_with_a_stand_off_points_at_its_centre() {
    let planner = Planner::new();
    let bodies = [stand_off_moon()];

    let plan = plan_with_stand_off(
        &planner,
        &bodies,
        STAND_START,
        STAND_CENTRE,
        Some(STAND_OFF),
    );
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.points);
    assert!(plan.detours() >= 1, "the moon is gone around");
    let goal = *plan.points.last().unwrap();
    assert!(
        (distance(goal, STAND_CENTRE) - STAND_OFF).abs() < 1e-6,
        "the goal is the stand-off from the centre, not on it: {}",
        distance(goal, STAND_CENTRE)
    );
    assert!(
        last_leg_angle_to(&plan, STAND_CENTRE) < RADIAL_TOLERANCE,
        "the last leg points at the centre"
    );

    // The same flight to the stand-off point on the start line turns on the last leg.
    let along = unit(sub(STAND_CENTRE, STAND_START));
    let old_goal = sub(STAND_CENTRE, scaled(along, STAND_OFF));
    let old = plan_with_stand_off(&planner, &bodies, STAND_START, old_goal, None);
    assert_eq!(old.status, STATUS_OK);
    assert!(
        last_leg_angle_to(&old, STAND_CENTRE) > RADIAL_TOLERANCE.sqrt(),
        "the point goal leaves a last leg that does not point at the centre"
    );
    assert_eq!(
        old.detours(),
        plan.detours(),
        "the stand-off does not change the detours"
    );
}

#[test]
fn a_goal_with_a_stand_off_is_cut_along_a_clear_straight_line() {
    let planner = Planner::new();
    let plan = plan_with_stand_off(&planner, &[], STAND_START, STAND_CENTRE, Some(STAND_OFF));
    assert_eq!(plan.status, STATUS_OK);
    assert_eq!(plan.points.len(), 1);
    let expected = sub(
        STAND_CENTRE,
        scaled(unit(sub(STAND_CENTRE, STAND_START)), STAND_OFF),
    );
    assert!(distance(plan.points[0], expected) < 1e-6);
    // The goal is reached when the ship has flown the shortened leg.
    let flown = distance(STAND_START, expected);
    assert!((plan.times[0] - flown / SPEED).abs() <= GOAL_TIME_SLACK * flown / SPEED + 1e-6);
}

#[test]
fn a_goal_closer_than_its_stand_off_is_not_moved() {
    let planner = Planner::new();
    let start = [STAND_CENTRE[0] - 100.0, STAND_CENTRE[1], 0.0];
    let plan = plan_with_stand_off(&planner, &[], start, STAND_CENTRE, Some(STAND_OFF));
    assert_eq!(plan.status, STATUS_OK);
    assert!(distance(*plan.points.last().unwrap(), start) < 1e-6);
}

#[test]
fn no_stand_off_is_the_plan_without_the_argument() {
    let planner = Planner::new();
    let bodies = [stand_off_moon()];
    let without = plan_with_stand_off(&planner, &bodies, STAND_START, STAND_CENTRE, None);
    let zero = plan_with_stand_off(&planner, &bodies, STAND_START, STAND_CENTRE, Some(0.0));
    assert_eq!(zero.status, without.status);
    assert_eq!(zero.offsets, without.offsets);
    assert_eq!(zero.frames, without.frames);
    assert_eq!(zero.times, without.times);
    assert_eq!(zero.radii, without.radii);
    assert_eq!(zero.speeds, without.speeds);
}

#[test]
fn a_negative_or_infinite_stand_off_is_invalid_input() {
    let planner = Planner::new();
    for bad in [-1.0, f64::INFINITY, f64::NAN] {
        let plan = plan_with_stand_off(&planner, &[], STAND_START, STAND_CENTRE, Some(bad));
        assert_eq!(plan.status, STATUS_INVALID_INPUT, "stand-off {bad}");
    }
}

#[test]
fn a_goal_in_a_wells_frame_with_a_stand_off_is_reached_along_one_radial_line() {
    let planner = Planner::new();
    let planet = well_planet(3.0 * SPEED);
    let start = [-100000.0, 0.0, 50000.0];
    let plan = planner.run_all(
        &[planet],
        start,
        0,
        [0.0; 3],
        SPEED,
        SHIP,
        DEPTH,
        CAPACITY,
        Some(DRIVE),
        None,
        None,
        None,
        Some(WELL_STAND_OFF),
    );
    assert_eq!(plan.status, STATUS_OK, "{:?}", plan.points);
    let n = plan.frames.len();
    assert_eq!(plan.frames[n - 1], 0, "the goal rides on the planet");
    assert!(
        (dot(plan.offsets[n - 1], plan.offsets[n - 1]).sqrt() - WELL_STAND_OFF).abs() < 1e-6,
        "the goal is the stand-off from the planet's centre"
    );
    let e = entry_index(&plan).expect("the entry in the planet's well");
    // The entry and the goal lie on one line through the centre, on the same side of it.
    let cosine = dot(unit(plan.offsets[e]), unit(plan.offsets[n - 1]));
    assert!(cosine > 1.0 - RADIAL_TOLERANCE, "cosine {cosine}");
}

// ---- Moons in a row, the last one the goal ----

/// A moon's size, and the clearance and stand-off the game gives it (1.5 radii).
const ROW_MOON_RADIUS: f64 = 1000.0;
const ROW_MOON_SAFE: f64 = 1.5 * ROW_MOON_RADIUS;
/// The distance between neighbouring moons along the row.
const ROW_SPACING: f64 = 30000.0;
/// How far each moon lies off the row's axis, so that the detour round each is on a known side.
const ROW_ASIDE: [f64; 4] = [0.0, 200.0, -300.0, 0.0];
/// The frame of the goal moon, the last of the row.
const ROW_GOAL_FRAME: i32 = 3;

/// Four moons with a well each, `ROW_SPACING` apart along x.
fn moon_row() -> Vec<Body> {
    (0..ROW_ASIDE.len())
        .map(|i| {
            fixed(
                [ROW_SPACING * i as f64, 0.0, ROW_ASIDE[i]],
                ROW_MOON_RADIUS,
                ROW_MOON_SAFE,
            )
            .with_zone(WELL_RADIUS, WELL_ZONE_CAP)
        })
        .collect()
}

/// The waypoint at the entry radius in the frame of the goal moon.
fn row_entry_index(plan: &Plan) -> Option<usize> {
    plan.frames
        .iter()
        .zip(&plan.offsets)
        .position(|(frame, offset)| {
            *frame == ROW_GOAL_FRAME
                && (distance(*offset, [0.0; 3]) - WELL_RADIUS * WELL_ENTRY_SHARE).abs()
                    <= 1e-9 * WELL_RADIUS
        })
}

/// The angle between the legs `a`-`b` and `b`-`c`: the turn at `b`.
fn turn_at(a: Vec3, b: Vec3, c: Vec3) -> f64 {
    dot(unit(sub(b, a)), unit(sub(c, b)))
        .clamp(-1.0, 1.0)
        .acos()
}

#[test]
fn the_flight_along_a_row_of_moons_does_not_turn_at_the_goals_well_or_stand_off() {
    let planner = Planner::new();
    let bodies = moon_row();
    let first = bodies[0].position;
    let centre = bodies[ROW_GOAL_FRAME as usize].position;
    // The ship starts at the first moon's stand-off, on the side of the row.
    let start = plus(first, [ROW_MOON_SAFE, 0.0, 0.0]);
    for drive in [None, Some(DRIVE)] {
        let plan = planner.run_all(
            &bodies,
            start,
            ROW_GOAL_FRAME,
            [0.0; 3],
            SPEED,
            SHIP,
            DEPTH,
            CAPACITY,
            drive,
            None,
            None,
            None,
            Some(ROW_MOON_SAFE),
        );
        let label = format!("drive {}", drive.is_some());
        assert_eq!(plan.status, STATUS_OK, "{label}: {:?}", plan.points);
        let n = plan.points.len();
        assert_eq!(plan.frames[n - 1], ROW_GOAL_FRAME, "{label}");
        assert!(
            (distance(plan.points[n - 1], centre) - ROW_MOON_SAFE).abs() < 1e-6,
            "{label}: the goal is the stand-off from the centre"
        );
        let e = row_entry_index(&plan).expect("the entry in the goal moon's well");
        assert!(e >= 1, "{label}: the moons on the row are gone round");
        let corners = plan.corners();
        // The corners are the start first, so waypoint i is corners[i + 1]: the leg that ends at the
        // entry begins at corners[e].
        let (before, entry, goal) = (corners[e], corners[e + 1], corners[n]);
        assert!(
            turn_at(before, entry, goal) < RADIAL_TOLERANCE,
            "{label}: the turn at the entry is {}",
            turn_at(before, entry, goal)
        );
        assert!(
            last_leg_angle_to(&plan, centre) < RADIAL_TOLERANCE,
            "{label}: the last leg points at the centre"
        );
    }
}
