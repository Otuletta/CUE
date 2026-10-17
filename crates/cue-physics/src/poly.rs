//! Real-root isolation for the low-degree polynomials that describe when two
//! balls touch or a ball reaches a cushion.
//!
//! Collision times are roots of polynomials of degree ≤ 4. Closed-form quartic
//! formulas are numerically fragile, so roots are isolated instead: the roots
//! of `p′` split the search window into intervals on which `p` is monotonic,
//! and each interval with a sign change is bisected down to machine precision.
//! The recursion bottoms out at constants, so degree is never special-cased
//! and leading coefficients that vanish are handled for free.

/// Maximum supported coefficient count (degree 4).
const MAX_COEFFS: usize = 5;

/// A polynomial with coefficients in ascending order: `c₀ + c₁t + c₂t² + …`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Poly {
    c: [f64; MAX_COEFFS],
    len: usize,
}

impl Poly {
    pub(crate) fn new(coeffs: &[f64]) -> Self {
        assert!(
            coeffs.len() <= MAX_COEFFS,
            "degree above 4 is not supported"
        );
        let mut c = [0.0; MAX_COEFFS];
        c[..coeffs.len()].copy_from_slice(coeffs);
        Self {
            c,
            len: coeffs.len(),
        }
    }

    /// Horner evaluation.
    pub(crate) fn eval(&self, t: f64) -> f64 {
        self.c[..self.len]
            .iter()
            .rev()
            .fold(0.0, |acc, &k| acc * t + k)
    }

    fn derivative(&self) -> Self {
        let mut d = [0.0; MAX_COEFFS];
        for i in 1..self.len {
            d[i - 1] = self.c[i] * i as f64;
        }
        Self {
            c: d,
            len: self.len.saturating_sub(1),
        }
    }

    /// Every point in `(lo, hi)` where the polynomial changes sign, ascending.
    fn sign_changes(&self, lo: f64, hi: f64, out: &mut Roots) {
        if self.len <= 1 {
            return;
        }
        let mut critical = Roots::default();
        self.derivative().sign_changes(lo, hi, &mut critical);

        let mut a = lo;
        for &b in critical.as_slice().iter().chain(std::iter::once(&hi)) {
            let (fa, fb) = (self.eval(a), self.eval(b));
            if (fa > 0.0) != (fb > 0.0) && a < b {
                out.push(self.bisect(a, b));
            }
            a = b;
        }
    }

    /// Earliest `t ∈ [lo, hi]` at which a contact begins: either the
    /// polynomial falls from positive to non-positive, or it is already
    /// non-positive and starts decreasing.
    ///
    /// The second case matters in practice. Rounding can leave two touching
    /// balls a fraction of a nanometre inside each other while they separate;
    /// if friction then turns them back towards each other, there is no
    /// positive-to-negative crossing to find, and a crossing-only search
    /// would let them interpenetrate.
    ///
    /// For a genuine crossing the returned time is on the positive side, i.e.
    /// just before contact.
    pub(crate) fn first_falling_root(&self, lo: f64, hi: f64) -> Option<f64> {
        self.first_contact(lo, hi, |_| true)
    }

    /// Like [`first_falling_root`](Self::first_falling_root), but each
    /// candidate is offered to `accept` in time order and only accepted
    /// candidates count. Used for finite cushion segments, where touching the
    /// infinite line outside the segment is not a contact.
    pub(crate) fn first_contact(
        &self,
        lo: f64,
        hi: f64,
        accept: impl Fn(f64) -> bool,
    ) -> Option<f64> {
        if lo.is_nan() || hi.is_nan() || lo > hi {
            return None;
        }
        let mut critical = Roots::default();
        self.derivative().sign_changes(lo, hi, &mut critical);

        // Between consecutive critical points the polynomial is monotonic.
        let mut a = lo;
        for &b in critical.as_slice().iter().chain(std::iter::once(&hi)) {
            if a < b {
                let (fa, fb) = (self.eval(a), self.eval(b));
                if fa > 0.0 && fb <= 0.0 {
                    let t = self.bisect(a, b);
                    if accept(t) {
                        return Some(t);
                    }
                } else if fa <= 0.0 && fb < fa && accept(a) {
                    return Some(a);
                }
            }
            a = b;
        }
        None
    }

    /// Shrinks `[a, b]` around the sign change and returns the end that keeps
    /// the sign of `p(a)`.
    fn bisect(&self, mut a: f64, mut b: f64) -> f64 {
        let positive_at_a = self.eval(a) > 0.0;
        loop {
            let m = 0.5 * (a + b);
            if m <= a || m >= b {
                return a;
            }
            if (self.eval(m) > 0.0) == positive_at_a {
                a = m;
            } else {
                b = m;
            }
        }
    }
}

/// Up to four roots without allocating.
#[derive(Clone, Copy, Debug, Default)]
struct Roots {
    r: [f64; MAX_COEFFS - 1],
    len: usize,
}

impl Roots {
    fn push(&mut self, t: f64) {
        if self.len < self.r.len() {
            self.r[self.len] = t;
            self.len += 1;
        }
    }

