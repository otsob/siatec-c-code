/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use std::cmp::Ordering;

use crate::algorithm::TecAlgorithm;
use crate::point_set::pattern::Pattern;
use crate::point_set::point::Point;
use crate::point_set::point_set::PointSet;
use crate::point_set::tec::Tec;
use crate::utilities::sort;

/// Implements the SIATEC algorithm for computing all translational equivalence classes (TECs) of
/// maximal translatable patterns (MTPs) in a point set (see [Meredith et al 2002]). The implementation
/// is based on the pseudocode in Figure 13.7 of [Meredith 2016] and on the description in [Meredith et al 2002]
/// that avoids computing TECs for duplicate MTPs. This implementation does not produces duplicate TECs
/// as avoiding computing translators for duplicates is considerably faster.
pub struct Siatec {}

impl<T: Point> TecAlgorithm<T> for Siatec {
    /// Returns all TECs of MTPs for the given point set.
    fn compute_tecs(&self, point_set: &PointSet<T>) -> Vec<Tec<T>> {
        let mut tecs = Vec::new();
        let on_output = |mtp: Tec<T>| tecs.push(mtp);
        self.compute_tecs_to_output(point_set, on_output);
        tecs
    }

    fn compute_tecs_to_output(&self, point_set: &PointSet<T>, mut on_output: impl FnMut(Tec<T>)) {
        let (diff_table, forward_diffs) = Siatec::compute_differences(point_set);

        let mut mtps_with_indices = Siatec::partition(point_set, &forward_diffs);
        let mtps = Siatec::remove_translational_duplicates(&mut mtps_with_indices);

        let n = point_set.len();

        // Compute the TECs by finding translators for each MTP
        for mtp_with_indices in &mtps {
            let translators = Siatec::find_translators(n, mtp_with_indices, &diff_table);
            on_output(Tec {
                pattern: mtp_with_indices.0.clone(),
                translators,
            });
        }
    }
}

impl Siatec {
    /// Initializes a size x size capacity table for differences.
    /// The table holds on the differences instead of also containing
    /// the indices as in the [Meredith et al. 2002] description.
    fn create_diff_table<T: Point>(size: usize) -> Vec<Vec<T>> {
        let mut diff_table: Vec<Vec<T>> = Vec::with_capacity(size);
        for _ in 0..size {
            diff_table.push(Vec::with_capacity(size));
        }

        diff_table
    }

    /// Computes the difference table and the forward differences with the indices required
    /// for MTP and translator computation.
    /// The forward differences are sorted in ascending lexicographical order.
    fn compute_differences<T: Point>(point_set: &PointSet<T>) -> (Vec<Vec<T>>, Vec<(T, usize)>) {
        let n = point_set.len();
        let mut diff_table = Siatec::create_diff_table(n);
        let mut forward_diffs: Vec<(T, usize)> = Vec::with_capacity(n * (n - 1) / 2);

        for i in 0..n {
            let from = &point_set[i];

            for j in 0..n {
                let to = &point_set[j];
                let diff = *to - *from;
                diff_table[i].push(diff);

                if i < j {
                    forward_diffs.push((diff, i));
                }
            }
        }

        sort(&mut forward_diffs);

        (diff_table, forward_diffs)
    }

    /// Partitions the sorted list of difference-index pairs into MTPs. The returned triples contain
    /// 0. the MTP pattern,
    /// 1. the vectorized representation of the pattern, and
    /// 2. the indices of the points belonging to the MTP.
    fn partition<T: Point>(
        point_set: &PointSet<T>,
        forward_diffs: &Vec<(T, usize)>,
    ) -> Vec<(Pattern<T>, Pattern<T>, Vec<usize>)> {
        let mut mtps_with_indices: Vec<(Pattern<T>, Pattern<T>, Vec<usize>)> = Vec::new();

        let m = forward_diffs.len();
        let mut i = 0;
        while i < m {
            let mut indices: Vec<usize> = Vec::new();
            let translator = &forward_diffs[i].0;

            let mut j = i;
            while j < m && *translator == forward_diffs[j].0 {
                indices.push(forward_diffs[j].1);
                j += 1;
            }

            i = j;
            let pattern = point_set.get_pattern(&indices);
            let vectorized = pattern.vectorize();
            mtps_with_indices.push((pattern, vectorized, indices));
        }
        mtps_with_indices
    }

