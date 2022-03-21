#!/bin/sh

mkdir -p output/benchmark

export BENCHMARK_DATA_PATH=$(pwd)/posemir/benchmark/benches/data
export BENCHMARK_DATASET_MIN_SIZE=1000
export BENCHMARK_DATASET_MAX_SIZE=10000
export BENCHMARK_DATASET_STEP_SIZE=1000

cd posemir/benchmark
cargo criterion 2>&1 | tee ../../output/benchmark/criterion_output.txt
cp -r target/criterion ../../output/benchmark/
cd ../..
