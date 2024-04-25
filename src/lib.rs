use rand::{random, RngCore, rngs::SmallRng, SeedableRng};
use zerocopy::AsBytes;

/// Generates a data block of the specified size using the given seed value.
///
/// # Arguments
///
/// * `size` - The size of the data block to generate.
/// * `seed_value` - The seed value to use for generating the data block.
///
/// # Returns
///
/// A vector of bytes representing the generated data block.
pub fn generate_data_block(size: usize, seed_rnd: Option<u128>) -> Vec<u8> {
    let seed_value = seed_rnd.unwrap_or_else(random);

    let mut seed = [0u8; 32];
    seed.copy_from_slice(seed_value.as_bytes().repeat(2).as_ref());

    let mut rng = SmallRng::from_seed(seed);
    let mut block = vec![0u8; size];
    rng.fill_bytes(&mut block);

    block
}

/// Generates a nested vector representing a set of data blocks for hashing tasks.
///
/// This function constructs a three-dimensional vector where each top-level element
/// represents a task, each second-level element represents a chunk within that task,
/// and each third-level element is a vector of bytes (the data chunk itself).
///
/// # Parameters
/// - `num_tasks`: The number of parallel tasks, each containing a set of data chunks.
/// - `num_chunks_per_task`: The number of data chunks per task.
/// - `chunk_size`: The size of each data chunk in bytes.
/// - `seed_rnd`: Optional seed for reproducibility. If `None`, a random seed is used.
///
/// # Returns
///
/// Returns a three-dimensional vector containing the generated data blocks.
pub fn generate_data_blocks_for_async_benchmarking(
    num_tasks: usize,
    num_chunks_per_task: usize,
    chunk_size: usize,
    seed_rnd: Option<u128>,
) -> Vec<Vec<Vec<u8>>> {
    let seed_value = seed_rnd.unwrap_or_else(random);

    (0..num_tasks)
        .map(|_| {
            (0..num_chunks_per_task)
                .map(|_| {
                    let mut seed = [0u8; 32];
                    seed.copy_from_slice(seed_value.as_bytes().repeat(2).as_ref());
                    let mut rng = SmallRng::from_seed(seed);
                    let mut block = vec![0u8; chunk_size];
                    rng.fill_bytes(&mut block);
                    block
                })
                .collect()
        })
        .collect()
}

/// Converts a byte count to a human-readable format.
///
/// # Arguments
///
/// * `bytes` - The number of bytes to convert.
///
/// # Returns
///
/// A string representing the human-readable byte count.
pub fn bytes_to_human_readable(bytes: u64) -> String {
    let sizes = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
    let factor = 1024u64;

    if bytes < factor {
        return format!("{} B", bytes);
    }

    let mut count = bytes as f64;
    let mut i = 0;
    while count >= factor as f64 && i < sizes.len() - 1 {
        count /= factor as f64;
        i += 1;
    }

    format!("{:.2} {}", count, sizes[i])
}

/// Function returning the system memory and CPU usage information.
///
/// # Returns
///
/// Result containing the system memory and CPU usage information on success.
pub async fn system_info() -> Result<(u64, heim::cpu::CpuUsage), heim::Error> {
    let memory = heim::memory::memory().await?;
    let cpu_usage = heim::cpu::usage().await?;

    Ok((memory.available().value, cpu_usage))
}
