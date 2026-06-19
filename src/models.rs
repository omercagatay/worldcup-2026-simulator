use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::sim::{SimConfig, SimResults};

#[derive(Deserialize, Clone)]
pub struct SimRequest {
    pub n_sims: Option<usize>,
    pub seed: Option<u64>,
    pub elo_overrides: Option<HashMap<String, f64>>,
}

#[derive(Deserialize, Clone)]
pub struct ScenarioRequest {
    pub prompt: String,
    pub n_sims: Option<usize>,
    pub seed: Option<u64>,
}

#[derive(Serialize, Clone)]
pub struct TeamRow {
    pub team: String,
    pub win_pct: f64,
    pub final_pct: f64,
    pub sf_pct: f64,
    pub qf_pct: f64,
    pub r16_pct: f64,
    pub r32_pct: f64,
}

#[derive(Serialize, Clone)]
pub struct GroupRow {
    pub team: String,
    pub first_pct: f64,
    pub second_pct: f64,
    pub third_q_pct: f64,
    pub third_out_pct: f64,
    pub advance_pct: f64,
}

#[derive(Serialize, Clone)]
pub struct BracketSlot {
    pub match_id: u32,
    pub team: String,
}

#[derive(Serialize, Clone)]
pub struct FinalPair {
    pub a: String,
    pub b: String,
    pub pct: f64,
    pub count: usize,
}

#[derive(Serialize, Clone)]
pub struct SimResponse {
    pub n_sims: usize,
    pub seed: u64,
    pub teams: Vec<TeamRow>,
    pub groups: Vec<(String, Vec<GroupRow>)>,
    pub bracket: Vec<BracketSlot>,
    pub consensus_champion: String,
    pub top_finals: Vec<FinalPair>,
    pub top_champions: Vec<TeamRow>,
    pub elo_overrides: HashMap<String, f64>,
    pub scenario_applied: Option<String>,
}

pub fn build_response(
    world: &crate::sim::World,
    results: &SimResults,
    config: &SimConfig,
    scenario: Option<String>,
) -> SimResponse {
    let n = results.n_sims as f64;
    let teams = world.teams.clone();

    let mut team_rows: Vec<TeamRow> = teams
        .iter()
        .enumerate()
        .map(|(i, name)| TeamRow {
            team: name.clone(),
            win_pct: pct(results.champ_counts.get(&i).copied(), n),
            final_pct: pct(results.final_counts.get(&i).copied(), n),
            sf_pct: pct(results.sf_counts.get(&i).copied(), n),
            qf_pct: pct(results.qf_counts.get(&i).copied(), n),
            r16_pct: pct(results.r16_counts.get(&i).copied(), n),
            r32_pct: pct(results.r32_counts.get(&i).copied(), n),
        })
        .collect();
    team_rows.sort_by(|a, b| b.win_pct.partial_cmp(&a.win_pct).unwrap());

    let groups = world
        .groups
        .iter()
        .map(|(letter, members)| {
            let mut rows: Vec<GroupRow> = members
                .iter()
                .map(|&i| {
                    let stat = results
                        .group_stats
                        .get(letter)
                        .and_then(|m| m.get(&i))
                        .cloned()
                        .unwrap_or_default();
                    GroupRow {
                        team: teams[i].clone(),
                        first_pct: pct(Some(stat.first), n),
                        second_pct: pct(Some(stat.second), n),
                        third_q_pct: pct(Some(stat.third_q), n),
                        third_out_pct: pct(Some(stat.third_out), n),
                        advance_pct: pct(Some(stat.advance), n),
                    }
                })
                .collect();
            rows.sort_by(|a, b| b.advance_pct.partial_cmp(&a.advance_pct).unwrap());
            (letter.clone(), rows)
        })
        .collect();

    let mut bracket: Vec<BracketSlot> = results
        .slot_mode
        .iter()
        .map(|(&m, &i)| BracketSlot {
            match_id: m,
            team: teams[i].clone(),
        })
        .collect();
    bracket.sort_by_key(|b| b.match_id);

    let consensus_champion = bracket
        .iter()
        .find(|b| b.match_id == crate::data::FINAL)
        .map(|b| b.team.clone())
        .unwrap_or_default();

    let mut final_pairs: Vec<FinalPair> = results
        .final_pairs
        .iter()
        .map(|(&(a, b), &c)| FinalPair {
            a: teams[a].clone(),
            b: teams[b].clone(),
            pct: c as f64 / n * 100.0,
            count: c,
        })
        .collect();
    final_pairs.sort_by_key(|b| std::cmp::Reverse(b.count));
    let top_finals = final_pairs.into_iter().take(5).collect();

    let mut champ_sorted: Vec<(usize, usize)> =
        results.champ_counts.iter().map(|(&k, &v)| (k, v)).collect();
    champ_sorted.sort_by_key(|b| std::cmp::Reverse(b.1));
    let top_champions: Vec<TeamRow> = champ_sorted
        .iter()
        .take(10)
        .map(|&(i, c)| TeamRow {
            team: teams[i].clone(),
            win_pct: c as f64 / n * 100.0,
            final_pct: pct(results.final_counts.get(&i).copied(), n),
            sf_pct: pct(results.sf_counts.get(&i).copied(), n),
            qf_pct: pct(results.qf_counts.get(&i).copied(), n),
            r16_pct: pct(results.r16_counts.get(&i).copied(), n),
            r32_pct: pct(results.r32_counts.get(&i).copied(), n),
        })
        .collect();

    SimResponse {
        n_sims: results.n_sims,
        seed: config.seed,
        teams: team_rows,
        groups,
        bracket,
        consensus_champion,
        top_finals,
        top_champions,
        elo_overrides: config.elo_overrides.clone(),
        scenario_applied: scenario,
    }
}

fn pct(count: Option<usize>, n: f64) -> f64 {
    count.map_or(0.0, |c| c as f64 / n * 100.0)
}
