use std::collections::HashMap;
use std::io::Cursor;
use std::sync::OnceLock;

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::data;

pub const CUTOFF_YEAR: i32 = 2010;
pub const ROW_TEAM_NAME: &str = "Rest of World";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoricalMatch {
    pub date: NaiveDate,
    pub home_team: String,
    pub away_team: String,
    pub home_score: u16,
    pub away_score: u16,
    pub neutral: bool,
}

#[derive(Clone, Debug)]
pub struct TeamIndex {
    pub name_to_idx: HashMap<String, usize>,
    pub idx_to_name: Vec<String>,
    pub wc_names: Vec<String>,
    pub row_idx: usize,
}

impl TeamIndex {
    pub fn wc() -> Self {
        let wc_names: Vec<String> = data::elo().iter().map(|(t, _)| t.to_string()).collect();
        let mut idx_to_name = wc_names.clone();
        idx_to_name.push(ROW_TEAM_NAME.to_string());
        let row_idx = idx_to_name.len() - 1;
        let name_to_idx = idx_to_name
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i))
            .collect();
        TeamIndex {
            name_to_idx,
            idx_to_name,
            wc_names,
            row_idx,
        }
    }

    pub fn canonical(&self, team: &str) -> usize {
        self.name_to_idx.get(team).copied().unwrap_or(self.row_idx)
    }
}

#[derive(Clone, Debug)]
pub struct FitMatch {
    pub home_idx: usize,
    pub away_idx: usize,
    pub home_score: u16,
    pub away_score: u16,
    pub neutral: bool,
    pub weight: f64,
    pub days_ago: i64,
}

pub fn load_history() -> Vec<HistoricalMatch> {
    load_history_with_cutoff(CUTOFF_YEAR)
}

pub fn load_history_with_cutoff(min_year: i32) -> Vec<HistoricalMatch> {
    static CSV: OnceLock<Vec<u8>> = OnceLock::new();
    let bytes = CSV.get_or_init(|| include_bytes!("../data/international_results.csv").to_vec());
    parse_csv(bytes, min_year)
}

#[derive(serde::Deserialize)]
struct CsvRow {
    date: String,
    home_team: String,
    away_team: String,
    home_score: String,
    away_score: String,
    #[serde(rename = "tournament")]
    _tournament: String,
    #[serde(rename = "city")]
    _city: String,
    #[serde(rename = "country")]
    _country: String,
    neutral: String,
}

fn parse_csv(bytes: &[u8], min_year: i32) -> Vec<HistoricalMatch> {
    let mut rdr = csv::ReaderBuilder::new()
        .flexible(false)
        .has_headers(true)
        .from_reader(Cursor::new(bytes));
    let mut out = Vec::new();
    for rec in rdr.deserialize::<CsvRow>() {
        let Ok(r) = rec else { continue };
        let Some(date) = parse_date(&r.date) else {
            continue;
        };
        if date.year() < min_year {
            continue;
        }
        let Ok(hs) = r.home_score.trim().parse::<u16>() else {
            continue;
        };
        let Ok(as_) = r.away_score.trim().parse::<u16>() else {
            continue;
        };
        let neutral = matches!(r.neutral.trim().to_ascii_uppercase().as_str(), "TRUE" | "1");
        out.push(HistoricalMatch {
            date,
            home_team: r.home_team,
            away_team: r.away_team,
            home_score: hs,
            away_score: as_,
            neutral,
        });
    }
    tracing::info!(
        "Loaded {} historical matches (cutoff {}+)",
        out.len(),
        min_year
    );
    out
}

fn parse_date(raw: &str) -> Option<NaiveDate> {
    let s = raw.trim();
    ["%Y-%m-%d", "%m/%d/%Y", "%-m/%-d/%Y"]
        .iter()
        .find_map(|fmt| NaiveDate::parse_from_str(s, fmt).ok())
}

pub fn prepare_fit_matches(
    history: &[HistoricalMatch],
    idx: &TeamIndex,
    half_life_days: f64,
    as_of: NaiveDate,
) -> Vec<FitMatch> {
    let xi = if half_life_days > 0.0 {
        std::f64::consts::LN_2 / half_life_days
    } else {
        0.0
    };
    let mut out = Vec::with_capacity(history.len());
    for m in history {
        if m.date > as_of {
            continue;
        }
        let h = idx.canonical(&m.home_team);
        let a = idx.canonical(&m.away_team);
        let days_ago = (as_of - m.date).num_days().max(0);
        let weight = (-xi * days_ago as f64).exp();
        out.push(FitMatch {
            home_idx: h,
            away_idx: a,
            home_score: m.home_score,
            away_score: m.away_score,
            neutral: m.neutral,
            weight,
            days_ago,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_maps_known_wc_team_and_row_for_unknown() {
        let idx = TeamIndex::wc();
        assert_eq!(idx.canonical("Argentina"), idx.name_to_idx["Argentina"]);
        assert_eq!(idx.canonical("Bolivia"), idx.row_idx);
        assert_eq!(idx.canonical("Nonexistent Country"), idx.row_idx);
        assert_eq!(idx.idx_to_name[idx.row_idx], ROW_TEAM_NAME);
    }

    #[test]
    fn load_history_drops_pre_cutoff_and_na_scores() {
        let ms = load_history_with_cutoff(2024);
        assert!(!ms.is_empty(), "expected some 2024+ matches");
        for m in &ms {
            assert!(m.date.year() >= 2024);
            assert!(!m.home_team.is_empty());
        }
    }

    #[test]
    fn prepare_fit_assigns_weights_and_row_bucket() {
        let idx = TeamIndex::wc();
        let as_of = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let history = vec![
            HistoricalMatch {
                date: NaiveDate::from_ymd_opt(2024, 6, 14).unwrap(),
                home_team: "Argentina".to_string(),
                away_team: "Bolivia".to_string(),
                home_score: 3,
                away_score: 0,
                neutral: false,
            },
            HistoricalMatch {
                date: as_of,
                home_team: "Argentina".to_string(),
                away_team: "France".to_string(),
                home_score: 1,
                away_score: 1,
                neutral: true,
            },
            HistoricalMatch {
                date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
                home_team: "Argentina".to_string(),
                away_team: "France".to_string(),
                home_score: 2,
                away_score: 0,
                neutral: true,
            },
        ];
        let fits = prepare_fit_matches(&history, &idx, 1460.0, as_of);
        assert_eq!(fits.len(), 2);
        assert!(fits[0].weight < 1.0 && fits[0].weight > 0.0);
        assert!((fits[1].weight - 1.0).abs() < 1e-9);
        assert_eq!(fits[0].away_idx, idx.row_idx);
        assert_eq!(fits[1].away_idx, idx.name_to_idx["France"]);
    }
}
