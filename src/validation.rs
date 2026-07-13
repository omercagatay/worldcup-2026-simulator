use std::collections::HashMap;

use crate::sim::World;

const MIN_SIMS: usize = 100;
const MAX_SIMS: usize = 200_000;
const MIN_ELO: f64 = 1000.0;
const MAX_ELO: f64 = 2600.0;
/// Scenario prompts are forwarded verbatim to the paid LLM API, so cap them
/// well below the 1 MB request-body limit to bound per-request token cost.
pub const MAX_PROMPT_CHARS: usize = 2000;

pub fn validate_prompt(prompt: &str) -> Result<(), String> {
    if prompt.trim().is_empty() {
        Err("Scenario prompt must not be empty".to_string())
    } else if prompt.chars().count() > MAX_PROMPT_CHARS {
        Err(format!(
            "Scenario prompt must be at most {MAX_PROMPT_CHARS} characters"
        ))
    } else {
        Ok(())
    }
}

pub fn validate_n_sims(n: usize) -> Result<usize, String> {
    if n < MIN_SIMS {
        Err(format!("n_sims must be at least {MIN_SIMS}"))
    } else if n > MAX_SIMS {
        Err(format!("n_sims must be at most {MAX_SIMS}"))
    } else {
        Ok(n)
    }
}

pub fn validate_elo_overrides(
    world: &World,
    overrides: &HashMap<String, f64>,
) -> Result<(), String> {
    for (team, rating) in overrides {
        if !world.idx.contains_key(team) {
            return Err(format!("Unknown team in Elo overrides: {team}"));
        }
        if *rating < MIN_ELO || *rating > MAX_ELO {
            return Err(format!(
                "Elo rating for {team} must be between {MIN_ELO:.0} and {MAX_ELO:.0}, got {rating:.1}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n_sims_within_bounds_is_ok() {
        assert_eq!(validate_n_sims(100).unwrap(), 100);
        assert_eq!(validate_n_sims(50_000).unwrap(), 50_000);
        assert_eq!(validate_n_sims(200_000).unwrap(), 200_000);
    }

    #[test]
    fn n_sims_out_of_bounds_is_rejected() {
        assert!(validate_n_sims(50).is_err());
        assert!(validate_n_sims(200_001).is_err());
        assert!(validate_n_sims(0).is_err());
    }

    #[test]
    fn prompt_within_limit_is_ok() {
        assert!(validate_prompt("Mbappé injured in training").is_ok());
        assert!(validate_prompt(&"x".repeat(MAX_PROMPT_CHARS)).is_ok());
    }

    #[test]
    fn prompt_over_limit_is_rejected() {
        assert!(validate_prompt(&"x".repeat(MAX_PROMPT_CHARS + 1)).is_err());
    }

    #[test]
    fn empty_or_blank_prompt_is_rejected() {
        assert!(validate_prompt("").is_err());
        assert!(validate_prompt("   \n\t").is_err());
    }

    #[test]
    fn elo_overrides_validated() {
        let world = World::new();
        let mut overrides = HashMap::new();
        overrides.insert("Argentina".to_string(), 2100.0);
        assert!(validate_elo_overrides(&world, &overrides).is_ok());

        overrides.insert("Atlantis".to_string(), 1800.0);
        assert!(validate_elo_overrides(&world, &overrides).is_err());
        overrides.remove("Atlantis");

        overrides.insert("Argentina".to_string(), 999.0);
        assert!(validate_elo_overrides(&world, &overrides).is_err());

        overrides.insert("Argentina".to_string(), 2601.0);
        assert!(validate_elo_overrides(&world, &overrides).is_err());
    }
}
