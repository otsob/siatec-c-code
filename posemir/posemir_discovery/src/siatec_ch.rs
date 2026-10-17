/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */

use std::cmp::max;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use hashers::fx_hash::FxHasher64;

use crate::algorithm::TecAlgorithm;
use crate::point_set::mtp::Mtp;
use crate::point_set::pattern::Pattern;
use crate::point_set::point::Point;
use crate::point_set::point_set::PointSet;
use crate::point_set::tec::Tec;
use crate::siatec_c::SiatecC;

type IndPair = [usize; 2];
type HMap<T> = HashMap<T, Vec<IndPair>, BuildHasherDefault<FxHasher64>>;

/// Implements the SIATEC-CH algorithm (prototype).
pub struct SiatecCH {
    /// Maximum allowed inter-onset-interval (IOI) between successive points in a pattern.
    pub max_ioi: f64,
}

impl<T: Point> TecAlgorithm<T> for SiatecCH {
    fn compute_tecs(&self, point_set: &PointSet<T>) -> Vec<Tec<T>> {
        let diff_index = self.compute_diff_index(point_set);
        let mut tecs = Vec::new();
        let on_output = |mtp: Tec<T>| tecs.push(mtp);
        self.compute_split_mtp_tecs(point_set, &diff_index, on_output);
        tecs
    }

    fn compute_tecs_to_output(&self, point_set: &PointSet<T>, on_output: impl FnMut(Tec<T>)) {
        let diff_index = self.compute_diff_index(point_set);
        self.compute_split_mtp_tecs(point_set, &diff_index, on_output)
    }
}

impl SiatecCH {
    fn new_hmap<T: Point>() -> HMap<T> {
        HashMap::with_hasher(BuildHasherDefault::<FxHasher64>::default())
    }

    /// Returns a hashmap of difference - index-pair-vector pairs
    fn compute_diff_index<T: Point>(&self, point_set: &PointSet<T>) -> HMap<T> {
        let n = point_set.len();
        let mut forward_diffs = SiatecCH::new_hmap();

        for i in 0..(n - 1) {
            let from = &point_set[i];

            for j in (i + 1)..n {
                let to = &point_set[j];
                let diff = *to - *from;
                let ioi_opt = diff.component_f64(0);
                match ioi_opt {
                    Some(ioi) => {
                        if ioi > self.max_ioi {
                            break;
                        }
                    }
                    None => panic!("Cannot compute with points with no onset component 0"),
                }

                match forward_diffs.get_mut(&diff) {
                    Some(indices) => {
                        indices.push([i, j]);
                    }
                    None => {
                        forward_diffs.insert(diff, vec![[i, j]]);
                    }
                }
            }
        }

        forward_diffs
    }

    fn compute_split_mtp_tecs<T: Point>(
        &self,
        point_set: &PointSet<T>,
        diff_index: &HMap<T>,
        mut on_output: impl FnMut(Tec<T>),
    ) {
        let n = point_set.len();
        // Initialize the window beginnings to start from the points:
        // target_indices keeps track of the target indices for the translators
        // window_bounds keeps track of the upper bounds of the windows within which
        // the target points of the translators must be.
        let mut target_indices: Vec<usize> = (0..n).collect();
        let mut window_bounds = SiatecC::init_window_upper_bounds(self.max_ioi, point_set);

        let mut cover: Vec<usize> = vec![0; n];

        while target_indices[0] < n {
            // Compute forward diffs in restricted size window
            let forward_diffs = self.compute_forward_diffs_within_window(
                point_set,
                n,
                &mut target_indices,
                &mut window_bounds,
            );
            let mtps = SiatecCH::partition_to_mtps(point_set, &forward_diffs);
            let split_triples = SiatecC::split_mtps_on_ioi(&mtps, self.max_ioi);

            for split_triple in &split_triples {
                let pattern = &split_triple.0;
                let source_ind = &split_triple.1;
                let target_ind = &split_triple.2;

                if SiatecC::improves_cover(&cover, source_ind, target_ind, pattern.len()) {
                    let translators = SiatecCH::find_translators_update_cover(
                        pattern, diff_index, point_set, &mut cover,
                    );
                    on_output(Tec {
                        pattern: pattern.clone(),
                        translators,
                    });
                }
            }
        }
    }

