/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use std::cmp::Ordering;

use crate::point_set::pattern::Pattern;
use crate::point_set::point::Point;
use crate::point_set::point_set::PointSet;
use crate::point_set::tec::Tec;

#[derive(Debug)]
pub struct TecStats<T: Point> {
    pub tec: Tec<T>,
    pub comp_ratio: f64,
    pub compactness: f64,
    pub covered_set: PointSet<T>,
    pub pattern_width: f64,
    pub pattern_area: f64,
}

pub fn stats_of<T: Point>(tec: Tec<T>, point_set: &PointSet<T>) -> TecStats<T> {
    let covered_set = tec.covered_set();
    let comp_ratio = compr_ratio_with_cov(&tec, &covered_set);
    let bb = bounding_box(&tec.pattern);
    let compactness = bb_compactness(&tec, point_set);

    let pattern_width = bb.upper_x - bb.lower_x;
    let pattern_area = (bb.upper_x - bb.lower_x) * (bb.upper_y - bb.lower_y);

    TecStats {
        tec,
        comp_ratio,
        compactness,
        covered_set,
        pattern_width,
        pattern_area,
    }
}

impl<T: Point> TecStats<T> {
    /// Compares two TEC statistics so that better TECs are ordered first (`Less`).
    ///
    /// Pre-condition: none. NaN values are allowed.
    ///
    /// Post-condition: the result is a total order. It is antisymmetric and transitive.
    /// The criteria are compared lexicographically in this order: compression ratio (higher is better),
    /// compactness (higher is better), covered set size (higher is better), pattern size (higher is better),
    /// pattern width (lower is better) and pattern area (lower is better).
    pub fn compare(&self, other: &TecStats<T>) -> Ordering {
        other
            .comp_ratio
            .total_cmp(&self.comp_ratio)
            .then_with(|| other.compactness.total_cmp(&self.compactness))
            .then_with(|| other.covered_set.len().cmp(&self.covered_set.len()))
            .then_with(|| other.tec.pattern.len().cmp(&self.tec.pattern.len()))
            .then_with(|| self.pattern_width.total_cmp(&other.pattern_width))
            .then_with(|| self.pattern_area.total_cmp(&other.pattern_area))
    }
}

struct BoundingBox {
    lower_x: f64,
    lower_y: f64,
    upper_x: f64,
    upper_y: f64,
}

impl BoundingBox {
    fn contains<T: Point>(&self, point: &T) -> bool {
        let x = point.component_f64(0).unwrap();
        let y = point.component_f64(1).unwrap();

        if x < self.lower_x {
            return false;
        }

        if x > self.upper_x {
            return false;
        }

        if y < self.lower_y {
            return false;
        }

        if y > self.upper_y {
            return false;
        }

        true
    }
}

fn bounding_box<T: Point>(pattern: &Pattern<T>) -> BoundingBox {
    let mut bb = BoundingBox {
        lower_x: f64::MAX,
        lower_y: f64::MAX,
        upper_x: f64::MIN,
        upper_y: f64::MIN,
    };

    for point in pattern {
        let point_x = point.component_f64(0).unwrap();
        let point_y = point.component_f64(1).unwrap();

        if point_x < bb.lower_x {
            bb.lower_x = point_x;
        }
        if point_x > bb.upper_x {
            bb.upper_x = point_x;
        }
        if point_y < bb.lower_y {
            bb.lower_y = point_y;
        }
        if point_y > bb.upper_y {
            bb.upper_y = point_y;
        }
    }

    bb
}

fn compr_ratio_with_cov<T: Point>(tec: &Tec<T>, cov: &PointSet<T>) -> f64 {
    let cov_size = cov.len() as f64;
    let pat_size = tec.pattern.len() as f64;
    let transl_size = tec.translators.len() as f64;

    // The TEC type is expected to not contain a zero-translator,
    // therefore the denominator does not include the -1 as in [Meredith2013].
    cov_size / (pat_size + transl_size)
}

