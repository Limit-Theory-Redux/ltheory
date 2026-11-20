//! Rapier at ship scale: contacts, tunnelling and queries at manual-flight and
//! travel-drive speeds, for several `length_unit` values and CCD modes.
//!
//! Sizes and speeds are in game units (1 gu = 10 km), taken from the game's
//! hull table and travel-drive zone caps. Each approach is repeated at several
//! sub-frame phases, because whether a fast body is caught depends on where
//! the frame boundaries fall.

use super::test_fixture::{SHIP_MASS, STEP, sphere_body};
use super::*;

const PHASES: usize = 8;
const MAX_STEPS: usize = 12;
const RAM_STEPS: usize = 90;
const SHIP_DRAG: f32 = 2.0;

const SOLO_RADIUS: f64 = 0.003;
const LARGE_RADIUS: f64 = 0.018;
const STATION_RADIUS: f64 = 0.05;
const MOON_RADIUS: f64 = 174.0;
const PLANET_RADIUS: f64 = 637.1;

const MANUAL_MAX_SPEED: f64 = 0.88;
const LENGTH_UNITS: [f64; 3] = [1.0, 1e-2, 1e-4];

/// Manual flight, then the travel-drive zone caps (moon, planet, star, open space).
const SPEEDS: [(&str, f64); 6] = [
    ("manual", MANUAL_MAX_SPEED),
    ("manual x2", 2.0 * MANUAL_MAX_SPEED),
    ("moon", 200.0 * MANUAL_MAX_SPEED),
    ("planet", 2000.0 * MANUAL_MAX_SPEED),
    ("star", 8000.0 * MANUAL_MAX_SPEED),
    ("open", 40000.0 * MANUAL_MAX_SPEED),
];

struct Case {
    name: &'static str,
    ship_radius: f64,
    target_radius: f64,
    target_kinematic: bool,
}

const CASES: [Case; 4] = [
    Case {
        name: "solo-solo",
        ship_radius: SOLO_RADIUS,
        target_radius: SOLO_RADIUS,
        target_kinematic: false,
    },
    Case {
        name: "large-station",
        ship_radius: LARGE_RADIUS,
        target_radius: STATION_RADIUS,
        target_kinematic: true,
    },
    Case {
        name: "solo-moon",
        ship_radius: SOLO_RADIUS,
        target_radius: MOON_RADIUS,
        target_kinematic: true,
    },
    Case {
        name: "solo-planet",
        ship_radius: SOLO_RADIUS,
        target_radius: PLANET_RADIUS,
        target_kinematic: true,
    },
];

#[derive(Clone, Copy, PartialEq, Debug)]
enum Ccd {
    Off,
    /// Rapier's shape-cast CCD (`enable_ccd`).
    Full,
    /// Predictive contacts out to one frame of travel (`set_soft_ccd_prediction`).
    Soft,
}

const CCD_MODES: [Ccd; 3] = [Ccd::Off, Ccd::Full, Ccd::Soft];

/// Solver settings varied on top of the CCD mode.
#[derive(Clone, Copy)]
struct Tuning {
    /// Soft-CCD prediction as a multiple of one frame of travel.
    soft_scale: f64,
    contact_frequency: f64,
    solver_iterations: usize,
}

/// Rapier's defaults.
const DEFAULT_TUNING: Tuning = Tuning {
    soft_scale: 1.0,
    contact_frequency: 30.0,
    solver_iterations: 4,
};

#[derive(Clone, Copy)]
struct Setup {
    length_unit: f64,
    ccd: Ccd,
    tuning: Tuning,
}

impl Setup {
    fn new(length_unit: f64, ccd: Ccd) -> Setup {
        Setup {
            length_unit,
            ccd,
            tuning: DEFAULT_TUNING,
        }
    }
}

/// Worst case over the phases of one approach.
#[derive(Default)]
struct Outcome {
    /// Phases in which a contact pair was active at some step.
    contacted: usize,
    /// Phases in which the ship's centre ended up past the target's centre.
    tunnelled: usize,
    /// Deepest overlap of the two surfaces, in game units.
    penetration: f64,
    /// Largest surface gap left when contact first appeared, in game units.
    stop_gap: f64,
}

