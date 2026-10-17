use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use serde_json::{json, Value};

use crate::point_set::pattern::Pattern;
use crate::point_set::point::Point;
use crate::point_set::point::Point2Df64;
use crate::point_set::tec::Tec;

/// Write a set of TECs into separate JSON files, following the following format for each TEC:
/// ```json
/// {
///    "piece": "Beethoven op.1",
///    "pattern": {
///     "label": "P3",
///     "source": "siatec",
///     "data_type": "point_set",
///     "data": [
///       [
///         1.0,
///         64.0
///       ],
///       [
///         2.0,
///         60.0
///       ]
///            ...
///     ]
///   },
///   "occurrences": [ list of pattern objects ]
/// }
/// ```
/// The files are written into the directory at the given path.
///
/// # Arguments:
/// * `piece` - Name of the piece
/// * `source` - The source of the TECs, e.g, algorithm or analysts name.
/// * `tecs` - The TECs that are written to JSON
/// * `path` - Output directory path
pub fn write_tecs_to_json_files(piece: &str, source: &str, tecs: &[Tec<Point2Df64>], path: &Path) {
    for (i, tec) in tecs.iter().enumerate() {
        let label = &format!("P{}", i);
        let expanded = tec.expand();
        let pattern = pattern_to_json(label, source, &expanded[0]);
        let occurrences: Vec<Value> = expanded[1..]
            .iter()
            .map(|p| pattern_to_json(label, source, p))
            .collect();

        let json_value = json!({
            "piece": piece,
            "pattern": pattern,
            "occurrences": occurrences
        });

        let file_name = format!("{}{}", label, ".json");
        let pattern_path = path.join(Path::new(&file_name));

        let mut buffered_writer = BufWriter::new(File::create(pattern_path).unwrap());
        serde_json::to_writer_pretty(&mut buffered_writer, &json_value).unwrap()
    }
}

/// Write a set of TECs into a single JSON file. The TECs are written into a JSON list
/// written using the format in `write_tecs_to_json_files`
///
/// # Arguments:
/// * `piece` - Name of the piece
/// * `source` - The source of the TECs, e.g, algorithm or analysts name.
/// * `tecs` - The TECs that are written to JSON
/// * `path` - Output path
pub fn write_tecs_to_json(piece: &str, source: &str, tecs: &[Tec<Point2Df64>], path: &Path) {
    let mut json_values = Vec::new();
    for (i, tec) in tecs.iter().enumerate() {
        let label = &format!("P{}", i);
        let expanded = tec.expand();
        let pattern = pattern_to_json(label, source, &expanded[0]);
        let occurrences: Vec<Value> = expanded[1..]
            .iter()
            .map(|p| pattern_to_json(label, source, p))
            .collect();

        json_values.push(json!({
            "piece": piece,
            "pattern": pattern,
            "occurrences": occurrences
        }));
    }

    let mut buffered_writer = BufWriter::new(File::create(path).unwrap());
    serde_json::to_writer_pretty(&mut buffered_writer, &json_values).unwrap()
}

fn pattern_to_json(label: &str, source: &str, pattern: &Pattern<Point2Df64>) -> Value {
    let data: Vec<Value> = pattern
        .into_iter()
        .map(|p| {
            Value::Array(vec![
                json!(p.component_f64(0).unwrap()),
                json!(p.component_f64(1).unwrap()),
            ])
        })
        .collect();

    json!({
        "label": label,
        "source": source,
        "data_type": "point_set",
        "data": data
    })
}
