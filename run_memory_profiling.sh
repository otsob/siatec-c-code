#!/bin/sh

REPO_ROOT=$(pwd)

OUTPUT_PATH=$(pwd)/output/heaptrack_output
mkdir -p $OUTPUT_PATH

CLI_PATH=$(pwd)/posemir/target/release/posemir_cli
BENCHMARK_DATA_PATH=$(pwd)/posemir/benchmark/benches/data

SIZES=(1000 2000 3000 4000 5000 6000 7000 8000 9000 10000)

ALGORITHMS=(SIA SIAR SIATEC SIATEC-C)

measure_algorithms() {

  PATH_PREFIX=${BENCHMARK_DATA_PATH}/$1

  for algo in "${ALGORITHMS[@]}"
  do

    OUTPUT_DIR=${OUTPUT_PATH}/${algo}/$1

    for size in "${SIZES[@]}"
    do

      OUTPUT=${OUTPUT_DIR}/${size}
      mkdir -p ${OUTPUT}

      INPUT=${PATH_PREFIX}${size}.csv

      cd $OUTPUT

      heaptrack ${CLI_PATH} -a ${algo} -p ${INPUT} -i ${INPUT} -o /dev/null --batch-size=1 --sub-diag=1 --max-ioi=50.0
    done
  done
}

RAND_PATH=random/random_points_
MAX_PATH=max_pattern_count/max_pattern_count_
MIN_PATH=min_pattern_count/min_pattern_count_

measure_algorithms $RAND_PATH
measure_algorithms $MAX_PATH
measure_algorithms $MIN_PATH

cd $REPO_ROOT
