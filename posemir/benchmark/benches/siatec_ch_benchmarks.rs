/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use criterion::{criterion_group, criterion_main, Criterion};

use benchmark::data_loader;
use benchmark::tec_benchmark;
use posemir_discovery::siatec_ch::SiatecCH;

fn siatec_ch_benchmarks_with_random(c: &mut Criterion) {
    let config = data_loader::Config::default_counts(String::from("random/random_points_"));
    tec_benchmark::run_tec_benchmarks(&SiatecCH { max_ioi: 50.0 }, "SIATEC-CH(50)", &config, c);
}

fn siatec_ch_benchmarks_with_min_pattern_count(c: &mut Criterion) {
    let config =
        data_loader::Config::default_counts(String::from("min_pattern_count/min_pattern_count_"));
    tec_benchmark::run_tec_benchmarks(&SiatecCH { max_ioi: 50.0 }, "SIATEC-CH(50)", &config, c);
}

fn siatec_ch_benchmarks_with_max_pattern_count(c: &mut Criterion) {
    let config =
        data_loader::Config::default_counts(String::from("max_pattern_count/max_pattern_count_"));
    tec_benchmark::run_tec_benchmarks(&SiatecCH { max_ioi: 50.0 }, "SIATEC-CH(50)", &config, c);
}

criterion_group!(name = siatec_ch_benchmarks;
    config = Criterion::default().sample_size(10);
    targets = siatec_ch_benchmarks_with_random, siatec_ch_benchmarks_with_min_pattern_count, siatec_ch_benchmarks_with_max_pattern_count);
criterion_main!(siatec_ch_benchmarks);
