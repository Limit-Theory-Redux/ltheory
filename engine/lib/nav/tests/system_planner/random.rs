use super::*;

#[test]
fn random_static_spheres_are_cleared_or_the_status_says_why() {
    const CASES: usize = 400;
    const EXTENT: f64 = 20000.0;
    let planner = Planner::new();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut tally = Tally::new(turn_share(&planner));
    while tally.cases < CASES {
        let bodies: Vec<Body> = (0..20)
            .map(|_| {
                let radius = rng.range(1000.0, 3000.0);
                fixed(rng.point(EXTENT), radius, radius * 1.5)
            })
            .collect();
        let start = rng.point(EXTENT);
        let goal = rng.point(EXTENT);
        if !clear_of_all(&bodies, start, 0.0) || !clear_of_all(&bodies, goal, 0.0) {
            continue;
        }
        let plan = planner.plan(&bodies, start, goal);
        assert!(
            matches!(
                plan.status,
                STATUS_OK | STATUS_DEPTH_EXCEEDED | STATUS_NO_DETOUR
            ),
            "status {}",
            plan.status
        );
        assert!(!plan.points.is_empty());
        assert_eq!(*plan.points.last().unwrap(), goal);
        assert_times(&plan, SPEED);
        if plan.status == STATUS_OK {
            assert_clear(&plan, &bodies, SPEED);
        }
        tally.record(&plan);
    }
    println!("static: {tally:?}");
    assert!(tally.ok * 10 >= CASES * 7, "most cases plan: {tally:?}");
    assert!(tally.detoured >= 40, "detours occur: {tally:?}");
    assert!(
        tally.multiple_detours >= 5,
        "nested detours occur: {tally:?}"
    );
}

#[test]
fn random_orbiting_systems_are_cleared_or_the_status_says_why() {
    const CASES: usize = 200;
    const EXTENT: f64 = 25000.0;
    let planner = Planner::new();
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    let mut tally = Tally::new(turn_share(&planner));
    while tally.cases < CASES {
        let bodies = random_orbiting_system(&mut rng);
        let start = rng.point(EXTENT);
        let goal = rng.point(EXTENT);
        let straight_time = distance(start, goal) / SPEED;
        if !clear_of_all(&bodies, start, 0.0) || !clear_of_all(&bodies, goal, straight_time) {
            continue;
        }
        let plan = planner.plan(&bodies, start, goal);
        assert!(!plan.points.is_empty());
        assert_times(&plan, SPEED);
        if plan.status == STATUS_OK {
            assert_clear(&plan, &bodies, SPEED);
        }
        tally.record(&plan);
    }
    println!("orbiting: {tally:?}");
    assert!(tally.ok * 10 >= CASES * 6, "most cases plan: {tally:?}");
    assert!(tally.detoured >= 20, "detours occur: {tally:?}");
}

#[test]
fn random_goals_on_moving_bodies_are_reached_or_the_status_says_why() {
    const CASES: usize = 200;
    const EXTENT: f64 = 25000.0;
    let planner = Planner::new();
    let mut rng = Rng(0x2F0B_3A55_91C4_7E6D);
    let mut tally = Tally::new(turn_share(&planner));
    let mut framed_detours = 0;
    while tally.cases < CASES {
        let bodies = random_orbiting_system(&mut rng);
        let start = rng.point(EXTENT);
        let frame = (rng.next() * bodies.len() as f64) as usize;
        // A point 2 to 4 safe radii from the frame body, in a random direction.
        let direction = rng.point(1.0);
        let norm = distance(direction, [0.0; 3]);
        if norm < 0.1 || !clear_of_all(&bodies, start, 0.0) {
            continue;
        }
        let reach = rng.range(2.0, 4.0) * bodies[frame].safe / norm;
        let offset = direction.map(|component| component * reach);

        let plan = planner.plan_framed(&bodies, start, frame as i32, offset);
        if plan.status == STATUS_ENDPOINT_INSIDE_BODY {
            continue; // another body holds the point
        }
        if plan.status != STATUS_GOAL_OUTRUN {
            assert_eq!(plan.frames.last(), Some(&(frame as i32)));
            assert_eq!(plan.offsets.last(), Some(&offset));
        }
        if plan.status != STATUS_GOAL_OUTRUN && plan.status != STATUS_GOAL_NOT_SETTLED {
            assert_times(&plan, SPEED);
        }
        // A goal inside another body's safe sphere shrinks that sphere for the plan, so only a
        // goal clear of every body is judged against the full radii.
        let goal_clear = plan.status == STATUS_OK
            && clear_of_all(
                &bodies,
                *plan.points.last().unwrap(),
                *plan.times.last().unwrap(),
            );
        if goal_clear {
            assert_clear(&plan, &bodies, SPEED);
            if plan.detours() >= 1 {
                framed_detours += 1;
            }
        }
        tally.record(&plan);
    }
    println!("framed goals: {tally:?}, with detours {framed_detours}");
    assert!(tally.ok * 10 >= CASES * 4, "many cases plan: {tally:?}");
    assert!(
        framed_detours >= 10,
        "detours to a moving goal occur: {tally:?}"
    );
    assert!(
        tally.goal_outrun >= 1,
        "a goal on a fast body occurs: {tally:?}"
    );
    // A detour that appears or disappears as the goal moves leaves no arrival time that holds,
    // which is reported; it stays a minority of the cases.
    assert!(
        tally.goal_not_settled * 5 <= CASES,
        "few goals do not settle: {tally:?}"
    );
}