    /// Computes the forward difference vectors for all points, such that, the target points are all within
    /// a restricted size window. Each source point has its own window position, so that difference
    /// vectors of the same size are always computed during the same iteration.
    fn compute_forward_diffs_within_window<T: Point>(
        &self,
        point_set: &PointSet<T>,
        n: usize,
        target_indices: &mut Vec<usize>,
        window_bounds: &mut Vec<f64>,
    ) -> HMap<T> {
        let mut forward_diffs = SiatecCH::new_hmap();
        for i in 0..(n - 1) {
            let from = &point_set[i];
            let target_index = target_indices[i];
            if target_index >= n {
                continue;
            }

            let mut window_exceeds_data = true;

            for j in target_index..n {
                if i == j {
                    continue;
                }

                let to = &point_set[j];
                let onset = to.component_f64(0).unwrap();
                let diff: T = *to - *from;

                if onset > window_bounds[i] {
                    target_indices[i] = j;
                    window_exceeds_data = false;
                    window_bounds[i] += self.max_ioi;
                    break;
                }

                match forward_diffs.get_mut(&diff) {
                    Some(indices) => {
                        indices.push([i, j]);
                    }
                    None => {
                        forward_diffs.insert(diff, vec![[i, j]]);
                    }
                }
            }

            // If the window has not reached the IOI limit, then the end of the window
            // extends beyond the points in the data set, so there are no mode windows
            // to handle from the starting index.
            if window_exceeds_data {
                target_indices[i] = n;
            }
        }
        forward_diffs
    }

    /// Partitions the forward diffs to MTPs and returns a vector of triples, where:
    /// 0. MTP
    /// 1. source indices: the indices that form the MTP
    /// 2. target indices: the indices of the points that form the translated MTP
    fn partition_to_mtps<T: Point>(
        point_set: &PointSet<T>,
        forward_diffs: &HMap<T>,
    ) -> Vec<(Mtp<T>, Vec<usize>, Vec<usize>)> {
        let mut mtps: Vec<(Mtp<T>, Vec<usize>, Vec<usize>)> = Vec::new();

        for (translator, ind_pairs) in forward_diffs {
            let m = ind_pairs.len();
            let mut source_indices = Vec::with_capacity(m);
            let mut target_indices = Vec::with_capacity(m);

            for ind_pair in ind_pairs.iter() {
                source_indices.push(ind_pair[0]);
                target_indices.push(ind_pair[1]);
            }

            mtps.push((
                Mtp {
                    translator: *translator,
                    pattern: point_set.get_pattern(&source_indices),
                },
                source_indices,
                target_indices,
            ));
        }
        mtps
    }

