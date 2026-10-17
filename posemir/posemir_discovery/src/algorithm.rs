/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use crate::point_set::mtp::Mtp;
use crate::point_set::point::Point;
use crate::point_set::point_set::PointSet;
use crate::point_set::tec::Tec;

/// Trait that defines an algorithm that computes MTPs from a point set.
pub trait MtpAlgorithm<T: Point> {
    /// Returns the MTPs in the given point set. Whether all MTPs are returned
    /// depends on the algorithm. This algorithms will collect all output to
    /// a vector in memory, so when using large point sets it is better to use
    /// `compute_mtps_to_output`.
    ///
    /// # Arguments
    ///
    /// * `point_set` - the set of points for which MTPs are computed
    fn compute_mtps(&self, point_set: &PointSet<T>) -> Vec<Mtp<T>>;

    /// Computes MTPs in the given point set and executes on_output for
    /// each produced MTP. For large outputs that should not be kept in
    /// memory this function should be used.
    ///
    /// # Arguments
    ///
    /// * `point_set` - the set of points for which MTPs are computed
    /// * `on_output` - a function to execute whenever the algorithm can produce output
    fn compute_mtps_to_output(&self, point_set: &PointSet<T>, on_output: impl FnMut(Mtp<T>));
}

/// Trait for algorithms that compute TECs in a point set.
pub trait TecAlgorithm<T: Point> {
    /// Returns the TECs in the given point set.
    /// The patterns for which the TECs are returned depends on the algorithm.
    /// This algorithms will collect all output to
    /// a vector in memory, so when using large point sets it is better to use
    /// `compute_tecs_to_output`.
    ///
    /// # Arguments
    ///
    /// * `point_set` - the set of points for which TECs are computed
    fn compute_tecs(&self, point_set: &PointSet<T>) -> Vec<Tec<T>>;


    /// Computes TECs in the given point set and executes on_output for
    /// each produced TEC. For large outputs that should not be kept in
    /// memory this function should be used.
    ///
    /// # Arguments
    ///
    /// * `point_set` - the set of points for which TECs are computed
    /// * `on_output` - a function to execute whenever the algorithm can produce output
    fn compute_tecs_to_output(&self, point_set: &PointSet<T>, on_output: impl FnMut(Tec<T>));
}
