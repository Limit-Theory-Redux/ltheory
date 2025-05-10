use criterion::{Criterion, black_box, criterion_group, criterion_main};
use phx::engine::Worker;
use phx::system::TimeStamp;

const TASK_RESULT_TIMEOUT_MS: f64 = 500.0;

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

    let start = TimeStamp::now();
    while !task_ids.is_empty() && start.get_elapsed_ms() < TASK_RESULT_TIMEOUT_MS {
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
