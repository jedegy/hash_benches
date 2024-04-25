use std::fmt::Display;
use std::time::Duration;

use criterion::{BenchmarkId, black_box, Criterion, criterion_group, criterion_main, Throughput};
use crypto::digest::Digest as _;
use futures::executor::block_on;
use streebog::Digest as _;

/// Constant representing a kilobyte in bytes
const KB: usize = 1024;
/// Constant representing a megabyte in bytes
const MB: usize = 1024 * KB;
/// Seed used for benchmark data generation
const SEED: u128 = 0xDEADBEEFCAFEF00DC0DEFACE99C0FFEEu128;
/// Size of the data block used in the benchmarks
const BENCH_DATA_SIZE: usize = 40 * MB;

/// Enum representing the various cryptographic hashing algorithms.
enum Algorithm {
    BLAKE3,
    MD5,
    SHA256,
    SHA512,
    STREEBOG256,
    STREEBOG512,
}

impl Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Algorithm::BLAKE3 => "BLAKE3",
            Algorithm::MD5 => "MD5",
            Algorithm::SHA256 => "SHA3-256",
            Algorithm::SHA512 => "SHA3-512",
            Algorithm::STREEBOG256 => "STREEBOG-256",
            Algorithm::STREEBOG512 => "STREEBOG-512",
        };
        write!(f, "{}", str)
    }
}

/// Run the specified hashing algorithm on the provided data.
///
/// # Arguments
///
/// * `algo` - The hashing algorithm to use.
/// * `data` - The data to hash.
fn run_hashing_algorithm(algo: &Algorithm, data: &[u8]) {
    match algo {
        Algorithm::BLAKE3 => {
            black_box(blake3::hash(data));
        }
        Algorithm::MD5 => {
            let mut hasher = crypto::md5::Md5::new();
            hasher.input(data);
            black_box(hasher.result_str());
        }
        Algorithm::SHA256 => {
            let mut hasher = crypto::sha3::Sha3::sha3_256();
            hasher.input(data);
            black_box(hasher.result_str());
        }
        Algorithm::SHA512 => {
            let mut hasher = crypto::sha3::Sha3::sha3_512();
            hasher.input(data);
            black_box(hasher.result_str());
        }
        Algorithm::STREEBOG256 => {
            let mut hasher = streebog::Streebog256::new();
            hasher.update(data);
            black_box(hasher.finalize());
        }
        Algorithm::STREEBOG512 => {
            let mut hasher = streebog::Streebog512::new();
            hasher.update(data);
            black_box(hasher.finalize());
        }
    }
}

/// Main function for running the hashing algorithm benchmarks.
///
/// This function benchmarks the throughput of various hashing algorithms using the
/// generated data block.
///
/// # Arguments
///
/// * `c` - The criterion context used for benchmarking.
fn run_benchmark(c: &mut Criterion) {
    // Generate a data block for benchmarking
    let data_block = hash_benches::generate_data_block(BENCH_DATA_SIZE, Some(SEED));

    // Benchmark the throughput of the various hashing algorithms
    let mut group = c.benchmark_group("hashers-throughput");
    group.throughput(Throughput::Bytes(BENCH_DATA_SIZE as u64));
    group.measurement_time(Duration::from_secs(10));

    let algorithms = vec![
        Algorithm::BLAKE3,
        Algorithm::MD5,
        Algorithm::SHA256,
        Algorithm::SHA512,
        Algorithm::STREEBOG256,
        Algorithm::STREEBOG512,
    ];

    for algo in algorithms.iter() {
        // Measure the memory and CPU usage before running the benchmark
        let Ok(pre_stats) = block_on(hash_benches::system_info()) else {
            eprintln!("Failed to retrieve pre system information");
            return;
        };

        group.bench_with_input(
            BenchmarkId::new(algo.to_string(), BENCH_DATA_SIZE),
            &data_block,
            |b, data| {
                b.iter(|| run_hashing_algorithm(algo, data));
            },
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
            algo,
            hash_benches::bytes_to_human_readable(mem_usage),
            cpu_usage,
        );
    }
}

criterion_group!(benches, run_benchmark);
criterion_main!(benches);
