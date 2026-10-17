/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use std::cmp::Ordering;
use std::marker::PhantomData;

use crate::algorithm::TecAlgorithm;
use crate::heuristic::{stats_of, TecStats};
use crate::point_set::pattern::Pattern;
use crate::point_set::point::Point;
use crate::point_set::point_set::PointSet;
use crate::point_set::tec::Tec;

/// Implements the COSIATEC algorithm as described in [Meredith2013].
pub struct Cosiatec<T: Point, A: TecAlgorithm<T>> {
    tec_algorithm: A,
    _t: PhantomData<T>,
}

impl<T: Point, A: TecAlgorithm<T>> TecAlgorithm<T> for Cosiatec<T, A> {
    fn compute_tecs(&self, point_set: &PointSet<T>) -> Vec<Tec<T>> {
        let mut tecs = Vec::new();
        let on_output = |tec: Tec<T>| tecs.push(tec);
        self.compute_tecs_to_output(point_set, on_output);
        tecs
    }

    fn compute_tecs_to_output(&self, point_set: &PointSet<T>, mut on_output: impl FnMut(Tec<T>)) {
        let mut point_set_clone = point_set.clone();
        let mut iterations = 0;
        while point_set_clone.len() > 0 && iterations < point_set.len() {
            let best = self.get_best_tec(&point_set_clone);
            point_set_clone = point_set_clone.difference(&best.covered_set);
            on_output(best.tec);
            iterations += 1;
        }
    }
}

impl<T: Point, A: TecAlgorithm<T>> Cosiatec<T, A> {
    /// Creates a new instance of COSIATEC that uses the given TEC-algorithm
    /// for computing the TEC candidates.
    pub fn with(tec_algorithm: A) -> Cosiatec<T, A> {
        Cosiatec {
            tec_algorithm,
            _t: Default::default(),
        }
    }

    fn get_best_tec(&self, point_set: &PointSet<T>) -> TecStats<T> {
        let mut best: TecStats<T> = TecStats {
            tec: Tec {
                pattern: Pattern::new(&Vec::new()),
                translators: Vec::new(),
            },
            comp_ratio: -1.0,
            compactness: 0.0,
            covered_set: PointSet::new(Vec::new()),
            pattern_width: 0.0,
            pattern_area: 0.0,
        };

        let replace_best = |tec: Tec<T>| {
            let candidate = stats_of(tec.remove_redundant_translators(), point_set);
            if candidate.compare(&best) == Ordering::Less {
                best = candidate;
            }

            let conjugate = stats_of(tec.conjugate().remove_redundant_translators(), point_set);
            if conjugate.compare(&best) == Ordering::Less {
                best = conjugate;
            }
        };

        self.tec_algorithm
            .compute_tecs_to_output(point_set, replace_best);

        best
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::TecAlgorithm;
    use crate::cosiatec::Cosiatec;
    use crate::point_set::pattern::Pattern;
    use crate::point_set::point::Point2Df64;
    use crate::point_set::point_set::PointSet;
    use crate::siatec::Siatec;

    #[test]
    fn test_simple_point_set() {
        let point_set = PointSet::new(vec![
            Point2Df64 { x: 0.0, y: 0.0 },
            Point2Df64 { x: 1.0, y: 0.0 },
            Point2Df64 { x: 2.0, y: 0.0 },
            Point2Df64 { x: 3.0, y: 0.0 },
        ]);

        let siatec = Siatec {};
        let cosiatec = Cosiatec::with(siatec);

        let tecs = cosiatec.compute_tecs(&point_set);

        assert_eq!(1, tecs.len());
        let best_tec = &tecs[0];
        assert_eq!(
            Pattern::new(&vec![
                &Point2Df64 { x: 0.0, y: 0.0 },
                &Point2Df64 { x: 1.0, y: 0.0 },
            ]),
            best_tec.pattern
        );
        assert_eq!(vec![Point2Df64 { x: 2.0, y: 0.0 }], best_tec.translators);
    }
}
