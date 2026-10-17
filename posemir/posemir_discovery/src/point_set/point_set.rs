/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use std::borrow::Borrow;
use std::ops::Index;
use std::slice;

use crate::point_set::pattern::Pattern;
use crate::point_set::point::Point;

/// Represents a sorted set of points (i.e. vectors).
/// The points in the set are in lexicographical order.
#[derive(Debug, Clone)]
pub struct PointSet<T: Point> {
    points: Vec<T>,
}

impl<T: Point> PointSet<T> {
    /// Returns a point set created from the given points.
    /// The given points do not have to be in any specific order, they are sorted
    /// when the point set is created. Point sets are sets in the sense that they
    /// do not contain duplicates, that is, duplicate points are removed on creation.
    ///
    /// # Arguments
    ///
    /// * `points` - A vector of points. The returned point set takes ownership of the points.
    ///
    pub fn new(mut points: Vec<T>) -> PointSet<T> {
        points.sort();
        points.dedup();
        PointSet { points }
    }

    /// Returns and gives ownership of the points from this point set.
    pub fn points(self) -> Vec<T> {
        self.points
    }

    //noinspection ALL
    /// Returns the number of points in this point set
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Returns a pattern consisting of points at the given indices.
    /// # Arguments
    ///
    /// * `indices` - The indices for the points that form the returned pattern
    pub fn get_pattern(&self, indices: &Vec<usize>) -> Pattern<T> {
        Pattern::new(
            &indices
                .iter()
                .map(|i| &self.points[*i])
                .collect::<Vec<&T>>(),
        )
    }

    /// Returns a point set translated by the given vector.
    ///
    /// # Arguments
    ///
    /// * `translator` - The translator by which the returned point set is translated
    pub fn translate(&self, translator: &T) -> PointSet<T> {
        let mut translated_points = Vec::with_capacity(self.len());
        for point in &self.points {
            translated_points.push(*point + *translator);
        }

        PointSet {
            points: translated_points,
        }
    }

    /// Returns the intersection of this point set with the given point set.
    ///
    /// # Arguments
    ///
    /// * `other` - The point set with which this point set is intersected
    pub fn intersect(&self, other: &PointSet<T>) -> PointSet<T> {
        let mut common_points = Vec::new();

        let mut i = 0;
        let mut j = 0;

        while i < self.len() && j < other.len() {
            let a = &self[i];
            let b = &other[j];

            if a == b {
                common_points.push(*a);
                i += 1;
                j += 1;
            } else if a > b {
                j += 1;
            } else {
                i += 1;
            }
        }

        PointSet {
            points: common_points,
        }
    }

    /// Returns the difference of this point set and the other point set (all points in this
    /// that are not present in other).
    ///
    /// # Arguments
    ///
    /// * `other` - The point set whose points are removed from this to produce the returned set
    pub fn difference(&self, other: &PointSet<T>) -> PointSet<T> {
        let mut diff = Vec::new();

        let mut i = 0;
        let mut j = 0;

        while i < self.len() && j < other.len() {
            let a = &self[i];
            let b = &other[j];

            if a == b {
                i += 1;
                j += 1;
            } else if a > b {
                j += 1;
            } else {
                diff.push(self[i]);
                i += 1;
            }
        }

        if i < self.len() && j == other.len() {
            for i in i..self.len() {
                diff.push(self[i]);
            }
        }

        PointSet { points: diff }
    }

    pub fn find_index(&self, point: &T) -> Result<usize, usize> {
        self.points.binary_search(point)
    }

    pub fn union(&self, point_set: &PointSet<T>) -> PointSet<T> {
        let mut points = self.points.clone();
        points.append(&mut point_set.points.clone());

        PointSet::new(points)
    }
}

impl<T: Point> Index<usize> for PointSet<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.points[index].borrow()
    }
}

impl<'a, T: Point> IntoIterator for &'a PointSet<T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.iter()
    }
}

