use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use glam::DVec3;
use nav::{Body, Goal, Obstacle, Profile, World};

const AGENTS: usize = 100;
const OBSTACLES: usize = 60;
/// Half the side of the cube the agents and obstacles are spread over, in length units.
const SPREAD: f64 = 20_000.0;
const OBSTACLE_RADIUS: f64 = 400.0;
const AGENT_RADIUS: f64 = 25.0;
const PROFILE: Profile = Profile {
    max_speed: 8_800.0,
    linear_drag: 2.0,
};

/// A fixed pseudo-random sequence in `[-1, 1)`, so every run measures the same scene.
struct Sequence(u64);

impl Sequence {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    }

    fn point(&mut self) -> DVec3 {
        DVec3::new(self.next(), self.next(), self.next()) * SPREAD
    }
}

fn scene() -> World {
    let mut sequence = Sequence(1);
    let mut world = World::new();
    for _ in 0..AGENTS {
        let id = world.add_agent(PROFILE);
        world.set_body(
            id,
            Body {
                position: sequence.point(),
                velocity: sequence.point().normalize() * PROFILE.max_speed * 0.5,
                radius: AGENT_RADIUS,
                ..Body::AT_REST
            },
        );
        world.set_goal(
            id,
            Some(Goal {
                target: sequence.point(),
                arrival_radius: 100.0,
            }),
        );
    }
    let obstacles: Vec<Obstacle> = (0..OBSTACLES)
        .map(|_| Obstacle {
            position: sequence.point(),
            velocity: sequence.point().normalize() * 50.0,
            radius: OBSTACLE_RADIUS,
        })
        .collect();
    world.set_obstacles(&obstacles);
    world
}

/// Half the side of the square the dense scene's agents, goals and obstacles are spread over.
const DENSE_SPREAD: f64 = 1_500.0;
/// The dense scene's goals lie this far ahead of the agents, and its obstacles between them.
const DENSE_LENGTH: f64 = 10_000.0;

/// Every agent flies the same way through a field of obstacles lying on its course, so the
/// avoidance has a threat at almost every step.
fn dense_scene() -> World {
    let mut sequence = Sequence(2);
    let across =
        |sequence: &mut Sequence| DVec3::new(sequence.next(), sequence.next(), 0.0) * DENSE_SPREAD;
    let mut world = World::new();
    for _ in 0..AGENTS {
        let id = world.add_agent(PROFILE);
        world.set_body(
            id,
            Body {
                position: across(&mut sequence),
                velocity: DVec3::NEG_Z * PROFILE.max_speed * 0.5,
                radius: AGENT_RADIUS,
                ..Body::AT_REST
            },
        );
        world.set_goal(
            id,
            Some(Goal {
                target: across(&mut sequence) + DVec3::NEG_Z * DENSE_LENGTH,
                arrival_radius: 100.0,
            }),
        );
    }
    let obstacles: Vec<Obstacle> = (0..OBSTACLES)
        .map(|_| Obstacle {
            position: across(&mut sequence)
                + DVec3::NEG_Z * (DENSE_LENGTH * (0.1 + 0.8 * (sequence.next() + 1.0) / 2.0)),
            velocity: DVec3::ZERO,
            radius: OBSTACLE_RADIUS,
        })
        .collect();
    world.set_obstacles(&obstacles);
    world
}

fn step_benchmark(c: &mut Criterion) {
    let mut sparse = scene();
    c.bench_function("step 100 agents 60 obstacles", |b| {
        b.iter(|| {
            sparse.step();
            black_box(sparse.axes());
        })
    });
    let mut dense = dense_scene();
    c.bench_function("step 100 agents 60 obstacles dense", |b| {
        b.iter(|| {
            dense.step();
            black_box(dense.axes());
        })
    });
}

criterion_group!(benches, step_benchmark);
criterion_main!(benches);
