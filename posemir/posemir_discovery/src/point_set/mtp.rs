/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use crate::point_set::pattern::Pattern;
use crate::point_set::point::Point;

/// Represents a Maximal Translatable Pattern (MTP) [Meredith et al. 2002].
/// An MTP is the set of all points in a point set D that can be
/// translated by a vector so that the translated points are also
/// within the point set D.
#[derive(Debug)]
pub struct Mtp<T: Point> {
    pub translator: T,
    pub pattern: Pattern<T>,
}

impl<T: Point> PartialEq for Mtp<T> {
    fn eq(&self, other: &Self) -> bool {
        self.translator == other.translator && self.pattern == other.pattern
    }
}

impl<T: Point> Eq for Mtp<T> {}