    fn find_indices<'a, T: Point>(diff_index: &'a HMap<T>, translation: &T) -> &'a Vec<IndPair> {
        match diff_index.get(translation) {
            Some(indices) => indices,
            None => {
                println!("Could not find exact match for {:?}", translation);
                panic!("Cannot default to any value");
            }
        }
    }

    fn find_translators_update_cover<T: Point>(
        pattern: &Pattern<T>,
        diff_index: &HMap<T>,
        point_set: &PointSet<T>,
        cover: &mut Vec<usize>,
    ) -> Vec<T> {
        if pattern.len() == 1 {
            return SiatecC::find_single_point_translators_update_cover(pattern, point_set, cover);
        }

        let vectorized = pattern.vectorize();
        let v = &vectorized[0];

        let indices = SiatecCH::find_indices(diff_index, v);
        let mut target_indices = Vec::with_capacity(indices.len());
        for ind_pair in indices.iter() {
            target_indices.push(ind_pair[1]);
        }

        for i in 1..vectorized.len() {
            let diff = &vectorized[i];
            let translatable_indices = SiatecCH::find_indices(diff_index, diff);
            target_indices =
                SiatecC::match_index_pairs_forward(&target_indices, translatable_indices);
        }

        let mut translators = Vec::with_capacity(target_indices.len());
        let last_point = pattern[pattern.len() - 1];
        for i in 0..target_indices.len() {
            let translator = point_set[target_indices[i]] - last_point;
            if !translator.is_zero() {
                translators.push(translator);
            }
        }

        // Update cover
        SiatecCH::update_cover(pattern, diff_index, cover, &vectorized, target_indices);

        translators
    }

    fn update_cover<T: Point>(
        pattern: &Pattern<T>,
        diff_index: &HMap<T>,
        cover: &mut Vec<usize>,
        vectorized: &Pattern<T>,
        init_cover_ind: Vec<usize>,
    ) {
        let mut cover_indices = init_cover_ind;

        for i in (0..vectorized.len()).rev() {
            let diff = &vectorized[i];
            let translatable_indices = SiatecCH::find_indices(diff_index, diff);
            cover_indices =
                SiatecC::match_index_pairs_backward(&cover_indices, translatable_indices);

            for c in &cover_indices {
                cover[*c] = max(cover[*c], pattern.len());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::TecAlgorithm;
    use crate::point_set::pattern::Pattern;
    use crate::point_set::point::Point2Df64;
    use crate::point_set::point_set::PointSet;
    use crate::point_set::tec::Tec;
    use crate::siatec_c::SiatecC;
    use crate::siatec_ch::SiatecCH;

    #[test]
    fn test_with_minimal_number_of_mtps() {
        // Create a point set where the number of MTPs is minimal.
        let mut points = Vec::new();
        let a = Point2Df64 { x: 1.0, y: 1.0 };
        points.push(a);
        let b = Point2Df64 { x: 2.0, y: 1.0 };
        points.push(b);
        let c = Point2Df64 { x: 3.0, y: 1.0 };
        points.push(c);
        let d = Point2Df64 { x: 4.0, y: 1.0 };
        points.push(d);

        let point_set = PointSet::new(points);
        let siatec_ch = SiatecCH { max_ioi: 2.0 };
        let mut tecs = siatec_ch.compute_tecs(&point_set);
        tecs.sort_by(|a, b| a.pattern.len().cmp(&b.pattern.len()));

        assert_eq!(3, tecs.len());
        assert_eq!(1, tecs[0].pattern.len());
        assert_eq!(point_set, tecs[0].covered_set());
        assert_eq!(
            Tec {
                pattern: Pattern::new(&vec![&a, &b]),
                translators: vec![Point2Df64 { x: 1.0, y: 0.0 }, Point2Df64 { x: 2.0, y: 0.0 }],
            },
            tecs[1]
        );
        assert_eq!(
            Tec {
                pattern: Pattern::new(&vec![&a, &b, &c]),
                translators: vec![Point2Df64 { x: 1.0, y: 0.0 }],
            },
            tecs[2]
        );
    }

    #[test]
    fn test_with_gap_and_minimal_number_of_mtps() {
        // Create a point set where the number of MTPs is minimal.
        let mut points = Vec::new();
        let a = Point2Df64 { x: 1.0, y: 1.0 };
        points.push(a);
        let b = Point2Df64 { x: 2.0, y: 1.0 };
        points.push(b);
        let c = Point2Df64 { x: 5.0, y: 1.0 };
        points.push(c);
        let d = Point2Df64 { x: 6.0, y: 1.0 };
        points.push(d);

        let point_set = PointSet::new(points);
        let siatec_ch = SiatecCH { max_ioi: 2.0 };
        let mut tecs = siatec_ch.compute_tecs(&point_set);

        SiatecC::remove_translational_duplicates(&mut tecs);

        assert_eq!(2, tecs.len());
        assert_eq!(1, tecs[0].pattern.len());
        assert_eq!(point_set, tecs[0].covered_set());
        assert_eq!(
            Tec {
                pattern: Pattern::new(&vec![&a, &b]),
                translators: vec![Point2Df64 { x: 4.0, y: 0.0 }],
            },
            tecs[1]
        );
    }

    #[test]
    fn test_with_gaps_and_minimal_number_of_mtps() {
        // Create a point set where the number of MTPs is minimal.
        let mut points = Vec::new();
        let a = Point2Df64 { x: 1.0, y: 1.0 };
        points.push(a);
        let b = Point2Df64 { x: 2.0, y: 1.0 };
        points.push(b);
        let c = Point2Df64 { x: 4.0, y: 1.0 };
        points.push(c);
        let d = Point2Df64 { x: 7.0, y: 1.0 };
        points.push(d);
        let e = Point2Df64 { x: 8.0, y: 1.0 };
        points.push(e);

        let point_set = PointSet::new(points);
        let siatec_ch = SiatecCH { max_ioi: 2.0 };
        let mut tecs = siatec_ch.compute_tecs(&point_set);

        SiatecC::remove_translational_duplicates(&mut tecs);

        assert_eq!(2, tecs.len());
        assert_eq!(1, tecs[0].pattern.len());
        assert_eq!(point_set, tecs[0].covered_set());
        assert_eq!(
            Tec {
                pattern: Pattern::new(&vec![&a, &b]),
                translators: vec![Point2Df64 { x: 6.0, y: 0.0 }],
            },
            tecs[1]
        );
    }
}
