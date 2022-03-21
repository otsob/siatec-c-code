#!/bin/sh

OUTPUT_PATH=output/heaptrack_output
mkdir -p $OUTPUT_PATH

CLI_PATH=./posemir/target/release/posemir_cli
BENCHMARK_DATA_PATH=$(pwd)/posemir/benchmark/benches/data

# TODO: Set this this to non-debug array
#SIZES=(1000 2000 3000 4000 5000 6000 7000 8000 9000 10000)
SIZES=(100 200 300)

ALGORITHMS=(SIA SIAR SIATEC SIATEC-C)

measure_algorithms() {

  PATH_PREFIX=${BENCHMARK_DATA_PATH}/$1

  for algo in "${ALGORITHMS[@]}"
  do

    OUTPUT_DIR=${OUTPUT_PATH}/${algo}/$1
    mkdir -p ${OUTPUT_DIR}

    for size in "${SIZES[@]}"
    do

      OUTPUT=${OUTPUT_DIR}/${size}.txt
      INPUT=${PATH_PREFIX}${size}.csv

      # TODO: Get heaptrack to print and save the peak heap usage
      heaptrack ${CLI_PATH} -a ${algo} -p ${INPUT} -i ${INPUT} -o /dev/null --batch-size=1 --sub-diag=3 --max-ioi=50.0 > ${OUTPUT}
    done
  done
}

RAND_PATH=random/random_points_
MAX_PATH=max_pattern_count/max_pattern_count_
MIN_PATH=min_pattern_count/min_pattern_count_

measure_algorithms $RAND_PATH
measure_algorithms $MAX_PATH
measure_algorithms $MIN_PATH
