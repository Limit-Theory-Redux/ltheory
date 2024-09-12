use criterion::{black_box, criterion_group, criterion_main, Criterion};
use phx::engine::Worker;

pub fn worker_benchmark(c: &mut Criterion) {
    c.bench_function("worker 4 1000", |b| {
        b.iter(|| run_worker(black_box(4), black_box(1000)))
    });
}

fn run_worker(instances_count: usize, messages_count: usize) {
    println!("\nStart worker bench");

    let mut worker: Worker<String, String> =
        Worker::new_native("BenchWorker", instances_count, |in_data| in_data);

    let mut task_ids = vec![];
    for _ in 0..messages_count {
        let task_id = worker
            .send("TestMessage".into())
            .expect("Cannot send a message");
        task_ids.push(task_id);
    }

    // println!("Tasks in progress: {task_ids:?}");

    while !task_ids.is_empty() {
        let task_result = worker.recv().expect("Cannot receive a message");
        if let Some((task_id, _)) = task_result {
            task_ids.retain(|&id| id != task_id);
            // println!("Task {task_id} completed");
        }
    }

    println!("Finish worker bench");
}

criterion_group!(benches, worker_benchmark);
criterion_main!(benches);