    fn as_slice(&self) -> &[f64] {
        &self.r[..self.len]
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    /// Expands `(t − r₀)(t − r₁)…` into ascending coefficients.
    fn from_roots(roots: &[f64]) -> Poly {
        let mut c = vec![1.0];
        for &r in roots {
            let mut next = vec![0.0; c.len() + 1];
            for (i, &k) in c.iter().enumerate() {
                next[i] -= r * k;
                next[i + 1] += k;
            }
            c = next;
        }
        Poly::new(&c)
    }

    #[test]
    fn horner_matches_direct_evaluation() {
        let p = Poly::new(&[1.0, -2.0, 3.0, -4.0, 5.0]);
        let t: f64 = 0.7;
        let direct = 1.0 - 2.0 * t + 3.0 * t.powi(2) - 4.0 * t.powi(3) + 5.0 * t.powi(4);
        assert_relative_eq!(p.eval(t), direct, epsilon = 1e-12);
    }

    #[test]
    fn finds_first_falling_root_of_a_quartic() {
        // Positive leading coefficient, roots at 0.5, 1, 2, 3: the polynomial
        // is positive before 0.5 and falls through zero there.
        let p = from_roots(&[0.5, 1.0, 2.0, 3.0]);
        let t = p.first_falling_root(0.0, 10.0).unwrap();
        assert_relative_eq!(t, 0.5, epsilon = 1e-12);
        assert!(p.eval(t) >= 0.0, "must report the pre-contact side");
    }

    #[test]
    fn skips_rising_roots() {
        // Falls at 2, rises at 1 — searching from 1.5 must find 2, not 1.
        // p = −(t−1)(t−2) is positive between 1 and 2.
        let p = Poly::new(&[-2.0, 3.0, -1.0]);
        assert_relative_eq!(
            p.first_falling_root(1.5, 5.0).unwrap(),
            2.0,
            epsilon = 1e-12
        );
        // From before 1 the polynomial is negative and rising: nothing falls
        // until 2.
        assert_relative_eq!(
            p.first_falling_root(0.0, 5.0).unwrap(),
            2.0,
            epsilon = 1e-12
        );
    }

    #[test]
    fn respects_the_window() {
        let p = from_roots(&[4.0, 6.0]);
        assert_eq!(p.first_falling_root(0.0, 3.9), None);
        assert!(p.first_falling_root(0.0, 4.1).is_some());
    }

    #[test]
    fn reports_lo_when_already_overlapping_and_closing() {
        // f(0) = −1, f′(0) = −1.
        let p = Poly::new(&[-1.0, -1.0]);
        assert_eq!(p.first_falling_root(0.0, 1.0), Some(0.0));
        // Overlapping but separating: no event.
        let q = Poly::new(&[-1.0, 1.0]);
        assert_eq!(q.first_falling_root(0.0, 1.0), None);
    }

    #[test]
    fn reports_the_turnaround_of_a_contact_that_starts_closing() {
        // Slightly negative, separating, then turning back while still
        // non-positive: f = −1e−19 + 1e−10·t − 0.1·t². Turnaround at 5e−10.
        let p = Poly::new(&[-1e-19, 1e-10, -0.1]);
        let t = p.first_falling_root(0.0, 1.0).unwrap();
        assert_relative_eq!(t, 5e-10, epsilon = 1e-15);
    }

    #[test]
    fn rejected_candidates_fall_through_to_later_ones() {
        // Two downward crossings, at 1 and 3; reject the first.
        let p = from_roots(&[1.0, 2.0, 3.0, 4.0]);
        let t = p.first_contact(0.0, 10.0, |t| t > 2.0).unwrap();
        assert_relative_eq!(t, 3.0, epsilon = 1e-12);
        assert_eq!(p.first_contact(0.0, 10.0, |_| false), None);
    }

    #[test]
    fn handles_vanishing_leading_coefficients() {
        // A "quartic" that is really linear: 1 − t.
        let p = Poly::new(&[1.0, -1.0, 0.0, 0.0, 0.0]);
        assert_relative_eq!(
            p.first_falling_root(0.0, 2.0).unwrap(),
            1.0,
            epsilon = 1e-12
        );
        // A constant never crosses.
        assert_eq!(Poly::new(&[3.0]).first_falling_root(0.0, 1.0), None);
    }

    #[test]
    fn close_roots_are_separated() {
        // Two balls that barely clip: roots 1 µs apart. Near a double root
        // the polynomial is flat, so accuracy degrades to ~ε/separation —
        // still far below anything physically meaningful.
        let p = from_roots(&[1.0, 1.000_001]);
        let t = p.first_falling_root(0.0, 2.0).unwrap();
        assert_relative_eq!(t, 1.0, epsilon = 1e-9);
        assert!(t < 1.000_001);
    }

    #[test]
    fn empty_or_inverted_windows_find_nothing() {
        let p = from_roots(&[1.0]);
        assert_eq!(p.first_falling_root(2.0, 1.0), None);
        assert_eq!(p.first_falling_root(f64::NAN, 1.0), None);
    }
}
