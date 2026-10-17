/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use crate::algorithm::MtpAlgorithm;
use crate::point_set::mtp::Mtp;
use crate::point_set::point::Point;
use crate::point_set::point_set::PointSet;
use crate::utilities::sort;

/// Implements the SIA algorithm [Meredith et al. 2002].
/// The SIA algorithm computes all Maximal Translatable Patterns (MTP) in a
/// given point set.
pub struct Sia {}

impl<T: Point> MtpAlgorithm<T> for Sia {
    /// Computes and returns all MTPs in the given point set.
    ///
    /// # Arguments
    ///
    /// * `point_set` - The point set for which all MTPs are computed
    fn compute_mtps(&self, point_set: &PointSet<T>) -> Vec<Mtp<T>> {
        let forward_diffs = Sia::compute_differences(point_set);

        let mut mtps = Vec::new();
        let on_output = |mtp: Mtp<T>| mtps.push(mtp);
        Sia::partition(point_set, &forward_diffs, on_output);
        mtps
    }

    fn compute_mtps_to_output(&self, point_set: &PointSet<T>, on_output: impl FnMut(Mtp<T>)) {
        let forward_diffs = Sia::compute_differences(point_set);
        Sia::partition(point_set, &forward_diffs, on_output);
    }
}

impl Sia {
    /// Computes the forward differences with the indices required
    /// for MTP computation.
    /// The forward differences are sorted in ascending lexicographical order.
    fn compute_differences<T: Point>(point_set: &PointSet<T>) -> Vec<(T, usize)> {
        let n = point_set.len();
        let mut diffs: Vec<(T, usize)> = Vec::with_capacity(n * (n - 1) / 2);

        for i in 0..n - 1 {
            let from = &point_set[i];
            for j in i + 1..n {
                let to = &point_set[j];
                diffs.push((*to - *from, i));
            }
        }

        sort(&mut diffs);
        diffs
    }

    /// Partitions the sorted list of difference-index pairs into MTPs.
    fn partition<T: Point>(
        point_set: &PointSet<T>,
        forward_diffs: &Vec<(T, usize)>,
        mut on_output: impl FnMut(Mtp<T>),
    ) {
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
            on_output(Mtp {
                translator: *translator,
                pattern: point_set.get_pattern(&indices),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::MtpAlgorithm;
    use crate::point_set::mtp::Mtp;
    use crate::point_set::pattern::Pattern;
    use crate::point_set::point::Point2Df64;
    use crate::point_set::point_set::PointSet;
    use crate::sia::Sia;

    const ALGORITHM: Sia = Sia {};

    #[test]
    fn test_minimal_number_of_mtps() {
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
        let mut mtps = ALGORITHM.compute_mtps(&point_set);
        mtps.sort_by(|a, b| a.translator.cmp(&b.translator));

        assert_eq!(3, mtps.len());
        assert_eq!(
            mtps[0],
            Mtp {
                translator: Point2Df64 { x: 1.0, y: 0.0 },
                pattern: Pattern::new(&vec![&a, &b, &c])
            }
        );
        assert_eq!(
            mtps[1],
            Mtp {
                translator: Point2Df64 { x: 2.0, y: 0.0 },
                pattern: Pattern::new(&vec![&a, &b])
            }
        );
        assert_eq!(
            mtps[2],
            Mtp {
                translator: Point2Df64 { x: 3.0, y: 0.0 },
                pattern: Pattern::new(&vec![&a])
            }
        );
    }

    #[test]
    fn test_maximal_number_of_mtps() {
        // Create point set where all MTPs only have one point (maximize number of MTPs).
        let mut points = Vec::new();
        let a = Point2Df64 { x: 1.0, y: 1.0 };
        points.push(a);
        let b = Point2Df64 { x: 2.0, y: 2.0 };
        points.push(b);
        let c = Point2Df64 { x: 3.0, y: 4.0 };
        points.push(c);

        let point_set = PointSet::new(points);
        let mut mtps = ALGORITHM.compute_mtps(&point_set);
        mtps.sort_by(|a, b| a.translator.cmp(&b.translator));

        assert_eq!(3, mtps.len());
        assert_eq!(
            mtps[0],
            Mtp {
                translator: Point2Df64 { x: 1.0, y: 1.0 },
                pattern: Pattern::new(&vec![&a])
            }
        );
        assert_eq!(
            mtps[1],
            Mtp {
                translator: Point2Df64 { x: 1.0, y: 2.0 },
                pattern: Pattern::new(&vec![&b])
            }
        );
        assert_eq!(
            mtps[2],
            Mtp {
                translator: Point2Df64 { x: 2.0, y: 3.0 },
                pattern: Pattern::new(&vec![&a])
            }
        );
    }
}
