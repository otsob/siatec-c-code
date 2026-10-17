/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use criterion::{criterion_group, criterion_main, Criterion};

use benchmark::data_loader;
use benchmark::mtp_benchmark;
use posemir_discovery::siar::SiaR;

fn siar_benchmarks_with_random(c: &mut Criterion) {
    let config = data_loader::Config::default_counts(String::from("random/random_points_"));
    let algorithm = SiaR { r: 1 };
    mtp_benchmark::run_mtp_benchmarks(&algorithm, "SIAR(1)", &config, c);
}

fn siar_benchmarks_with_min_pattern_count(c: &mut Criterion) {
    let config =
        data_loader::Config::default_counts(String::from("min_pattern_count/min_pattern_count_"));
    let algorithm = SiaR { r: 1 };
    mtp_benchmark::run_mtp_benchmarks(&algorithm, "SIAR(1)", &config, c);
}

fn siar_benchmarks_with_max_pattern_count(c: &mut Criterion) {
    let config =
        data_loader::Config::default_counts(String::from("max_pattern_count/max_pattern_count_"));
    let algorithm = SiaR { r: 1 };
    mtp_benchmark::run_mtp_benchmarks(&algorithm, "SIAR(1)", &config, c);
}

criterion_group!(name = siar_benchmarks;
    config = Criterion::default().sample_size(10);
    targets = siar_benchmarks_with_random, siar_benchmarks_with_min_pattern_count, siar_benchmarks_with_max_pattern_count);
criterion_main!(siar_benchmarks);
