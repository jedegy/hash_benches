use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use criterion::{
    BenchmarkGroup, black_box, Criterion, criterion_group, criterion_main, Throughput,
};
use criterion::measurement::WallTime;
use crypto::digest::Digest as _;
use futures::executor::block_on;
use streebog::Digest as _;

/// Constant representing a kilobyte in bytes.
pub const KB: usize = 1024;

/// Constant representing a chunk size in bytes.
pub const CHUNK_SIZE: usize = 10 * KB;

/// Constant representing the number of tasks to use for processing.
pub const NUM_TASKS: usize = 16;

/// Constant representing the number of chunks to process in a task.
pub const NUM_CHUNKS_IN_TASK: usize = 1000;

/// Seed used for benchmark data generation.
const SEED: u128 = 0xDEADBEEFCAFEF00DC0DEFACE99C0FFEEu128;

/// Type alias for an asynchronous hash function.
type AsyncHashFn =
    Box<dyn Fn(Arc<Vec<Vec<u8>>>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Asynchronously computes the BLAKE3 hash for the provided data blocks.
///
/// # Arguments
/// * `data` - An `Arc` pointing to a vector of data chunks (each a `Vec<u8>`) to hash.
async fn blake3_async(data: Arc<Vec<Vec<u8>>>) {
    let mut hasher = blake3::Hasher::new();
    data.iter().for_each(|chunk| {
        hasher.update(chunk);
    });
    black_box(hasher.finalize());
}

/// Asynchronously computes the MD5 hash for the provided data blocks.
///
/// # Arguments
/// * `data` - An `Arc` pointing to a vector of data chunks (each a `Vec<u8>`) to hash.
async fn md5_async(data: Arc<Vec<Vec<u8>>>) {
    let mut hasher = crypto::md5::Md5::new();
    data.iter().for_each(|chunk| hasher.input(chunk));
    black_box(hasher.result_str());
}

/// Asynchronously computes the SHA-256 hash for the provided data blocks.
///
/// Uses the SHA-256 implementation from the `crypto` crate to process the data.
///
/// # Arguments
/// * `data` - An `Arc` pointing to a vector of data chunks (each a `Vec<u8>`) to hash.
async fn sha256_async(data: Arc<Vec<Vec<u8>>>) {
    let mut hasher = crypto::sha3::Sha3::sha3_256();
    data.iter().for_each(|chunk| hasher.input(chunk));
    black_box(hasher.result_str());
}

/// Asynchronously computes the SHA-512 hash for the provided data blocks.
///
/// Utilizes the SHA-512 variant from the `crypto` crate for hashing operations.
///
/// # Arguments
/// * `data` - An `Arc` pointing to a vector of data chunks (each a `Vec<u8>`) to hash.
async fn sha512_async(data: Arc<Vec<Vec<u8>>>) {
    let mut hasher = crypto::sha3::Sha3::sha3_512();
    data.iter().for_each(|chunk| hasher.input(chunk));
    black_box(hasher.result_str());
}

/// Asynchronously computes the Streebog-256 hash for the provided data blocks.
///
/// # Arguments
/// * `data` - An `Arc` pointing to a vector of data chunks (each a `Vec<u8>`) to hash.
async fn streebog256_async(data: Arc<Vec<Vec<u8>>>) {
    let mut hasher = streebog::Streebog256::new();
    data.iter().for_each(|chunk| hasher.update(chunk));
    black_box(hasher.finalize());
}

/// Asynchronously computes the Streebog-512 hash for the provided data blocks.
///
/// # Arguments
/// * `data` - An `Arc` pointing to a vector of data chunks (each a `Vec<u8>`) to hash.
async fn streebog512_async(data: Arc<Vec<Vec<u8>>>) {
    let mut hasher = streebog::Streebog512::new();
    data.iter().for_each(|chunk| hasher.update(chunk));
    black_box(hasher.finalize());
}

/// Run the benchmark for the specified hash function.
///
/// # Arguments
/// * `group` - A group of benchmarks.
/// * `name` - The name of the benchmark.
/// * `hash_fn` - The hash function to benchmark.
fn async_hasher_benchmark(group: &mut BenchmarkGroup<WallTime>, name: &str, hash_fn: AsyncHashFn) {
    let data_blocks = Arc::new(hash_benches::generate_data_blocks_for_async_benchmarking(
        NUM_TASKS,
        NUM_CHUNKS_IN_TASK,
        CHUNK_SIZE,
        Some(SEED),
    ));

    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function(name, |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::new(0, 0);

            for _ in 0..iters {
                let data_blocks = data_blocks.clone();
                rt.block_on(async {
                    // Spawn a tokio task for each data block
                    let tasks: Vec<_> = data_blocks
                        .iter()
                        .map(|data| {
                            let data = Arc::new(data.clone());
                            tokio::spawn(hash_fn(data))
                        })
                        .collect();

                    let start = std::time::Instant::now();

                    // Wait for all tasks to complete
                    for task in tasks {
                        task.await.unwrap();
                    }

                    total_duration += start.elapsed();
                });
            }

            total_duration
        });
    });
}

/// Runs benchmarks for different hashing algorithms.
///
/// # Arguments
/// * `c` - Criterion benchmarking context.
fn run_benchmark(c: &mut Criterion) {
    // Benchmark the throughput of the various hashing algorithms
    let mut group = c.benchmark_group("hashers-throughput");
    group.throughput(Throughput::Bytes(
        (NUM_TASKS * NUM_CHUNKS_IN_TASK * CHUNK_SIZE) as u64,
    ));
    group.measurement_time(Duration::from_secs(10));

    let algorithms: Vec<(&str, AsyncHashFn)> = vec![
        ("BLAKE3", Box::new(|data| Box::pin(blake3_async(data)))),
        ("MD5", Box::new(|data| Box::pin(md5_async(data)))),
        ("SHA-256", Box::new(|data| Box::pin(sha256_async(data)))),
        ("SHA-512", Box::new(|data| Box::pin(sha512_async(data)))),
        (
            "STREEBOG-256",
            Box::new(|data| Box::pin(streebog256_async(data))),
        ),
        (
            "STREEBOG-512",
            Box::new(|data| Box::pin(streebog512_async(data))),
        ),
    ];

    for (hash_algo_name, fn_hash) in algorithms.into_iter() {
        // Measure the memory and CPU usage before running the benchmark
        let Ok(pre_stats) = block_on(hash_benches::system_info()) else {
            eprintln!("Failed to retrieve pre system information");
            return;
        };

        async_hasher_benchmark(
            &mut group,
            &format!(
                "{}: chunk size = {}, parallel tokio tasks = {}, num chunks in task = {}",
                hash_algo_name, CHUNK_SIZE, NUM_TASKS, NUM_CHUNKS_IN_TASK
            ),
            fn_hash,
        );

        // Measure the memory and CPU usage after running the benchmark
        let Ok(post_stats) = block_on(hash_benches::system_info()) else {
            eprintln!("Failed to retrieve post system information");
            return;
        };

        // Calculate CPU usage difference as a percentage and memory usage difference
        let cpu_usage = (post_stats.1 - pre_stats.1).get::<heim::units::ratio::percent>();
        let mem_usage = post_stats.0.saturating_sub(pre_stats.0);

        println!(
            "{}: Memory usage {}. CPU usage: {} %",
            hash_algo_name,
            hash_benches::bytes_to_human_readable(mem_usage),
            cpu_usage,
        );
    }
}

criterion_group!(benches, run_benchmark);
criterion_main!(benches);
