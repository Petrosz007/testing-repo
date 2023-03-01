use std::fmt;

pub trait Intersectable {
    fn intersects_with(&self, other: &Self) -> bool;

    fn intersect(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Boundary {
    Open,
    Closed,
}

impl Boundary {
    pub fn inverse(&self) -> Self {
        match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }
}

/// Represents one interval with boundaries, a low value and a high value
#[derive(PartialEq, Clone, Copy)]
pub struct Interval {
    pub lo_boundary: Boundary,
    pub lo: f32,
    pub hi: f32,
    pub hi_boundary: Boundary,
}

impl Interval {
    fn contains_point(&self, point: f32) -> bool {
        (self.lo < point && point < self.hi)
            || (self.lo == point && self.lo_boundary == Boundary::Closed)
            || (self.hi == point && self.hi_boundary == Boundary::Closed)
    }

    pub fn new(
        lo_boundary: Boundary,
        lo: f32,
        hi: f32,
        hi_boundary: Boundary,
    ) -> Result<Self, IntervalError> {
        if lo > hi {
            Err(IntervalError::LoIsGreaterThanHi)
        } else {
            Ok(Self {
                lo_boundary,
                lo,
                hi,
                hi_boundary,
            })
        }
    }

    pub fn new_closed(lo: f32, hi: f32) -> Result<Self, IntervalError> {
        Self::new(Boundary::Closed, lo, hi, Boundary::Closed)
    }

    #[must_use]
    pub const fn new_closed_point(point: f32) -> Self {
        Self {
            lo_boundary: Boundary::Closed,
            lo: point,
            hi: point,
            hi_boundary: Boundary::Closed,
        }
    }
}

impl Intersectable for Interval {
    fn intersects_with(&self, other: &Self) -> bool {
        let doesnt_intersect = (self.lo > other.hi || other.lo > self.hi)
            || self.lo == other.hi
                && (self.lo_boundary == Boundary::Open || other.hi_boundary == Boundary::Open)
            || other.lo == self.hi
                && (other.lo_boundary == Boundary::Open || self.hi_boundary == Boundary::Open);

        !doesnt_intersect
    }

    fn intersect(&self, other: &Self) -> Option<Self> {
        if !self.intersects_with(other) {
            return None;
        }

        let bigger_lo = if self.lo > other.lo { self } else { other };
        let smaller_hi = if self.hi < other.hi { self } else { other };

        Some(Self {
            lo_boundary: bigger_lo.lo_boundary,
            lo: bigger_lo.lo,
            hi: smaller_hi.hi,
            hi_boundary: smaller_hi.hi_boundary,
        })
    }
}

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lo_boundary = match self.lo_boundary {
            Boundary::Open => "(",
            Boundary::Closed => "[",
        };
        let hi_boundary = match self.hi_boundary {
            Boundary::Open => ")",
            Boundary::Closed => "]",
        };

        write!(f, "{}{}, {}{}", lo_boundary, self.lo, self.hi, hi_boundary)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct MultiInterval {
    /// `intervals` is always sorted in ascending order and there are no overlapping intervals
    intervals: Vec<Interval>,
}

// TODO: There could be a more pragmatic rust solution
#[derive(Debug)]
pub enum IntervalError {
    LoIsGreaterThanHi,
}

// TODO: implement a simplifier function, which
//          - removes empty intervals, like (0,0)
//          - merges bordering intervals, like [10, 20] [20, 30] becomes [10, 30]
impl MultiInterval {
    pub fn new(
        lo_boundary: Boundary,
        lo: f32,
        hi: f32,
        hi_boundary: Boundary,
    ) -> Result<Self, IntervalError> {
        Ok(Self {
            intervals: vec![Interval::new(lo_boundary, lo, hi, hi_boundary)?],
        })
    }

    pub fn new_closed(lo: f32, hi: f32) -> Result<Self, IntervalError> {
        Self::new(Boundary::Closed, lo, hi, Boundary::Closed)
    }

    #[must_use]
    pub fn highest_hi(&self) -> f32 {
        self.intervals
            .last()
            .expect("Interval should always contain an interval")
            .hi
    }

    #[must_use]
    pub fn lowest_lo(&self) -> f32 {
        self.intervals
            .first()
            .expect("Interval should always contain an interval")
            .lo
    }

    #[must_use]
    pub fn highest_boundary(&self) -> Boundary {
        self.intervals
            .last()
            .expect("Interval should always contain an interval")
            .hi_boundary
    }

    #[must_use]
    pub fn lowest_boundary(&self) -> Boundary {
        self.intervals
            .first()
            .expect("Interval should always contain an interval")
            .lo_boundary
    }

    #[must_use]
    pub fn DONOTUSE_get_interval(&self) -> Interval {
        self.intervals[0]
    }

    pub fn inverse(&self) -> Self {
        if self.intervals.is_empty() {
            return Self {
                intervals: vec![Interval {
                    lo_boundary: Boundary::Open,
                    lo: f32::NEG_INFINITY,
                    hi: f32::INFINITY,
                    hi_boundary: Boundary::Open,
                }],
            };
        }

        let mut new_intervals = Vec::new();

        if self.lowest_lo() != f32::NEG_INFINITY {
            new_intervals.push(Interval {
                lo_boundary: Boundary::Open,
                lo: f32::NEG_INFINITY,
                hi: self.lowest_lo(),
                hi_boundary: self.lowest_boundary().inverse(),
            })
        }

        new_intervals.append(
            &mut self
                .intervals
                .windows(2)
                .map(|x| {
                    let (a, b) = (x[0], x[1]);

                    Interval {
                        lo_boundary: a.hi_boundary.inverse(),
                        lo: a.hi,
                        hi: b.lo,
                        hi_boundary: b.lo_boundary.inverse(),
                    }
                })
                .collect(),
        );

        if self.highest_hi() != f32::INFINITY {
            new_intervals.push(Interval {
                lo_boundary: self.highest_boundary().inverse(),
                lo: self.highest_hi(),
                hi: f32::INFINITY,
                hi_boundary: Boundary::Open,
            })
        }

        Self {
            intervals: new_intervals,
        }
    }
}

impl Intersectable for MultiInterval {
    // TODO: This could be sped up, because the interval Vecs are sorted
    // It could be a step-by-step comparison
    fn intersects_with(&self, other: &Self) -> bool {
        for x in &self.intervals {
            for y in &other.intervals {
                if x.intersects_with(y) {
                    return true;
                }
            }
        }

        false
    }

    fn intersect(&self, other: &Self) -> Option<Self> {
        let mut intersected_intervals: Vec<Interval> = self
            .intervals
            .iter()
            .flat_map(|x| other.intervals.iter().map(|y| x.intersect(y)))
            .flatten()
            .collect();

        intersected_intervals.sort_unstable_by(|a, b| {
            a.lo.partial_cmp(&b.lo)
                .expect("f32::NaN should not be the lo value of intervals")
        });

        if intersected_intervals.is_empty() {
            None
        } else {
            Some(Self {
                intervals: intersected_intervals,
            })
        }
    }
}

#[cfg(test)]
pub(crate) mod test {
    use nom::{combinator::complete, multi::many0};

    use super::Interval;
    use crate::{
        interval::{Intersectable, MultiInterval},
        parser::interval,
    };

    pub fn int(input: &str) -> Interval {
        let (_, x) = interval(input).unwrap();
        *x.intervals.first().unwrap()
    }

    pub fn multiint(input: &str) -> MultiInterval {
        let (_, x) = many0(complete(interval))(input.trim()).unwrap();
        MultiInterval {
            intervals: x
                .into_iter()
                .map(|y| *y.intervals.first().unwrap())
                .collect(),
        }
    }

    #[test]
    fn test_contains_point() {
        let test_cases = vec![
            (int("[5, 10]"), 4.0, false),
            (int("(5, 10]"), 5.0, false),
            (int("[5, 10]"), 5.0, true),
            (int("[5, 10]"), 7.0, true),
            (int("[5, 10]"), 10.0, true),
            (int("[5, 10)"), 10.0, false),
            (int("[5, 10)"), 11.0, false),
        ];

        for (interval, point, expected) in test_cases {
            assert_eq!(
                interval.contains_point(point),
                expected,
                "Interval.contains_point failed: {interval:?}.contains_point({point:?}) should be {expected:?}",
            );
        }
    }

    #[test]
    fn test_Interval_intersects_with() {
        let test_cases = vec![
            // self.hi equals other.lo
            (int("[0, 10]"), int("[10, 20]"), true),
            (int("[0, 10]"), int("(10, 20]"), false),
            (int("[0, 10)"), int("[10, 20]"), false),
            (int("[0, 10)"), int("(10, 20]"), false),
            // self.lo equals other.hi
            (int("[10, 20]"), int("[0, 10]"), true),
            (int("(10, 20]"), int("[0, 10]"), false),
            (int("[10, 20]"), int("[0, 10)"), false),
            (int("(10, 20]"), int("[0, 10)"), false),
            // self.hi inside other == other.lo inside self
            (int("[0, 10]"), int("[5, 20]"), true),
            (int("[0, 10]"), int("(5, 20]"), true),
            (int("[0, 10)"), int("[5, 20]"), true),
            (int("[0, 10)"), int("(5, 20]"), true),
            // self.lo inside other == other.hi inside self
            (int("[5, 20]"), int("[0, 10]"), true),
            (int("(5, 20]"), int("[0, 10]"), true),
            (int("[5, 20]"), int("[0, 10)"), true),
            (int("(5, 20]"), int("[0, 10)"), true),
            // self inside other
            (int("[10, 20]"), int("[0, 30]"), true),
            (int("[10, 20)"), int("[0, 30]"), true),
            (int("(10, 20]"), int("[0, 30]"), true),
            (int("(10, 20)"), int("[0, 30]"), true),
            // other inside self
            (int("[0, 30]"), int("[10, 20]"), true),
            (int("[0, 30]"), int("[10, 20)"), true),
            (int("[0, 30]"), int("(10, 20]"), true),
            (int("[0, 30]"), int("(10, 20)"), true),
            // self.lo > other.hi
            (int("[20, 30]"), int("[0, 10]"), false),
            (int("[20, 30]"), int("[0, 10)"), false),
            (int("(20, 30]"), int("[0, 10]"), false),
            (int("(20, 30]"), int("[0, 10)"), false),
            // other.lo > self.hi
            (int("[0, 10]"), int("[20, 30]"), false),
            (int("[0, 10)"), int("[20, 30]"), false),
            (int("[0, 10]"), int("(20, 30]"), false),
            (int("[0, 10)"), int("(20, 30]"), false),
            // TODO: Inf, -Inf
        ];

        for (this, that, expected) in test_cases {
            assert_eq!(
                this.intersects_with(&that),
                expected,
                "Interval.intersects_with failed: {this:?}.intersects_with({that:?}) should be {expected:?}",
            );
        }
    }

    #[test]
    fn test_Interval_intersect() {
        let test_cases = vec![
            // self.hi equals other.lo
            (int("[0, 10]"), int("[10, 20]"), Some(int("[10, 10]"))),
            (int("[0, 10]"), int("(10, 20]"), None),
            (int("[0, 10)"), int("[10, 20]"), None),
            (int("[0, 10)"), int("(10, 20]"), None),
            // self.lo equals other.hi
            (int("[10, 20]"), int("[0, 10]"), Some(int("[10, 10]"))),
            (int("(10, 20]"), int("[0, 10]"), None),
            (int("[10, 20]"), int("[0, 10)"), None),
            (int("(10, 20]"), int("[0, 10)"), None),
            // self.hi inside other == other.lo inside self
            (int("[0, 10]"), int("[5, 20]"), Some(int("[5, 10]"))),
            (int("[0, 10]"), int("(5, 20]"), Some(int("(5, 10]"))),
            (int("[0, 10)"), int("[5, 20]"), Some(int("[5, 10)"))),
            (int("[0, 10)"), int("(5, 20]"), Some(int("(5, 10)"))),
            // self.lo inside other == other.hi inside self
            (int("[5, 20]"), int("[0, 10]"), Some(int("[5, 10]"))),
            (int("(5, 20]"), int("[0, 10]"), Some(int("(5, 10]"))),
            (int("[5, 20]"), int("[0, 10)"), Some(int("[5, 10)"))),
            (int("(5, 20]"), int("[0, 10)"), Some(int("(5, 10)"))),
            // self inside other
            (int("[10, 20]"), int("[0, 30]"), Some(int("[10, 20]"))),
            (int("[10, 20)"), int("[0, 30]"), Some(int("[10, 20)"))),
            (int("(10, 20]"), int("[0, 30]"), Some(int("(10, 20]"))),
            (int("(10, 20)"), int("[0, 30]"), Some(int("(10, 20)"))),
            // other inside self
            (int("[0, 30]"), int("[10, 20]"), Some(int("[10, 20]"))),
            (int("[0, 30]"), int("[10, 20)"), Some(int("[10, 20)"))),
            (int("[0, 30]"), int("(10, 20]"), Some(int("(10, 20]"))),
            (int("[0, 30]"), int("(10, 20)"), Some(int("(10, 20)"))),
            // self.lo > other.hi
            (int("[20, 30]"), int("[0, 10]"), None),
            (int("[20, 30]"), int("[0, 10)"), None),
            (int("(20, 30]"), int("[0, 10]"), None),
            (int("(20, 30]"), int("[0, 10)"), None),
            // other.lo > self.hi
            (int("[0, 10]"), int("[20, 30]"), None),
            (int("[0, 10)"), int("[20, 30]"), None),
            (int("[0, 10]"), int("(20, 30]"), None),
            (int("[0, 10)"), int("(20, 30]"), None),
            // TODO: Inf, -Inf
        ];

        for (this, that, expected) in test_cases {
            assert_eq!(
                this.intersect(&that),
                expected,
                "Interval.intersect failed: {this:?}.intersect({that:?}) should be {expected:?}",
            );
        }
    }

    #[test]
    fn test_MultiInterval_intersect() {
        let test_cases = vec![
            // zero elements
            (vec![], vec![], None),
            // one elem - zero elem
            (vec![int("[0, 10]")], vec![], None),
            (vec![], vec![int("[0, 10]")], None),
            // one element, has intersection
            (
                vec![int("[0, 10]")],
                vec![int("[5, 20]")],
                Some(vec![int("[5, 10]")]),
            ),
            // one element - two elements, has intersection
            (
                vec![int("[0, 10]")],
                vec![int("[5, 20]"), int("[100, 200]")],
                Some(vec![int("[5, 10]")]),
            ),
            (
                vec![int("[5, 20]"), int("[100, 200]")],
                vec![int("[0, 10]")],
                Some(vec![int("[5, 10]")]),
            ),
            // contains multiple intervals
            (
                vec![int("[0, 100]")],
                vec![int("[10, 20]"), int("[30, 40]")],
                Some(vec![int("[10, 20]"), int("[30, 40]")]),
            ),
            // overlaps with multiple intervals
            (
                vec![int("[20, 50]")],
                vec![int("[0, 30]"), int("[40, 60]")],
                Some(vec![int("[20, 30]"), int("[40, 50]")]),
            ),
            // multiple elements
            (
                vec![int("(-Inf, 10]"), int("[20, 30]"), int("[40, 50]")],
                vec![int("(-Inf, 10)"), int("[15, 25]"), int("(26, 35]")],
                Some(vec![int("(-Inf, 10)"), int("[20, 25]"), int("(26, 30]")]),
            ),
        ];

        for (a, b, expected_vec) in test_cases {
            let this = MultiInterval { intervals: a };
            let that = MultiInterval { intervals: b };
            let expected = expected_vec.map(|intervals| MultiInterval { intervals });

            assert_eq!(
                this.intersect(&that),
                expected,
                "MultiInterval.intersect failed: {this:?}.intersect({that:?}) should be {expected:?}",
            );
        }
    }

    #[test]
    fn test_MultiInterval_inverse() {
        let test_cases = vec![
            // zero elements
            ("", "(-Inf, Inf)"),
            ("(-Inf, Inf)", ""),
            // one element - -Inf on left
            ("(-Inf, 10)", "[10, Inf)"),
            ("(-Inf, 10]", "(10, Inf)"),
            // one element - no Infs on either side
            ("(10, 20)", "(-Inf, 10] [20, Inf)"),
            ("(10, 20]", "(-Inf, 10] (20, Inf)"),
            ("[10, 20)", "(-Inf, 10) [20, Inf)"),
            ("[10, 20]", "(-Inf, 10) (20, Inf)"),
            // one element - Inf on right
            ("(10, Inf)", "(-Inf, 10]"),
            ("[10, Inf)", "(-Inf, 10)"),
            // multiple elements - has Inf on either side
            ("(-Inf, 10) (20, Inf)", "[10, 20]"),
            (
                "(-Inf, 10) (20, 30) (40, Inf)",
                "[10, 20] [30, 40]",
            ),
            // multiple elements - Inf on one side
            (
                "(-Inf, 10) [20, 30)",
                "[10, 20) [30, Inf)",
            ),
            (
                "(-Inf, 10)        (20, 30)        (40, 50)",
                "          [10, 20]        [30, 40]        [50, Inf)",
            ),
            ("(0, 10) [20, Inf)", "(-Inf, 0] [10, 20)"),
            (
                "         [0, 10)        (20, 30)        (40, Inf)",
                "(-Inf, 0)       [10, 20]        [30, 40]",
            ),
            // TODO: same endpoint, like (0,0) [0,0] (0,0] [0,0)
            // complex examples
            (
                "           [-42, 3)      (3, 67)         (100, 101)          [205, 607]          (700, Inf)",
                "(-Inf, -42)        [3, 3]       [67, 100]          [101, 205)          (607, 700]",
            ),
        ];

        for (interval, expected) in test_cases {
            let interval = multiint(interval);
            let expected = multiint(expected);

            assert_eq!(
                interval.inverse(),
                expected,
                "MultiInterval.invert failed: {interval:?}.inverse() should be {expected:?}",
            );
        }
    }

    #[test]
    fn test_MultiInterval_axioms() {
        let input1 = multiint("[-42, 3) (3, 67) (100, 101) [205, 607] (700, Inf)");

        assert_eq!(
            input1.inverse().inverse(),
            input1,
            "The inverse of an inverse should be the original",
        );
        assert_eq!(
            input1.intersect(&multiint("(-Inf, Inf)")),
            Some(input1.clone()),
            "Intersecting something with (-Inf, Inf) should be the original"
        );
        assert!(
            !input1.intersects_with(&input1.inverse()),
            "An interval can't be intersected with its inverse"
        );
        assert_eq!(
            input1.intersect(&input1.inverse()),
            None,
            "An interval can't be intersected with its inverse"
        );
    }
}