fn new_physics(setup: Setup) -> Physics {
    let mut physics = Physics::new();
    physics.set_length_unit(setup.length_unit);
    physics.set_contact_natural_frequency(setup.tuning.contact_frequency);
    physics.integration_parameters.num_solver_iterations = setup
        .tuning
        .solver_iterations
        .try_into()
        .expect("solver iterations are non-zero");
    physics
}

/// A moving ship and its target, in a world with the given setup. The ship
/// starts `distance` (centre to centre) before the target, which is at the origin.
fn approach_world(
    case: &Case,
    setup: Setup,
    speed: f64,
    distance: f64,
) -> (Physics, Box<RigidBody>, Box<RigidBody>) {
    let mut physics = new_physics(setup);
    let mut target = sphere_body(case.target_radius, case.target_kinematic);
    let mut ship = sphere_body(case.ship_radius, false);
    physics.add_rigid_body(&mut target);
    physics.add_rigid_body(&mut ship);
    ship.set_position(&Position::new(-distance, 0.0, 0.0));
    ship.set_linear_velocity(&na::Vector3::new(speed, 0.0, 0.0));
    match setup.ccd {
        Ccd::Off => {}
        Ccd::Full => ship.enable_ccd(),
        Ccd::Soft => {
            ship.set_soft_ccd_prediction(speed * f64::from(STEP) * setup.tuning.soft_scale)
        }
    }
    (physics, ship, target)
}

fn touching_distance(case: &Case) -> f64 {
    f64::from(case.ship_radius as f32) + f64::from(case.target_radius as f32)
}

impl Physics {
    fn has_active_contact(&self) -> bool {
        let world = self.world.as_ref();
        world
            .narrow_phase
            .contact_pairs()
            .any(|pair| pair.has_any_active_contact)
    }
}

/// One head-on approach at `speed`, starting a sub-frame `phase` away from the frame
/// boundary that would reach the target.
fn approach(case: &Case, speed: f64, setup: Setup, phase: f64) -> Outcome {
    let touching = touching_distance(case);
    let frame_travel = speed * f64::from(STEP);
    let (mut physics, ship, target) =
        approach_world(case, setup, speed, touching + frame_travel * (1.0 + phase));

    let mut outcome = Outcome::default();
    for _ in 0..MAX_STEPS {
        physics.update(STEP);
        let gap = ship.distance_to(&target) - touching;
        outcome.penetration = outcome.penetration.max(-gap);
        if outcome.contacted == 0 && physics.has_active_contact() {
            outcome.contacted = 1;
            outcome.stop_gap = gap.max(0.0);
        }
    }
    outcome.tunnelled = usize::from(ship.get_position().v.x > target.get_position().v.x);
    outcome
}

fn measure(case: &Case, speed: f64, setup: Setup) -> Outcome {
    let mut worst = Outcome::default();
    for i in 0..PHASES {
        let phase = i as f64 / PHASES as f64;
        let o = approach(case, speed, setup, phase);
        worst.contacted += o.contacted;
        worst.tunnelled += o.tunnelled;
        worst.penetration = worst.penetration.max(o.penetration);
        worst.stop_gap = worst.stop_gap.max(o.stop_gap);
    }
    worst
}

/// A ship that keeps thrusting into the target, flown like the game does: a constant
/// force and linear drag, so that `speed` is its free terminal speed. Returns the
/// deepest overlap over the second half of the run, once the contact has settled.
fn ram_penetration(case: &Case, speed: f64, setup: Setup) -> f64 {
    let touching = touching_distance(case);
    let (mut physics, mut ship, mut target) =
        approach_world(case, setup, speed, touching + speed * f64::from(STEP));
    ship.set_restitution(0.0);
    target.set_restitution(0.0);
    ship.set_drag(SHIP_DRAG, 0.0);
    let thrust = Vec3::new((speed * f64::from(SHIP_MASS * SHIP_DRAG)) as f32, 0.0, 0.0);

    let mut settled_penetration = 0.0_f64;
    for step in 0..RAM_STEPS {
        ship.apply_force(&thrust);
        physics.update(STEP);
        if step >= RAM_STEPS / 2 {
            settled_penetration = settled_penetration.max(touching - ship.distance_to(&target));
        }
    }
    settled_penetration
}

