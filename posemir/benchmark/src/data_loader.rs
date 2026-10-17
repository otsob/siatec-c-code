/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use std::env;
use std::path::Path;

use posemir_discovery::io::csv::csv_to_2d_point_f64;
use posemir_discovery::point_set::point::Point2Df64;
use posemir_discovery::point_set::point_set::PointSet;

/// Configuration for running a benchmark
pub struct Config {
    /// min data set size
    pub min: usize,
    /// max data set size
    pub max: usize,
    /// step for incrementing data set size
    pub step: usize,
    /// the part of the path that defines which types of datasets are used
    pub path_str: String,
}

impl Config {
    /// Returns the default data loading config.
    /// The values for defaults can be set with environment variables:
    /// BENCHMARK_DATASET_MIN_SIZE
    /// BENCHMARK_DATASET_MAX_SIZE
    /// BENCHMARK_DATASET_STEP_SIZE
    ///
    /// # Arguments
    /// * `path_str` - the paths to the data set types
    pub fn default_counts(path_str: String) -> Config {
        let min = env::var("BENCHMARK_DATASET_MIN_SIZE")
            .unwrap_or_else(|_| String::from("100"))
            .parse()
            .unwrap();
        let max = env::var("BENCHMARK_DATASET_MAX_SIZE")
            .unwrap_or_else(|_| String::from("100"))
            .parse()
            .unwrap();
        let step = env::var("BENCHMARK_DATASET_STEP_SIZE")
            .unwrap_or_else(|_| String::from("100"))
            .parse()
            .unwrap();

        Config {
            min,
            max,
            step,
            path_str,
        }
    }
}

/// Returns the datasets of sizes defined by min, max, and step from the given data directory.
///
/// # Arguments
/// * `data_path` - Absolute path to the benches/data directory inside this repository
/// * `config` - The config that defines which types of data sets to load
///
pub fn load_datasets(data_path: &Path, config: &Config) -> Vec<PointSet<Point2Df64>> {
    let mut point_sets = Vec::new();

    let file_name_format = &config.path_str;

    for size in (config.min..config.max + 1).step_by(config.step) {
        let data_set_str_path = format!("{}{}.csv", &file_name_format, size);
        let data_set_path = Path::new(&data_set_str_path);
        let path = data_path.join(&data_set_path);
        let point_set = PointSet::new(csv_to_2d_point_f64(&path).unwrap());
        point_sets.push(point_set);
    }

    point_sets
}
