/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */

use std::cmp::Ordering::Equal;
use std::cmp::{max, Ordering};

use crate::algorithm::TecAlgorithm;
use crate::point_set::mtp::Mtp;
use crate::point_set::pattern::Pattern;
use crate::point_set::point::Point;
use crate::point_set::point_set::PointSet;
use crate::point_set::tec::Tec;

type IndPair = [usize; 2];

/// Implements the SIATEC-C algorithm (prototype).
pub struct SiatecC {
    /// Maximum allowed inter-onset-interval (IOI) between successive points in a pattern.
    pub max_ioi: f64,
}

impl<T: Point> TecAlgorithm<T> for SiatecC {
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

impl SiatecC {
    /// Computes the IOI between to points. Onset time is
    /// assumed to be the first component of the points and all points
    /// are assumed to have dimensionality of at least one.
    pub(crate) fn ioi<T: Point>(a: &T, b: &T) -> f64 {
        let a_onset = a.component_f64(0);
        let b_onset = b.component_f64(0);
        b_onset.unwrap() - a_onset.unwrap()
    }

    /// Returns a vector of difference - index-pair-vector pairs, sorted in ascending lexicographical
    /// order of the difference vectors.
    fn compute_diff_index<T: Point>(&self, point_set: &PointSet<T>) -> Vec<(T, Vec<IndPair>)> {
        let n = point_set.len();

        let forward_diffs = self.compute_forward_diffs(point_set, n);

        SiatecC::partition_by_diff_vector(&forward_diffs)
    }

    fn partition_by_diff_vector<T: Point>(
        forward_diffs: &Vec<(T, [usize; 2])>,
    ) -> Vec<(T, Vec<IndPair>)> {
        let mut diff_index: Vec<(T, Vec<IndPair>)> = Vec::new();
        let m = forward_diffs.len();
        let mut i = 0;
        while i < m {
            let mut index_pairs: Vec<IndPair> = Vec::new();
            let translator = &forward_diffs[i].0;

            let mut j = i;
            while j < m && *translator == forward_diffs[j].0 {
                index_pairs.push(forward_diffs[j].1);
                j += 1;
            }

            diff_index.push((*translator, index_pairs));
            i = j;
        }

        diff_index
    }

    /// Computes forward differences that have inter-onset-interval of at most the limit set
    /// in this instance of SiatecC.
    fn compute_forward_diffs<T: Point>(
        &self,
        point_set: &PointSet<T>,
        n: usize,
    ) -> Vec<(T, IndPair)> {
        let mut forward_diffs: Vec<(T, IndPair)> = Vec::new();

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

                forward_diffs.push((diff, [i, j]));
            }
        }

        SiatecC::sort_with_ind_pairs(&mut forward_diffs);