impl<T: Point> PartialEq for PointSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.points == other.points
    }
}

#[cfg(test)]
mod tests {
    use crate::point_set::point::Point2Df64;
    use crate::point_set::point_set::PointSet;

    #[test]
    fn test_constructor_and_access() {
        let points = vec![
            Point2Df64 { x: 2.1, y: 0.1 },
            Point2Df64 { x: -1.0, y: 0.0 },
            Point2Df64 { x: -1.0, y: 0.0 },
            Point2Df64 { x: -1.0, y: 0.5 },
        ];
        let point_set = PointSet::new(points);

        assert_eq!(3, point_set.len());
        assert_eq!(Point2Df64 { x: -1.0, y: 0.0 }, point_set[0]);
        assert_eq!(Point2Df64 { x: -1.0, y: 0.5 }, point_set[1]);
        assert_eq!(Point2Df64 { x: 2.1, y: 0.1 }, point_set[2]);
    }

    #[test]
    fn test_iteration() {
        let points = vec![
            Point2Df64 { x: 2.1, y: 0.1 },
            Point2Df64 { x: -1.0, y: 0.0 },
            Point2Df64 { x: -1.0, y: 0.5 },
            Point2Df64 { x: -2.0, y: 0.5 },
        ];

        let mut sorted_points = points.to_vec();
        sorted_points.sort();

        let point_set = PointSet::new(points);

        for (i, point) in point_set.into_iter().enumerate() {
            assert_eq!(sorted_points[i], *point);
        }
    }

    #[test]
    fn test_get_pattern() {
        let points = vec![
            Point2Df64 { x: 2.1, y: 0.1 },
            Point2Df64 { x: -1.0, y: 0.0 },
            Point2Df64 { x: -1.0, y: 0.5 },
            Point2Df64 { x: -2.0, y: 0.5 },
        ];

        let mut sorted_points = points.to_vec();
        sorted_points.sort();

        let point_set = PointSet::new(points);

        let pattern = point_set.get_pattern(&vec![0, 3]);
        assert_eq!(2, pattern.len());
        assert_eq!(sorted_points[0], pattern[0]);
        assert_eq!(sorted_points[3], pattern[1]);
    }

    #[test]
    fn test_intersect() {
        let points = vec![
            Point2Df64 { x: 1.0, y: 1.0 },
            Point2Df64 { x: 2.0, y: 1.0 },
            Point2Df64 { x: 3.0, y: 2.0 },
            Point2Df64 { x: 4.0, y: 2.0 },
        ];

        let point_set_a = PointSet::new(points);
        let point_set_b = point_set_a.translate(&(Point2Df64 { x: 2.0, y: 1.0 } * -1.0));

        let intersection = point_set_a.intersect(&point_set_b);

        assert_eq!(2, intersection.len());
        assert_eq!(Point2Df64 { x: 1.0, y: 1.0 }, intersection[0]);
        assert_eq!(Point2Df64 { x: 2.0, y: 1.0 }, intersection[1]);
    }

    #[test]
    fn test_difference() {
        let point_set_a = PointSet::new(vec![
            Point2Df64 { x: 1.0, y: 1.0 },
            Point2Df64 { x: 2.0, y: 1.0 },
            Point2Df64 { x: 3.0, y: 2.0 },
            Point2Df64 { x: 4.0, y: 2.0 },
        ]);

        let point_set_b = PointSet::new(vec![
            Point2Df64 { x: 2.0, y: 1.0 },
            Point2Df64 { x: 3.0, y: 1.0 },
            Point2Df64 { x: 3.0, y: 2.0 },
            Point2Df64 { x: 4.0, y: 2.5 },
        ]);

        let diff = point_set_a.difference(&point_set_b);
        assert_eq!(2, diff.len());
        assert_eq!(Point2Df64 { x: 1.0, y: 1.0 }, diff[0]);
        assert_eq!(Point2Df64 { x: 4.0, y: 2.0 }, diff[1]);
    }
}
