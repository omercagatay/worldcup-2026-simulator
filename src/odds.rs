//! Conversion of Monte Carlo probabilities to betting-style odds.

/// Fair (margin-free) decimal odds for a probability in `0.0..=1.0`.
/// Returns `None` when the outcome never occurred in the simulation,
/// where decimal odds are undefined.
pub fn decimal_odds(p: f64) -> Option<f64> {
    if p > 0.0 && p <= 1.0 && p.is_finite() {
        Some(1.0 / p)
    } else {
        None
    }
}

/// Decimal odds from a percentage in `0.0..=100.0` (the unit used in
/// API responses).
pub fn decimal_odds_from_pct(pct: f64) -> Option<f64> {
    decimal_odds(pct / 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certain_outcome_has_odds_of_one() {
        assert_eq!(decimal_odds(1.0), Some(1.0));
    }

    #[test]
    fn even_chance_pays_double() {
        assert!((decimal_odds(0.5).unwrap() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn impossible_and_invalid_probabilities_have_no_odds() {
        assert_eq!(decimal_odds(0.0), None);
        assert_eq!(decimal_odds(-0.1), None);
        assert_eq!(decimal_odds(1.5), None);
        assert_eq!(decimal_odds(f64::NAN), None);
        assert_eq!(decimal_odds(f64::INFINITY), None);
    }

    #[test]
    fn pct_variant_matches_probability_variant() {
        assert_eq!(decimal_odds_from_pct(25.0), decimal_odds(0.25));
        assert_eq!(decimal_odds_from_pct(0.0), None);
    }
}