#[test]
#[ignore = "prints a table; run with --ignored --nocapture"]
fn report_approaches() {
    println!(
        "{:<14}{:<11}{:>7} {:<5}{:>9}{:>9}{:>12}{:>12}",
        "case", "speed", "unit", "ccd", "contact", "tunnel", "pen/r", "gap/r"
    );
    for case in &CASES {
        for (speed_name, speed) in SPEEDS {
            for length_unit in LENGTH_UNITS {
                for ccd in CCD_MODES {
                    let o = measure(case, speed, Setup::new(length_unit, ccd));
                    println!(
                        "{:<14}{:<11}{:>7.0e} {:<5}{:>6}/{PHASES}{:>6}/{PHASES}{:>12.3}{:>12.3}",
                        case.name,
                        speed_name,
                        length_unit,
                        format!("{ccd:?}"),
                        o.contacted,
                        o.tunnelled,
                        o.penetration / case.ship_radius,
                        o.stop_gap / case.ship_radius,
                    );
                }
            }
        }
    }
}

#[test]
#[ignore = "prints a table; run with --ignored --nocapture"]
fn report_tuning() {
    println!(
        "{:<14}{:<11}{:>7}{:>7}{:>7}{:>6}{:>10}{:>12}",
        "case", "speed", "unit", "scale", "freq", "iter", "pen/r", "ram pen/r"
    );
    for case in &CASES {
        for (speed_name, speed) in &SPEEDS {
            for length_unit in [1e-2, 1e-4] {
                for contact_frequency in [30.0, 120.0, 480.0, 1920.0] {
                    let tuning = Tuning {
                        contact_frequency,
                        ..DEFAULT_TUNING
                    };
                    let setup = Setup {
                        length_unit,
                        ccd: Ccd::Soft,
                        tuning,
                    };
                    let o = measure(case, *speed, setup);
                    let ram = ram_penetration(case, *speed, setup);
                    println!(
                        "{:<14}{:<11}{:>7.0e}{:>7}{:>7}{:>6}{:>10.4}{:>12.4}",
                        case.name,
                        speed_name,
                        length_unit,
                        tuning.soft_scale,
                        contact_frequency,
                        tuning.solver_iterations,
                        o.penetration / case.ship_radius,
                        ram / case.ship_radius
                    );
                }
            }
        }
    }
}

#[test]
#[ignore = "prints a table; run with --ignored --nocapture"]
fn report_ramming() {
    println!(
        "{:<14}{:>7} {:<5}{:>12}",
        "case", "unit", "ccd", "settled pen/r"
    );
    for case in CASES.iter().take(2) {
        for length_unit in LENGTH_UNITS {
            for ccd in CCD_MODES {
                let setup = Setup::new(length_unit, ccd);
                let pen = ram_penetration(case, MANUAL_MAX_SPEED, setup);
                println!(
                    "{:<14}{:>7.0e} {:<5}{:>12.4}",
                    case.name,
                    length_unit,
                    format!("{ccd:?}"),
                    pen / case.ship_radius
                );
            }
        }
    }
}

/// Soft CCD with a stiff contact: the settings that keep contacts shallow.
const RECOMMENDED: Tuning = Tuning {
    soft_scale: 1.0,
    contact_frequency: 1920.0,
    solver_iterations: 4,
};

/// Largest accepted overlap, as a fraction of the ship's radius.
const TOLERANCE: f64 = 0.05;
/// Largest accepted error of a cast, as a fraction of the ship's radius.
const CAST_TOLERANCE: f64 = 0.01;

const SHAPED_LENGTH_UNITS: [f64; 2] = [1e-2, 1e-4];

fn recommended(length_unit: f64) -> Setup {
    Setup {
        length_unit,
        ccd: Ccd::Soft,
        tuning: RECOMMENDED,
    }
}