        forward_diffs
    }

    pub(crate) fn init_window_upper_bounds<T: Point>(
        max_ioi: f64,
        point_set: &PointSet<T>,
    ) -> Vec<f64> {
        let mut window_bounds = Vec::with_capacity(point_set.len());

        for point in point_set {
            let end = point.component_f64(0).unwrap() + max_ioi;
            window_bounds.push(end);
        }

        window_bounds
    }

    fn compute_split_mtp_tecs<T: Point>(
        &self,
        point_set: &PointSet<T>,
        diff_index: &Vec<(T, Vec<IndPair>)>,
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
            let mut forward_diffs = self.compute_forward_diffs_within_window(
                point_set,
                n,
                &mut target_indices,
                &mut window_bounds,
            );
            let mtps = SiatecC::partition_to_mtps(point_set, &mut forward_diffs);
            let split_triples = SiatecC::split_mtps_on_ioi(&mtps, self.max_ioi);

            for split_triple in &split_triples {
                let pattern = &split_triple.0;
                let source_ind = &split_triple.1;
                let target_ind = &split_triple.2;

                if SiatecC::improves_cover(&cover, source_ind, target_ind, pattern.len()) {
                    let translators = SiatecC::find_translators_update_cover(
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

    pub(crate) fn improves_cover(
        cover: &Vec<usize>,
        source_ind: &Vec<usize>,
        target_ind: &Vec<usize>,
        pattern_len: usize,
    ) -> bool {
        for s_ind in source_ind {
            if cover[*s_ind] < pattern_len {
                return true;
            }
        }

        for t_ind in target_ind {
            if cover[*t_ind] < pattern_len {
                return true;
            }
        }

        false
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
    ) -> Vec<(T, IndPair)> {
        let mut forward_diffs = Vec::new();
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

                forward_diffs.push((diff, [i, j]))
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

    /// Split the MTPs and their associated source and target index vectors on gaps that exceed max_ioi.
    /// The returned vector is sorted in descendind order of pattern size.
    pub(crate) fn split_mtps_on_ioi<T: Point>(
        mtps: &Vec<(Mtp<T>, Vec<usize>, Vec<usize>)>,
        max_ioi: f64,
    ) -> Vec<(Pattern<T>, Vec<usize>, Vec<usize>)> {
        let mut split_mtps = Vec::new();

        for mtp_triple in mtps {
            let mtp = &mtp_triple.0;
            let split = SiatecC::split_pattern_on_ioi_gaps(
                &mtp.pattern,
                &mtp_triple.1,
                &mtp_triple.2,
                max_ioi,
            );
            for s in split {
                split_mtps.push(s);
            }
        }

        split_mtps.sort_by(|triple_a, triple_b| triple_b.0.len().cmp(&triple_a.0.len()));
        split_mtps
    }

    /// Partitions the forward diffs to MTPs and returns a vector of triples, where:
    /// 0. MTP
    /// 1. source indices: the indices that form the MTP
    /// 2. target indices: the indices of the points that form the translated MTP
    fn partition_to_mtps<T: Point>(
        point_set: &PointSet<T>,
        forward_diffs: &mut Vec<(T, IndPair)>,
    ) -> Vec<(Mtp<T>, Vec<usize>, Vec<usize>)> {
        // Sort and partition the diffs to find MTPs
        SiatecC::sort_with_ind_pairs(forward_diffs);

        let mut mtps: Vec<(Mtp<T>, Vec<usize>, Vec<usize>)> = Vec::new();

        let m = forward_diffs.len();
        let mut i = 0;
        while i < m {
            let mut source_indices: Vec<usize> = Vec::new();
            let mut target_indices: Vec<usize> = Vec::new();
            let translator = &forward_diffs[i].0;

            let mut j = i;
            while j < m && *translator == forward_diffs[j].0 {
                source_indices.push(forward_diffs[j].1[0]);
                target_indices.push(forward_diffs[j].1[1]);
                j += 1;
            }

            i = j;
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

    pub(crate) fn split_pattern_on_ioi_gaps<T: Point>(
        pattern: &Pattern<T>,
        source_ind: &Vec<usize>,
        target_ind: &Vec<usize>,
        max_ioi: f64,
    ) -> Vec<(Pattern<T>, Vec<usize>, Vec<usize>)> {
        let mut split_patterns = Vec::new();
        let mut split = Vec::new();
        let mut split_source_ind = Vec::new();
        let mut split_target_ind = Vec::new();
        let mut prev = &pattern[0];
        for i in 0..pattern.len() {
            let p = &pattern[i];
            let ioi = SiatecC::ioi(prev, p);
            if ioi > max_ioi {
                split_patterns.push((
                    Pattern::new(&split),
                    split_source_ind.clone(),
                    split_target_ind.clone(),
                ));
                split.clear();
                split_source_ind.clear();
                split_target_ind.clear();
            }
            split.push(p);
            split_source_ind.push(source_ind[i]);
            split_target_ind.push(target_ind[i]);
            prev = p;
        }

        // Handle any potentially remaining points.
        if !split.is_empty() {
            split_patterns.push((
                Pattern::new(&split),
                split_source_ind.clone(),
                split_target_ind.clone(),
            ));
        }
        split_patterns
    }

    fn find_indices<'a, T: Point>(
        diff_index: &'a Vec<(T, Vec<IndPair>)>,
        translation: &T,
    ) -> &'a Vec<IndPair> {
        let index_res = diff_index.binary_search_by(|t| t.0.cmp(translation));
        match index_res {
            Ok(index) => &diff_index[index].1,
            Err(index) => {
                println!(
                    "Could not find exact match for {:?}, returning closest to {}",
                    translation, index
                );
                if index >= diff_index.len() {
                    return &diff_index[diff_index.len() - 1].1;
                }

                &diff_index[index].1
            }
        }
    }

    pub(crate) fn find_single_point_translators_update_cover<T: Point>(
        pattern: &Pattern<T>,
        point_set: &PointSet<T>,
        cover: &mut Vec<usize>,
    ) -> Vec<T> {
        let mut translators = Vec::with_capacity(point_set.len() - 1);
        let pattern_point = pattern[0];

        for point in point_set {
            if *point != pattern_point {
                translators.push(*point - pattern_point);
            }
        }

        // Update cover
        for i in 0..cover.len() {
            if cover[i] == 0 {
                cover[i] = 1;
            }
        }

        translators
    }

    fn find_translators_update_cover<T: Point>(
        pattern: &Pattern<T>,
        diff_index: &Vec<(T, Vec<IndPair>)>,
        point_set: &PointSet<T>,
        cover: &mut Vec<usize>,
    ) -> Vec<T> {
        if pattern.len() == 1 {
            return SiatecC::find_single_point_translators_update_cover(pattern, point_set, cover);
        }

        let vectorized = pattern.vectorize();
        let v = &vectorized[0];

        let indices = SiatecC::find_indices(diff_index, v);
        let mut target_indices = Vec::with_capacity(indices.len());
        for ind_pair in indices.iter() {
            target_indices.push(ind_pair[1]);
        }

        for i in 1..vectorized.len() {
            let diff = &vectorized[i];
            let translatable_indices = SiatecC::find_indices(diff_index, diff);
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
        SiatecC::update_cover(pattern, diff_index, cover, &vectorized, target_indices);

        translators
    }

    fn update_cover<T: Point>(
        pattern: &Pattern<T>,
        diff_index: &Vec<(T, Vec<[usize; 2]>)>,
        cover: &mut Vec<usize>,
        vectorized: &Pattern<T>,
        init_cover_ind: Vec<usize>,
    ) {
        let mut cover_indices = init_cover_ind;

        for i in (0..vectorized.len()).rev() {
            let diff = &vectorized[i];
            let translatable_indices = SiatecC::find_indices(diff_index, diff);
            cover_indices =
                SiatecC::match_index_pairs_backward(&cover_indices, translatable_indices);

            for c in &cover_indices {
                cover[*c] = max(cover[*c], pattern.len());
            }
        }
    }

    pub(crate) fn match_index_pairs_forward(
        target_indices: &Vec<usize>,
        translatable_indices: &Vec<IndPair>,
    ) -> Vec<usize> {
        SiatecC::match_index_pairs(target_indices, translatable_indices, true)
    }

    pub(crate) fn match_index_pairs_backward(
        target_indices: &Vec<usize>,
        translatable_indices: &Vec<IndPair>,
    ) -> Vec<usize> {
        SiatecC::match_index_pairs(target_indices, translatable_indices, false)
    }

    //noinspection ALL
    fn match_index_pairs(
        target_indices: &Vec<usize>,
        index_pairs: &Vec<[usize; 2]>,
        forward: bool,
    ) -> Vec<usize> {
        let mut matching_ind = Vec::new();
        let mut j = 0;
        let mut k = 0;

        let test_index = if forward { 0 } else { 1 };
        let match_index = if forward { 1 } else { 0 };

        // Find all indices from which it's possible to continue translating
        // the points forward by a diff-vector from the pattern's vectorized representation.
        while j < target_indices.len() && k < index_pairs.len() {
            if target_indices[j] == index_pairs[k][test_index] {
                matching_ind.push(index_pairs[k][match_index]);
                j += 1;
                k += 1;
            } else if target_indices[j] < index_pairs[k][test_index] {
                j += 1;
            } else if target_indices[j] > index_pairs[k][test_index] {
                k += 1;
            }
        }

        matching_ind
    }

    pub fn remove_translational_duplicates<T: Point>(tecs: &mut Vec<Tec<T>>) {
        tecs.sort_by(|tec_a, tec_b| {
            let a = tec_a.pattern.vectorize();
            let b = tec_b.pattern.vectorize();

            let size_order = a.len().cmp(&b.len());
            if size_order == Ordering::Equal {
                return a.cmp(&b);
            }
            size_order
        });

        tecs.dedup_by(|a, b| a.pattern.vectorize() == b.pattern.vectorize())
    }

    fn sort_with_ind_pairs<T: Point>(diffs: &mut Vec<(T, IndPair)>) {
        diffs.sort_by(|a, b| {
            let ordering = a.0.cmp(&b.0);

            if ordering == Equal {
                a.1[0].cmp(&b.1[0])
            } else {
                ordering
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::TecAlgorithm;
    use crate::point_set::mtp::Mtp;
    use crate::point_set::pattern::Pattern;
    use crate::point_set::point::Point2Df64;
    use crate::point_set::point_set::PointSet;
    use crate::point_set::tec::Tec;
    use crate::siatec_c::SiatecC;

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
        let siatec_c = SiatecC { max_ioi: 2.0 };
        let mut tecs = siatec_c.compute_tecs(&point_set);
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
        let siatec_c = SiatecC { max_ioi: 2.0 };
        let mut tecs = siatec_c.compute_tecs(&point_set);

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
        let siatec_c = SiatecC { max_ioi: 2.0 };
        let mut tecs = siatec_c.compute_tecs(&point_set);

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

    #[test]
    fn test_splitting_on_ioi() {
        let mut mtp_triples: Vec<(Mtp<Point2Df64>, Vec<usize>, Vec<usize>)> = Vec::new();
        let max_ioi = 1.5;

        mtp_triples.push((
            Mtp {
                translator: Point2Df64 { x: 1.0, y: 1.0 },
                pattern: Pattern::new(&vec![
                    &Point2Df64 { x: 0.0, y: 0.0 },
                    &Point2Df64 { x: 1.0, y: 0.0 },
                    &Point2Df64 { x: 10.0, y: 0.0 },
                    &Point2Df64 { x: 11.0, y: 0.0 },
                ]),
            },
            vec![0, 1, 2, 3],
            vec![10, 11, 12, 13],
        ));

        mtp_triples.push((
            Mtp {
                translator: Point2Df64 { x: 0.0, y: 0.0 },
                pattern: Pattern::new(&vec![
                    &Point2Df64 { x: 100.0, y: 0.0 },
                    &Point2Df64 { x: 101.0, y: 0.0 },
                ]),
            },
            vec![100, 101],
            vec![110, 111],
        ));

        let split_triples = SiatecC::split_mtps_on_ioi(&mtp_triples, max_ioi);
        assert_eq!(3, split_triples.len());

        assert!(split_triples.contains(&(
            Pattern::new(&vec![
                &Point2Df64 { x: 0.0, y: 0.0 },
                &Point2Df64 { x: 1.0, y: 0.0 },
            ]),
            vec![0, 1],
            vec![10, 11]
        )));

        assert!(split_triples.contains(&(
            Pattern::new(&vec![
                &Point2Df64 { x: 10.0, y: 0.0 },
                &Point2Df64 { x: 11.0, y: 0.0 },
            ]),
            vec![2, 3],
            vec![12, 13]
        )));

        assert!(split_triples.contains(&(
            Pattern::new(&vec![
                &Point2Df64 { x: 100.0, y: 0.0 },
                &Point2Df64 { x: 101.0, y: 0.0 },
            ]),
            vec![100, 101],
            vec![110, 111]
        )));
    }
}