fn bb_compactness<T: Point>(tec: &Tec<T>, point_set: &PointSet<T>) -> f64 {
    let mut best_compactness = 0.0;
    let expanded = tec.expand();

    for pattern in &expanded {
        let bb = bounding_box(pattern);
        let mut contained: f64 = 0.0;

        for point in point_set {
            if bb.contains(point) {
                contained += 1.0;
            }
        }

        let pat_size = tec.pattern.len() as f64;

        let compactness = pat_size / contained;
        if compactness > best_compactness {
            best_compactness = compactness;
        }
    }

    best_compactness
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::heuristic::{bounding_box, TecStats};
    use crate::point_set::pattern::Pattern;
    use crate::point_set::point::Point2Df64;
    use crate::point_set::point_set::PointSet;
    use crate::point_set::tec::Tec;

    fn stats(
        comp_ratio: f64,
        compactness: f64,
        covered: usize,
        pattern: usize,
        width: f64,
        area: f64,
    ) -> TecStats<Point2Df64> {
        let points: Vec<Point2Df64> = (0..pattern)
            .map(|i| Point2Df64 { x: i as f64, y: 0.0 })
            .collect();
        let covered_points: Vec<Point2Df64> = (0..covered)
            .map(|i| Point2Df64 { x: i as f64, y: 0.0 })
            .collect();
        TecStats {
            tec: Tec {
                pattern: Pattern::new(&points.iter().collect()),
                translators: Vec::new(),
            },
            comp_ratio,
            compactness,
            covered_set: PointSet::new(covered_points),
            pattern_width: width,
            pattern_area: area,
        }
    }

    fn sample() -> Vec<TecStats<Point2Df64>> {
        let mut v = Vec::new();
        for cr in [1.0, 2.0, f64::NAN] {
            for comp in [0.5, f64::NAN] {
                for cov in [3, 4] {
                    for pat in [1, 2] {
                        for w in [1.0, 2.0] {
                            v.push(stats(cr, comp, cov, pat, w, 1.0));
                        }
                    }
                }
            }
        }
        v
    }

    #[test]
    fn test_compare_priority_order() {
        let better = stats(2.0, 0.1, 3, 1, 5.0, 5.0);
        let worse = stats(1.0, 0.9, 9, 9, 1.0, 1.0);
        assert_eq!(Ordering::Less, better.compare(&worse));
        assert_eq!(Ordering::Greater, worse.compare(&better));

        let narrow = stats(1.0, 0.5, 3, 2, 1.0, 1.0);
        let wide = stats(1.0, 0.5, 3, 2, 2.0, 1.0);
        assert_eq!(Ordering::Less, narrow.compare(&wide));
        assert_eq!(Ordering::Equal, narrow.compare(&narrow));
    }

    #[test]
    fn test_compare_antisymmetric_and_transitive_with_nan() {
        let v = sample();
        for a in &v {
            for b in &v {
                assert_eq!(a.compare(b), b.compare(a).reverse());
                for c in &v {
                    if a.compare(b) != Ordering::Greater && b.compare(c) != Ordering::Greater {
                        assert_ne!(Ordering::Greater, a.compare(c));
                    }
                }
            }
        }
    }

    #[test]
    fn test_compare_allows_sorting_with_nan() {
        let mut v = sample();
        v.sort_by(TecStats::compare);
        for pair in v.windows(2) {
            assert_ne!(Ordering::Greater, pair[0].compare(&pair[1]));
        }
    }

    #[test]
    fn test_bounding_box_upper_y() {
        let a = Point2Df64 { x: 1.0, y: 2.0 };
        let b = Point2Df64 { x: 3.0, y: 5.0 };
        let c = Point2Df64 { x: 2.0, y: 4.0 };
        let bb = bounding_box(&Pattern::new(&vec![&a, &b, &c]));
        assert_eq!(1.0, bb.lower_x);
        assert_eq!(3.0, bb.upper_x);
        assert_eq!(2.0, bb.lower_y);
        assert_eq!(5.0, bb.upper_y);
    }
}