#[test]
fn default_rapier_lets_ships_pass_through_each_other() {
    let o = measure(&CASES[0], MANUAL_MAX_SPEED, Setup::new(1.0, Ccd::Off));
    assert!(
        o.tunnelled > 0,
        "tunnelled in {} of {PHASES} phases",
        o.tunnelled
    );
    assert!(o.penetration > TOLERANCE * SOLO_RADIUS);
}

#[test]
fn full_ccd_stops_ships_passing_through_each_other() {
    for (name, speed) in SPEEDS {
        let o = measure(&CASES[0], speed, Setup::new(1.0, Ccd::Full));
        assert_eq!(o.tunnelled, 0, "{name}");
        assert_eq!(o.contacted, PHASES, "{name}");
    }
}

#[test]
fn full_ccd_does_not_limit_the_depth_reached_in_a_larger_body() {
    let o = measure(&CASES[3], MANUAL_MAX_SPEED, Setup::new(1.0, Ccd::Full));
    assert!(o.penetration > TOLERANCE * SOLO_RADIUS);
}

#[test]
fn soft_ccd_keeps_manual_flight_contacts_shallow() {
    for case in &CASES {
        for (name, speed) in &SPEEDS[..2] {
            for length_unit in SHAPED_LENGTH_UNITS {
                let o = measure(case, *speed, recommended(length_unit));
                let label = format!("{} {name} unit {length_unit:e}", case.name);
                assert_eq!(o.tunnelled, 0, "{label}");
                assert_eq!(o.contacted, PHASES, "{label}");
                assert!(o.penetration <= TOLERANCE * case.ship_radius, "{label}");
            }
        }
    }
}

#[test]
fn soft_ccd_keeps_thrusting_into_a_fixed_body_shallow() {
    for case in &CASES[1..3] {
        for length_unit in SHAPED_LENGTH_UNITS {
            let pen = ram_penetration(case, MANUAL_MAX_SPEED, recommended(length_unit));
            assert!(pen <= TOLERANCE * case.ship_radius, "{}", case.name);
        }
    }
}

#[test]
fn soft_ccd_does_not_tunnel_through_fixed_bodies_at_travel_speeds() {
    for case in &CASES[1..] {
        for (name, speed) in &SPEEDS[2..] {
            for length_unit in SHAPED_LENGTH_UNITS {
                let o = measure(case, *speed, recommended(length_unit));
                let label = format!("{} {name} unit {length_unit:e}", case.name);
                assert_eq!(o.tunnelled, 0, "{label}");
                assert_eq!(o.contacted, PHASES, "{label}");
            }
        }
    }
}

/// Distances from the origin at which a target is placed: the origin, 10 000 km,
/// 1 AU and 40 AU, in game units. The fraction keeps them off exactly representable
/// single-precision values.
const SYSTEM_DISTANCES: [f64; 4] = [0.0, 1e3 + 0.3, 1.496e7 + 0.3, 5.98e8 + 0.3];
/// Length of the casts, in game units (100 km).
const CAST_RANGE: f64 = 10.0;

/// A ship-sized sphere at `distance` along x, in a stepped world.
fn world_with_target_at(distance: f64) -> (Physics, Box<RigidBody>) {
    let mut physics = new_physics(recommended(1e-2));
    let mut target = sphere_body(SOLO_RADIUS, false);
    physics.add_rigid_body(&mut target);
    target.set_position(&Position::new(distance, 0.0, 0.0));
    physics.update(STEP);
    (physics, target)
}

#[test]
fn ray_cast_hits_within_a_fraction_of_a_ship_radius_at_system_distances() {
    let radius = f64::from(SOLO_RADIUS as f32);
    for distance in SYSTEM_DISTANCES {
        let (physics, _target) = world_with_target_at(distance);
        let ray = Ray {
            p: Position::new(distance - CAST_RANGE, 0.0, 0.0),
            dir: DVec3::X,
            t_min: 0.0,
            t_max: 2.0 * CAST_RANGE,
        };
        let hit = physics.ray_cast(&ray);
        assert!(!hit.body.is_null(), "no hit at {distance}");
        let error = (hit.pos.v.x - (distance - radius)).abs();
        assert!(
            error <= CAST_TOLERANCE * radius,
            "error {error} at {distance}"
        );
    }
}