    /// Remove duplication of translationally equivalent patterns.
    fn remove_translational_duplicates<T: Point>(
        mtps_with_indices: &mut Vec<(Pattern<T>, Pattern<T>, Vec<usize>)>,
    ) -> Vec<(&Pattern<T>, &Vec<usize>)> {
        // Sort by the vectorized representations so that translationally
        // equivalent patterns are adjacent.
        mtps_with_indices.sort_by(|a, b| {
            let size_order = a.1.len().cmp(&b.1.len());
            if size_order == Ordering::Equal {
                return a.1.cmp(&b.1);
            }
            size_order
        });

        // Store only the translationally distinct MTPs
        let mut distinct_mtps = Vec::new();
        let mut vec_representation = &mtps_with_indices[0].1;
        distinct_mtps.push((&mtps_with_indices[0].0, &mtps_with_indices[0].2));
        // Derefence+refence of mtps_with_indices is performed to ensure immutable reference is used.
        for mtp_triplet in &*mtps_with_indices {
            if mtp_triplet.1 != *vec_representation {
                distinct_mtps.push((&mtp_triplet.0, &mtp_triplet.2));
                vec_representation = &mtp_triplet.1;
            }
        }
        distinct_mtps
    }

    /// Finds all translators for the pattern in the given pattern-indices pair by using the difference
    /// table.
    fn find_translators<T: Point>(
        n: usize,
        mtp_indices: &(&Pattern<T>, &Vec<usize>),
        diff_table: &Vec<Vec<T>>,
    ) -> Vec<T> {
        let pattern = mtp_indices.0;
        let pat_len = pattern.len();
        // Column indices that correspond to the indices of the pattern in the point set.
        let col_ind = mtp_indices.1;

        let initial_value: usize = 0;

        // The row indices for the columns selected by the pattern's point indices.
        let mut row_ind = vec![initial_value; pat_len];

        let mut translators: Vec<T> = Vec::new();

        while row_ind[0] <= n - pat_len {
            for j in 1..pat_len {
                row_ind[j] = row_ind[0] + j;
            }

            let vec = diff_table[col_ind[0]][row_ind[0]];
            let mut found = false;

            for col in 1..pat_len {
                while row_ind[col] < n && diff_table[col_ind[col]][row_ind[col]] < vec {
                    row_ind[col] += 1;
                }

                if row_ind[col] >= n || vec != diff_table[col_ind[col]][row_ind[col]] {
                    break;
                }

                if col == pat_len - 1 {
                    found = true;
                }
            }

            if (found || pat_len == 1) && !vec.is_zero() {
                translators.push(vec);
            }

            row_ind[0] += 1;
        }

        translators
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::TecAlgorithm;
    use crate::point_set::pattern::Pattern;
    use crate::point_set::point::Point2Df64;
    use crate::point_set::point_set::PointSet;
    use crate::point_set::tec::Tec;
    use crate::siatec::Siatec;

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
        let siatec = Siatec {};
        let mut tecs = siatec.compute_tecs(&point_set);
        tecs.sort_by(|a, b| a.pattern.len().cmp(&b.pattern.len()));

        assert_eq!(3, tecs.len());
        assert_eq!(
            Tec {
                pattern: Pattern::new(&vec![&a]),
                translators: vec![
                    Point2Df64 { x: 1.0, y: 0.0 },
                    Point2Df64 { x: 2.0, y: 0.0 },
                    Point2Df64 { x: 3.0, y: 0.0 }
                ],
            },
            tecs[0]
        );
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
}
