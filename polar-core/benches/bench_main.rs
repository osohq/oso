//! Polar benchmarking suite
//!
mod benchmarks;

use criterion::criterion_main;

criterion_main!(benchmarks::queries::benches, benchmarks::partial::benches);
