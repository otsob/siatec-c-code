/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use std::marker::PhantomData;

use crate::algorithm::TecAlgorithm;
use crate::heuristic::{stats_of, TecStats};
use crate::point_set::pattern::Pattern;
use crate::point_set::point::Point;
use crate::point_set::point_set::PointSet;
use crate::point_set::tec::Tec;

/// Implements the SIATECCompress algorithm as described in [Meredith2013].
pub struct SiatecCompress<T: Point, A: TecAlgorithm<T>> {
    tec_algorithm: A,
    _t: PhantomData<T>,
}

impl<T: Point, A: TecAlgorithm<T>> TecAlgorithm<T> for SiatecCompress<T, A> {
    fn compute_tecs(&self, point_set: &PointSet<T>) -> Vec<Tec<T>> {
        let mut tecs = self.tec_algorithm.compute_tecs(point_set);
        let mut conjugate_tecs: Vec<Tec<T>> = tecs.iter().map(|tec| tec.conjugate()).collect();
        tecs.append(&mut conjugate_tecs);
        let mut tec_stats: Vec<TecStats<T>> = tecs
            .iter()
            .map(|tec| stats_of(tec.remove_redundant_translators(), point_set))
            .collect();

        // Sort the tec stats so that best ones are first
        tec_stats.sort_by(TecStats::compare);

        self.compute_encoding(&tec_stats, point_set)
    }

    fn compute_tecs_to_output(&self, point_set: &PointSet<T>, mut on_output: impl FnMut(Tec<T>)) {
        let tecs = self.compute_tecs(point_set);
        for tec in tecs {
            on_output(tec);
        }
    }
}

impl<T: Point, A: TecAlgorithm<T>> SiatecCompress<T, A> {
    /// Creates a new instance of SIATECCompress that uses the given TEC-algorithm
    /// for computing the TEC candidates.
    pub fn with(tec_algorithm: A) -> SiatecCompress<T, A> {
        SiatecCompress {
            tec_algorithm,
            _t: Default::default(),
        }
    }

    fn compute_encoding(
        &self,
        tec_stats: &Vec<TecStats<T>>,
        point_set: &PointSet<T>,
    ) -> Vec<Tec<T>> {
        let mut total_cover = PointSet::new(Vec::new());
        let mut tec_cover = Vec::new();

        for tec_stat in tec_stats.iter() {
            let cov = &tec_stat.covered_set;
            let new_points = cov.difference(&total_cover);

            // Omitting -1 from the representation size as TECs do not have zero translator.
            let tec_repr_size = tec_stat.tec.pattern.len() + tec_stat.tec.translators.len();

            if new_points.len() > tec_repr_size {
                tec_cover.push(tec_stat.tec.clone());
                total_cover = total_cover.union(&cov);
                if total_cover.len() == point_set.len() {
                    break;
                }
            }
        }

        // Add any remaining residual points as a TEC
        let residual_points = point_set.difference(&total_cover);
        if residual_points.len() > 0 {
            let first = &residual_points[0];
            let pattern = Pattern::new(&vec![first]);
            let mut translators = Vec::new();

            for i in 1..residual_points.len() {
                translators.push(residual_points[i] - *first);
            }

            tec_cover.push(Tec {
                pattern,
                translators,
            });
        }

        tec_cover
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::TecAlgorithm;
    use crate::point_set::pattern::Pattern;
    use crate::point_set::point::Point2Df64;
    use crate::point_set::point_set::PointSet;
    use crate::siatec::Siatec;
    use crate::siatec_compress::SiatecCompress;

    #[test]
    fn test_simple_point_set() {
        let point_set = PointSet::new(vec![
            Point2Df64 { x: 0.0, y: 0.0 },
            Point2Df64 { x: 1.0, y: 0.0 },
            Point2Df64 { x: 2.0, y: 0.0 },
            Point2Df64 { x: 3.0, y: 0.0 },
        ]);

        let siatec = Siatec {};
        let siatec_compress = SiatecCompress::with(siatec);

        let tecs = siatec_compress.compute_tecs(&point_set);
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
