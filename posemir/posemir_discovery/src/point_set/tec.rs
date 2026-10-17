/*
 * (c) Otso Björklund (2021)
 * Distributed under the MIT license (see LICENSE.txt or https://opensource.org/licenses/MIT).
 */
use crate::point_set::pattern::Pattern;
use crate::point_set::point::Point;
use crate::point_set::point_set::PointSet;

/// Represents a translational equivalence class (see [Meredith et al. 2002]).
/// A TEC consists of a pattern and all of its translationally equivalent occurrences in a point set.
/// TECs are represented as a pattern and the translators by which it can be translated
/// to produce all of the translationally equivalent occurrences. The translators do *not* contain
/// the zero vector.
#[derive(Debug, Clone)]
pub struct Tec<T: Point> {
    pub pattern: Pattern<T>,
    pub translators: Vec<T>,
}

impl<T: Point> Tec<T> {
    /// Returns the expansion of this TEC.
    ///
    /// The TEC is expanded by creating all translated copies of the pattern.
    pub fn expand(&self) -> Vec<Pattern<T>> {
        let mut occurrences = Vec::with_capacity(self.translators.len() + 1);
        occurrences.push(self.pattern.clone());

        for translator in &self.translators {
            occurrences.push(self.pattern.translate(translator));
        }

        occurrences
    }

    /// Returns the set of points covered by this TEC.
    pub fn covered_set(&self) -> PointSet<T> {
        let expanded = self.expand();
        let mut points = Vec::new();
        for pattern in &expanded {
            for point in pattern {
                points.push(*point);
            }
        }

        PointSet::new(points)
    }

    /// Returns the conjugate TEC of this TEC (see [Meredith2013]).
    pub fn conjugate(&self) -> Tec<T> {
        let first = self.pattern[0];
        let mut conj_pat_points = vec![first];

        for translator in &self.translators {
            conj_pat_points.push(first + *translator);
        }

        let mut pattern_points = Vec::new();
        for p in &conj_pat_points {
            pattern_points.push(p);
        }

        let mut translators = Vec::new();
        for i in 1..self.pattern.len() {
            let p = &self.pattern[i];
            translators.push(*p - first);
        }

        Tec {
            pattern: Pattern::new(&pattern_points),
            translators,
        }
    }

    /// Returns a TEC with all redundant translators removed.
    /// A translator is redundant if it can be removed without affecting the
    /// covered set of the TEC.
    pub fn remove_redundant_translators(&self) -> Tec<T> {
        let covered_set = self.covered_set();
        let mut translators = Vec::new();

        let mut cleaned_translators = self.translators.clone();
        cleaned_translators.sort();
        cleaned_translators.dedup();

        for i in 0..cleaned_translators.len() {
            let mut transl_copy = cleaned_translators.clone();
            transl_copy.remove(i);

            let cov = Tec {
                pattern: self.pattern.clone(),
                translators: transl_copy,
            }
            .covered_set();

            if cov != covered_set {
                translators.push(cleaned_translators[i]);
            }
        }

        Tec {
            pattern: self.pattern.clone(),
            translators,
        }
    }
}

impl<T: Point> PartialEq for Tec<T> {
    fn eq(&self, other: &Self) -> bool {
        self.translators == other.translators && self.pattern == other.pattern
    }
}

impl<T: Point> Eq for Tec<T> {}

#[cfg(test)]
mod tests {
    use crate::point_set::pattern::Pattern;
    use crate::point_set::point::Point2Df64;
    use crate::point_set::tec::Tec;

    #[test]
    fn test_covered_set() {
        let pattern = Pattern::new(&vec![
            &Point2Df64 { x: 1.0, y: 0.0 },
            &Point2Df64 { x: 2.0, y: 0.0 },
        ]);
        let translators = vec![Point2Df64 { x: 1.0, y: 0.0 }, Point2Df64 { x: 1.0, y: 1.0 }];
        let tec = Tec {
            pattern,
            translators,
        };

        let cov = tec.covered_set();

        assert_eq!(5, cov.len());
        assert_eq!(Point2Df64 { x: 1.0, y: 0.0 }, cov[0]);
        assert_eq!(Point2Df64 { x: 2.0, y: 0.0 }, cov[1]);
        assert_eq!(Point2Df64 { x: 2.0, y: 1.0 }, cov[2]);
        assert_eq!(Point2Df64 { x: 3.0, y: 0.0 }, cov[3]);
        assert_eq!(Point2Df64 { x: 3.0, y: 1.0 }, cov[4]);
    }

    #[test]
    fn test_conjugate() {
        let a = Point2Df64 { x: 1.0, y: 1.0 };
        let b = Point2Df64 { x: 2.0, y: 1.0 };
        let c = Point2Df64 { x: 3.0, y: 1.0 };

        let pattern = Pattern::new(&vec![&a, &b, &c]);
        let t_a = Point2Df64 { x: 1.0, y: 0.0 };
        let t_b = Point2Df64 { x: 1.0, y: 1.0 };
        let translators = vec![t_a, t_b];
        let tec = Tec {
            pattern,
            translators,
        };

        let conj = tec.conjugate();
        assert_eq!(
            Pattern::new(&vec![&a, &b, &Point2Df64 { x: 2.0, y: 2.0 }]),
            conj.pattern
        );

        assert_eq!(
            vec![Point2Df64 { x: 1.0, y: 0.0 }, Point2Df64 { x: 2.0, y: 0.0 }],
            conj.translators
        );
    }

    #[test]
    fn test_remove_redundant_translators() {
        let a = Point2Df64 { x: 1.0, y: 1.0 };
        let b = Point2Df64 { x: 2.0, y: 1.0 };
        let c = Point2Df64 { x: 3.0, y: 1.0 };

        let pattern = Pattern::new(&vec![&a, &b, &c]);
        let t_a = Point2Df64 { x: 0.0, y: 0.0 };
        let t_b = Point2Df64 { x: 1.0, y: 1.0 };
        let t_c = Point2Df64 { x: 1.0, y: 1.0 };

        let translators = vec![t_a, t_b, t_c];
        let tec = Tec {
            pattern,
            translators,
        };

        let without_redundant_transl = tec.remove_redundant_translators();
        assert_eq!(
            Pattern::new(&vec![&a, &b, &c]),
            without_redundant_transl.pattern
        );
        assert_eq!(vec![t_b], without_redundant_transl.translators);
    }
}
