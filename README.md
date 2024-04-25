# Overview

This project aims to provide benchmarking tools for evaluating the performance of various hash functions implemented in
Rust. It includes two benchmarks: one for single-threaded testing and another for asynchronous testing using
Tokio's tasks.

# Usage

## Single-Threaded Benchmark

To run the single-threaded benchmark, execute the following command:

```shell
cargo bench --bench hash_benches
```

This will run the benchmark using a single thread and output the performance metrics for each tested hash function.

## Asynchronous Benchmark

For the asynchronous benchmark using Tokio's green threads, execute:

```shell
cargo bench --bench async_hash_benches
```

This will run the benchmark in an asynchronous manner, leveraging Tokio's concurrency features, and display the
performance results.

# Supported Algorithms

The following hash algorithms are currently supported in this benchmarking tool:

- BLAKE3
- MD5
- SHA3-256
- SHA3-512
- STREEBOG-256
- STREEBOG-512