#[test]
fn swept_sphere_time_of_impact_is_exact_at_system_distances() {
    let target_radius = f64::from(SOLO_RADIUS as f32);
    let cast_radius = f64::from(LARGE_RADIUS as f32);
    for distance in SYSTEM_DISTANCES {
        let (physics, _target) = world_with_target_at(distance);
        let hit = physics.sphere_sweep(
            &Position::new(distance - CAST_RANGE, 0.0, 0.0),
            &Position::new(distance + CAST_RANGE, 0.0, 0.0),
            cast_radius,
            None,
            CollisionMask::MAX,
        );
        assert!(!hit.body.is_null(), "no hit at {distance}");
        let expected = CAST_RANGE - target_radius - cast_radius;
        let error = (hit.t - expected).abs();
        assert!(
            error <= CAST_TOLERANCE * target_radius,
            "error {error} at {distance}"
        );
    }
}

#[test]
fn single_precision_positions_lose_ship_scale_at_system_distances() {
    let radius = f64::from(SOLO_RADIUS as f32);
    let error_at = |distance: f64| (f64::from(distance as f32) - distance).abs();
    assert!(error_at(SYSTEM_DISTANCES[1]) < TOLERANCE * radius);
    assert!(error_at(SYSTEM_DISTANCES[2]) > radius);
    assert!(error_at(SYSTEM_DISTANCES[3]) > radius);
}

/// Ships in a grid, each flying in a pseudo-random direction at `speed`.
// The bodies are boxed so their addresses stay stable while registered with the world.
#[expect(
    clippy::vec_box,
    reason = "bodies must not move once added to the world"
)]
fn crowd(count: usize, speed: f64, setup: Setup) -> (Physics, Vec<Box<RigidBody>>) {
    let mut physics = new_physics(setup);
    let mut state = 0x2545_f491_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % 2000) as f64 / 1000.0 - 1.0
    };
    let side = (count as f64).cbrt().ceil() as usize;
    let spacing = 10.0 * SOLO_RADIUS;
    let mut ships = Vec::with_capacity(count);
    for i in 0..count {
        let mut ship = sphere_body(SOLO_RADIUS, false);
        physics.add_rigid_body(&mut ship);
        let cell = |n: usize| (n as f64) * spacing;
        ship.set_position(&Position::new(
            cell(i % side),
            cell((i / side) % side),
            cell(i / (side * side)),
        ));
        let dir = na::Vector3::new(next(), next(), next()).normalize();
        ship.set_linear_velocity(&(dir * speed));
        match setup.ccd {
            Ccd::Off => {}
            Ccd::Full => ship.enable_ccd(),
            Ccd::Soft => ship.set_soft_ccd_prediction(speed * f64::from(STEP)),
        }
        ships.push(ship);
    }
    (physics, ships)
}

#[test]
#[ignore = "timing report; run with --release --ignored --nocapture"]
fn report_crowd_cost() {
    const SHIPS: usize = 100;
    const WARMUP: usize = 60;
    const TIMED: usize = 600;
    println!(
        "threads {}, profile {}",
        std::thread::available_parallelism().map_or(0, usize::from),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
    for (name, speed) in &SPEEDS[..3] {
        for ccd in CCD_MODES {
            let (mut physics, _ships) = crowd(SHIPS, *speed, recommended(1e-2).with_ccd(ccd));
            for _ in 0..WARMUP {
                physics.update(STEP);
            }
            let start = std::time::Instant::now();
            for _ in 0..TIMED {
                physics.update(STEP);
            }
            let per_step = start.elapsed().as_secs_f64() * 1e3 / TIMED as f64;
            println!("{name:<10} {ccd:<6?} {per_step:8.4} ms/step ({SHIPS} ships)");
        }
    }
}

impl Setup {
    fn with_ccd(self, ccd: Ccd) -> Setup {
        Setup { ccd, ..self }
    }
}
