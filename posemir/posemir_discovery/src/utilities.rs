/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use std::cmp::Ordering::Equal;

use crate::point_set::point::Point;

/// Sorts the given pairs into ascending lexicographical order by
/// first comparing the first elements, and comparing the second elements only
/// if the first elements are equal.
pub fn sort<T: Point>(diffs: &mut Vec<(T, usize)>) {
    diffs.sort_by(|a, b| {
        let ordering = a.0.cmp(&b.0);

        if ordering == Equal {
            a.1.cmp(&b.1)
        } else {
            ordering
        }
    });
}
