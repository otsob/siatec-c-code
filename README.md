# siatec-c-code
The code for the paper _SIATEC-C: Computationally efficient repeated pattern discovery in polyphonic music_.


### Requirements
- Rust development tools (compiler and cargo)
- Python 3.8 and Poetry
- Jupyter Notebook
- [Heaptrack](https://www.kdab.com/heaptrack-v1-3-0-release/) (works only on Linux)


### Installation
After cloning this repository with `git clone --recurse-submodules`, build [posemir](./posemir) with cargo
and install the `musii-kit` Jupyter kernel ([see README](./musii-kit/README.md)).

In order to build `posemir_cli`, run `cargo install --path posemir_cli` inside the `posemir` directory. 

### Running algorithms on JKU-PDD
You need to download JKU-PDD and extract it to this repository.
To run a single algorithm on JKU-PDD, run: `./run_on_jku_pdd.sh <ALGORITHM>`.
To run all algorithms included in the paper comparison on JKU-PDD, run `.run_all_on_jku_pdd.sh`.
The results are saved to the `output` directory by default.

### Running evaluation of the results on JKU-PDD
Navigate to `notebooks` and run `jupyter notebook`. Run all cells of the notebook to evaluate all
algorithms outputs on JKU-PDD.

### Running performance benchmarks
Run `./run_benchmarks.sh` to run performance benchmarks. The results can be found in `output/benchmark`
when execution is finished.

### Running memory profiling
TODO
