use std::collections::HashMap;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rand_distr::Poisson;
use rayon::prelude::*;

use crate::data;

#[derive(Clone)]
pub struct World {
    pub teams: Vec<String>,
    pub idx: HashMap<String, usize>,
    pub elo: Vec<f64>,
    pub host: Vec<bool>,
    pub groups: Vec<(String, Vec<usize>)>,
    pub played: HashMap<(String, usize, usize), (u16, u16)>,
}

#[derive(Clone, Default, Debug)]
pub struct GroupStat {
    pub first: usize,
    pub second: usize,
    pub third_q: usize,
    pub third_out: usize,
    pub advance: usize,
}

#[derive(Clone, Debug)]
pub struct SimResults {
    pub n_sims: usize,
    pub champ_counts: HashMap<usize, usize>,
    pub final_counts: HashMap<usize, usize>,
    pub sf_counts: HashMap<usize, usize>,
    pub qf_counts: HashMap<usize, usize>,
    pub r16_counts: HashMap<usize, usize>,
    pub r32_counts: HashMap<usize, usize>,
    pub group_stats: HashMap<String, HashMap<usize, GroupStat>>,
    pub slot_mode: HashMap<u32, usize>,
    pub final_pairs: HashMap<(usize, usize), usize>,
    #[allow(dead_code)]
    pub third_place_counts: HashMap<usize, usize>,
}

#[derive(Clone, Debug)]
pub struct SimConfig {
    pub n_sims: usize,
    pub seed: u64,
    pub elo_overrides: HashMap<String, f64>,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            n_sims: 50000,
            seed: 12345,
            elo_overrides: HashMap::new(),
        }
    }
}

impl World {
    pub fn new() -> Self {
        let teams: Vec<String> = data::elo().iter().map(|(t, _)| t.to_string()).collect();
        let idx: HashMap<String, usize> = teams
            .iter()
            .enumerate()
            .map(|(i, t)| (t.clone(), i))
            .collect();
        let elo: Vec<f64> = data::elo().iter().map(|(_, e)| *e).collect();
        let hosts = data::hosts();
        let host: Vec<bool> = data::elo().iter().map(|(t, _)| hosts.contains(t)).collect();

        let groups: Vec<(String, Vec<usize>)> = data::groups()
            .iter()
            .map(|(letter, members)| {
                (
                    letter.to_string(),
                    members.iter().map(|t| idx[*t]).collect(),
                )
            })
            .collect();

        let mut played: HashMap<(String, usize, usize), (u16, u16)> = HashMap::new();
        for pm in data::played() {
            let members: Vec<&str> = data::groups()
                .iter()
                .find(|(l, _)| *l == pm.group)
                .unwrap()
                .1
                .clone();
            let pa = members.iter().position(|&t| t == pm.a).unwrap();
            let pb = members.iter().position(|&t| t == pm.b).unwrap();
            if pa < pb {
                played.insert((pm.group.to_string(), pa, pb), (pm.sa, pm.sb));
            } else {
                played.insert((pm.group.to_string(), pb, pa), (pm.sb, pm.sa));
            }
        }

        World {
            teams,
            idx,
            elo,
            host,
            groups,
            played,
        }
    }

    pub fn update_from_live(&mut self, live: &crate::scraper::LiveData) -> (usize, usize) {
        let mut elo_updated = 0;
        for (team, rating) in &live.elo_ratings {
            if let Some(&i) = self.idx.get(team) {
                self.elo[i] = *rating;
                elo_updated += 1;
            }
        }

        self.played.clear();
        for (letter, members) in &self.groups {
            let member_names: Vec<&str> = members.iter().map(|&i| self.teams[i].as_str()).collect();
            for pm in &live.played_matches {
                if pm.group != *letter {
                    continue;
                }
                let pa = match member_names.iter().position(|&t| t == pm.team_a) {
                    Some(p) => p,
                    None => continue,
                };
                let pb = match member_names.iter().position(|&t| t == pm.team_b) {
                    Some(p) => p,
                    None => continue,
                };
                if pa < pb {
                    self.played
                        .insert((letter.clone(), pa, pb), (pm.score_a, pm.score_b));
                } else {
                    self.played
                        .insert((letter.clone(), pb, pa), (pm.score_b, pm.score_a));
                }
            }
        }

        let matches_updated = self.played.len();
        tracing::info!(
            "World updated from live data: {} Elo ratings, {} played matches applied",
            elo_updated,
            matches_updated
        );
        (elo_updated, matches_updated)
    }

    pub fn apply_overrides(&mut self, overrides: &HashMap<String, f64>) {
        for (team, rating) in overrides {
            if let Some(&i) = self.idx.get(team) {
                self.elo[i] = *rating;
            }
        }
    }

    fn lam_pair(&self, ia: usize, ib: usize) -> (f64, f64) {
        let dr = self.elo[ia] - self.elo[ib]
            + data::HOME_ADV * (self.host[ia] as i8 - self.host[ib] as i8) as f64;
        let la = data::BASE * (10.0_f64).powf(dr / data::D_DIV);
        let lb = data::BASE * (10.0_f64).powf(-dr / data::D_DIV);
        (la.clamp(0.15, 5.0), lb.clamp(0.15, 5.0))
    }

    fn sample_poisson(rng: &mut SmallRng, lambda: f64) -> i64 {
        let dist = Poisson::new(lambda).unwrap();
        rng.sample(dist) as i64
    }

    fn rank_group(
        &self,
        letter: &str,
        members: &[usize],
        rng: &mut SmallRng,
    ) -> (Vec<usize>, usize, (i64, i64, i64, i64)) {
        let pairs = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
        let mut stats: [[i64; 3]; 4] = [[0; 3]; 4]; // [pts, gf, ga]
        let mut results: HashMap<(usize, usize), (i64, i64)> = HashMap::new();

        for &(pa, pb) in &pairs {
            let (la, lb) = self.lam_pair(members[pa], members[pb]);
            let (ga, gb) = if let Some(&(sa, sb)) = self.played.get(&(letter.to_string(), pa, pb)) {
                (sa as i64, sb as i64)
            } else {
                (Self::sample_poisson(rng, la), Self::sample_poisson(rng, lb))
            };
            stats[pa][1] += ga;
            stats[pa][2] += gb;
            stats[pb][1] += gb;
            stats[pb][2] += ga;
            if ga > gb {
                stats[pa][0] += 3;
            } else if ga == gb {
                stats[pa][0] += 1;
                stats[pb][0] += 1;
            } else {
                stats[pb][0] += 3;
            }
            results.insert((pa, pb), (ga, gb));
        }

        let pkey = |p: usize| -> (i64, i64, i64) {
            let s = &stats[p];
            (s[0], s[1] - s[2], s[1])
        };

        let mut order: Vec<usize> = (0..4).collect();
        order.sort_by_key(|&b| std::cmp::Reverse(pkey(b)));

        let mut blocks: Vec<Vec<usize>> = Vec::new();
        let mut i = 0;
        while i < 4 {
            let mut j = i;
            while j + 1 < 4 && pkey(order[j + 1]) == pkey(order[i]) {
                j += 1;
            }
            blocks.push(order[i..=j].to_vec());
            i = j + 1;
        }

        let mut final_order: Vec<usize> = Vec::new();
        for block in blocks {
            if block.len() == 1 {
                final_order.push(block[0]);
                continue;
            }
            let bset: std::collections::HashSet<usize> = block.iter().copied().collect();
            let mut h: HashMap<usize, [i64; 3]> = HashMap::new();
            for &p in &block {
                h.insert(p, [0; 3]);
            }
            for ((pa, pb), (ga, gb)) in &results {
                if bset.contains(pa) && bset.contains(pb) {
                    h.get_mut(pa).unwrap()[1] += ga;
                    h.get_mut(pa).unwrap()[2] += gb;
                    h.get_mut(pb).unwrap()[1] += gb;
                    h.get_mut(pb).unwrap()[2] += ga;
                    if ga > gb {
                        h.get_mut(pa).unwrap()[0] += 3;
                    } else if ga == gb {
                        h.get_mut(pa).unwrap()[0] += 1;
                        h.get_mut(pb).unwrap()[0] += 1;
                    } else {
                        h.get_mut(pb).unwrap()[0] += 3;
                    }
                }
            }
            let tiebreak: HashMap<usize, u64> =
                block.iter().map(|&p| (p, rng.gen::<u64>())).collect();
            let hkey = |p: usize| -> (i64, i64, i64, i64, u64) {
                let hs = &h[&p];
                let ga_overall = stats[p][2];
                (hs[0], hs[1] - hs[2], hs[1], -ga_overall, tiebreak[&p])
            };
            let mut block_sorted = block.clone();
            block_sorted.sort_by_key(|&b| std::cmp::Reverse(hkey(b)));
            final_order.extend(block_sorted);
        }

        let third = final_order[2];
        let s = &stats[third];
        let third_rec = (s[0], s[1] - s[2], s[1], s[2]);
        let ordered: Vec<usize> = final_order.iter().map(|&p| members[p]).collect();
        (ordered, members[third], third_rec)
    }

    fn assign_thirds(
        qual_groups: &[String],
        slots_elig: &HashMap<u32, Vec<&'static str>>,
    ) -> HashMap<u32, String> {
        let mut slots: Vec<u32> = slots_elig.keys().copied().collect();
        slots.sort();
        let mut assignment: HashMap<u32, String> = HashMap::new();
        let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();

        fn backtrack(
            remaining: &[u32],
            slots_elig: &HashMap<u32, Vec<&'static str>>,
            qual_groups: &[String],
            assignment: &mut HashMap<u32, String>,
            used: &mut std::collections::HashSet<String>,
        ) -> bool {
            if remaining.is_empty() {
                return true;
            }
            let mut ordered = remaining.to_vec();
            // Sort by constrained-ness, then slot id for deterministic ordering.
            ordered.sort_by_key(|s| {
                (
                    qual_groups
                        .iter()
                        .filter(|g| {
                            slots_elig
                                .get(s)
                                .is_some_and(|elig| elig.contains(&g.as_str()))
                        })
                        .filter(|g| !used.contains(*g))
                        .count(),
                    *s,
                )
            });
            let s = ordered[0];
            for g in qual_groups {
                if slots_elig
                    .get(&s)
                    .is_some_and(|elig| elig.contains(&g.as_str()))
                    && !used.contains(g)
                {
                    assignment.insert(s, g.clone());
                    used.insert(g.clone());
                    let rest: Vec<u32> = remaining.iter().filter(|&&r| r != s).copied().collect();
                    if backtrack(&rest, slots_elig, qual_groups, assignment, used) {
                        return true;
                    }
                    used.remove(g);
                    assignment.remove(&s);
                }
            }
            false
        }

        backtrack(&slots, slots_elig, qual_groups, &mut assignment, &mut used);
        assignment
    }

    fn ko_match(
        &self,
        ia: usize,
        ib: usize,
        rng: &mut SmallRng,
        knockout: bool,
    ) -> (i64, i64, bool, bool) {
        let dr = self.elo[ia] - self.elo[ib]
            + data::HOME_ADV * (self.host[ia] as i8 - self.host[ib] as i8) as f64;
        let la = (data::BASE * (10.0_f64).powf(dr / data::D_DIV)).clamp(0.15, 5.0);
        let lb = (data::BASE * (10.0_f64).powf(-dr / data::D_DIV)).clamp(0.15, 5.0);
        let ga = Self::sample_poisson(rng, la);
        let gb = Self::sample_poisson(rng, lb);
        if !knockout {
            return (ga, gb, false, false);
        }
        if ga == gb {
            let et_a = Self::sample_poisson(rng, la * data::ET_FACTOR);
            let et_b = Self::sample_poisson(rng, lb * data::ET_FACTOR);
            let tot_a = ga + et_a;
            let tot_b = gb + et_b;
            if tot_a == tot_b {
                let we = 1.0 / (1.0 + (10.0_f64).powf(-dr / 400.0));
                let pen_prob_a = (0.5 + data::PEN_DAMP * (we - 0.5)).clamp(0.2, 0.8);
                let u: f64 = rng.gen();
                let win_a = u < pen_prob_a;
                (tot_a, tot_b, win_a, !win_a)
            } else {
                (tot_a, tot_b, tot_a > tot_b, tot_a < tot_b)
            }
        } else {
            (ga, gb, ga > gb, ga < gb)
        }
    }

    pub fn simulate_one(&self, rng: &mut SmallRng) -> SingleSimResult {
        let _letters: Vec<String> = self.groups.iter().map(|(l, _)| l.clone()).collect();
        let mut slot_team: HashMap<String, usize> = HashMap::new();
        let mut thirds: Vec<(String, (i64, i64, i64, i64))> = Vec::new();

        for (letter, members) in &self.groups {
            let (ordered, _third_idx, third_rec) = self.rank_group(letter, members, rng);
            slot_team.insert(format!("1{}", letter), ordered[0]);
            slot_team.insert(format!("2{}", letter), ordered[1]);
            slot_team.insert(format!("3{}", letter), ordered[2]);
            thirds.push((letter.clone(), third_rec));
        }

        let lots_rand: f64 = rng.gen();
        let mut thirds_scored: Vec<(i64, i64, i64, i64, f64, String)> = thirds
            .iter()
            .enumerate()
            .map(|(i, (letter, rec))| {
                (
                    rec.0,
                    rec.1,
                    rec.2,
                    -rec.3,
                    lots_rand * 0.001 + i as f64 * 1e-6,
                    letter.clone(),
                )
            })
            .collect();
        thirds_scored.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then(b.1.cmp(&a.1))
                .then(b.2.cmp(&a.2))
                .then(b.3.cmp(&a.3))
                .then(b.4.total_cmp(&a.4))
        });
        let qual: Vec<String> = thirds_scored
            .iter()
            .take(8)
            .map(|(_, _, _, _, _, l)| l.clone())
            .collect();

        let slots_elig = data::third_place_slots();
        let assign = Self::assign_thirds(&qual, &slots_elig);

        let mut winners: HashMap<u32, usize> = HashMap::new();
        let mut losers: HashMap<u32, usize> = HashMap::new();
        let mut r32_teams: Vec<usize> = Vec::new();

        for (m, sa, sb) in data::r32() {
            let ta = if sa == "3" {
                let g = assign.get(&m).cloned().unwrap_or_default();
                slot_team[&format!("3{}", g)]
            } else {
                slot_team[sa]
            };
            let tb = if sb == "3" {
                let g = assign.get(&m).cloned().unwrap_or_default();
                slot_team[&format!("3{}", g)]
            } else {
                slot_team[sb]
            };
            r32_teams.push(ta);
            r32_teams.push(tb);
            let (_, _, wa, _wb) = self.ko_match(ta, tb, rng, true);
            winners.insert(m, if wa { ta } else { tb });
        }

        for (m, a, b) in data::r16() {
            let ta = winners[&a];
            let tb = winners[&b];
            let (_, _, wa, _wb) = self.ko_match(ta, tb, rng, true);
            winners.insert(m, if wa { ta } else { tb });
            losers.insert(m, if wa { tb } else { ta });
        }
        for (m, a, b) in data::qf() {
            let ta = winners[&a];
            let tb = winners[&b];
            let (_, _, wa, _wb) = self.ko_match(ta, tb, rng, true);
            winners.insert(m, if wa { ta } else { tb });
            losers.insert(m, if wa { tb } else { ta });
        }
        for (m, a, b) in data::sf() {
            let ta = winners[&a];
            let tb = winners[&b];
            let (_, _, wa, _wb) = self.ko_match(ta, tb, rng, true);
            winners.insert(m, if wa { ta } else { tb });
            losers.insert(m, if wa { tb } else { ta });
        }

        let sf_a = winners[&101];
        let sf_b = winners[&102];
        let (_, _, wa, _) = self.ko_match(sf_a, sf_b, rng, true);
        let champion = if wa { sf_a } else { sf_b };
        winners.insert(data::FINAL, champion);
        let finalists = (sf_a.min(sf_b), sf_a.max(sf_b));

        let (_, _, twa, _) = self.ko_match(losers[&101], losers[&102], rng, true);
        let third_place = if twa { losers[&101] } else { losers[&102] };

        SingleSimResult {
            champion,
            finalists,
            sf_teams: vec![winners[&97], winners[&98], winners[&99], winners[&100]],
            qf_teams: data::r16().iter().map(|(m, _, _)| winners[m]).collect(),
            r16_teams: data::r32().iter().map(|(m, _, _)| winners[m]).collect(),
            r32_teams,
            slot_winners: winners.clone(),
            third_place,
            group_order: slot_team,
            qual_thirds: qual,
        }
    }

    pub fn simulate(&self, config: &SimConfig) -> SimResults {
        let n = config.n_sims;
        let results: Vec<SingleSimResult> = (0..n)
            .into_par_iter()
            .map(|i| {
                let mut rng =
                    SmallRng::seed_from_u64(config.seed.wrapping_add(i as u64 * 2654435761));
                self.simulate_one(&mut rng)
            })
            .collect();

        let mut champ_counts: HashMap<usize, usize> = HashMap::new();
        let mut final_counts: HashMap<usize, usize> = HashMap::new();
        let mut sf_counts: HashMap<usize, usize> = HashMap::new();
        let mut qf_counts: HashMap<usize, usize> = HashMap::new();
        let mut r16_counts: HashMap<usize, usize> = HashMap::new();
        let mut r32_counts: HashMap<usize, usize> = HashMap::new();
        let mut final_pairs: HashMap<(usize, usize), usize> = HashMap::new();
        let mut third_place_counts: HashMap<usize, usize> = HashMap::new();
        let mut group_stats: HashMap<String, HashMap<usize, GroupStat>> = HashMap::new();
        let mut slot_winner_counts: HashMap<u32, HashMap<usize, usize>> = HashMap::new();

        for letter in self.groups.iter().map(|(l, _)| l) {
            group_stats.insert(letter.clone(), HashMap::new());
        }
        for m in data::r32()
            .iter()
            .map(|(m, _, _)| *m)
            .chain(data::r16().iter().map(|(m, _, _)| *m))
            .chain(data::qf().iter().map(|(m, _, _)| *m))
            .chain(data::sf().iter().map(|(m, _, _)| *m))
            .chain(std::iter::once(data::FINAL))
        {
            slot_winner_counts.insert(m, HashMap::new());
        }

        for r in &results {
            *champ_counts.entry(r.champion).or_insert(0) += 1;
            *final_counts.entry(r.finalists.0).or_insert(0) += 1;
            *final_counts.entry(r.finalists.1).or_insert(0) += 1;
            for &t in &r.sf_teams {
                *sf_counts.entry(t).or_insert(0) += 1;
            }
            for &t in &r.qf_teams {
                *qf_counts.entry(t).or_insert(0) += 1;
            }
            for &t in &r.r16_teams {
                *r16_counts.entry(t).or_insert(0) += 1;
            }
            for &t in &r.r32_teams {
                *r32_counts.entry(t).or_insert(0) += 1;
            }
            *final_pairs.entry(r.finalists).or_insert(0) += 1;
            *third_place_counts.entry(r.third_place).or_insert(0) += 1;

            let qual_set: std::collections::HashSet<&String> = r.qual_thirds.iter().collect();
            for (letter, _members) in &self.groups {
                let gs = group_stats.get_mut(letter).unwrap();
                let t1 = r.group_order[&format!("1{}", letter)];
                let t2 = r.group_order[&format!("2{}", letter)];
                let t3 = r.group_order[&format!("3{}", letter)];
                let s1 = gs.entry(t1).or_default();
                s1.first += 1;
                s1.advance += 1;
                let s2 = gs.entry(t2).or_default();
                s2.second += 1;
                s2.advance += 1;
                let s3 = gs.entry(t3).or_default();
                if qual_set.contains(letter) {
                    s3.third_q += 1;
                    s3.advance += 1;
                } else {
                    s3.third_out += 1;
                }
            }

            for (&m, &team) in &r.slot_winners {
                if let Some(counts) = slot_winner_counts.get_mut(&m) {
                    *counts.entry(team).or_insert(0) += 1;
                }
            }
        }

        let slot_mode: HashMap<u32, usize> = slot_winner_counts
            .iter()
            .map(|(&m, counts)| {
                let (team, _) = counts.iter().max_by_key(|(_, &c)| c).unwrap();
                (m, *team)
            })
            .collect();

        SimResults {
            n_sims: n,
            champ_counts,
            final_counts,
            sf_counts,
            qf_counts,
            r16_counts,
            r32_counts,
            group_stats,
            slot_mode,
            final_pairs,
            third_place_counts,
        }
    }
}

pub struct SingleSimResult {
    pub champion: usize,
    pub finalists: (usize, usize),
    pub sf_teams: Vec<usize>,
    pub qf_teams: Vec<usize>,
    pub r16_teams: Vec<usize>,
    pub r32_teams: Vec<usize>,
    pub slot_winners: HashMap<u32, usize>,
    pub third_place: usize,
    pub group_order: HashMap<String, usize>,
    pub qual_thirds: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn world_new_has_expected_teams_and_groups() {
        let world = World::new();
        assert_eq!(world.teams.len(), 48);
        assert_eq!(world.groups.len(), 12);
        for (_, members) in &world.groups {
            assert_eq!(members.len(), 4);
        }
    }

    #[test]
    fn lam_pair_is_symmetric_and_host_advantage_boosts_lambda() {
        let world = World::new();
        let arg = world.idx["Argentina"];
        let bra = world.idx["Brazil"];

        let (la, lb) = world.lam_pair(arg, bra);
        let (lb2, la2) = world.lam_pair(bra, arg);
        assert!((la - la2).abs() < 1e-9);
        assert!((lb - lb2).abs() < 1e-9);
        assert!(la > 0.0 && lb > 0.0);

        // USA (host) and Iran (non-host) have similar Elo.
        // Hosting should give USA a higher expected goals than Iran.
        let usa = world.idx["United States"];
        let irn = world.idx["Iran"];
        let (usa_home, irn_away) = world.lam_pair(usa, irn);
        let (irn_home, usa_away) = world.lam_pair(irn, usa);
        assert!(usa_home > irn_away);
        assert!(usa_away > irn_home);
        // The host's lambda when playing the same opponent should be the same regardless of order.
        assert!((usa_home - usa_away).abs() < 1e-9);
        assert!((irn_home - irn_away).abs() < 1e-9);
    }

    #[test]
    fn simulate_is_deterministic_for_same_seed() {
        let world = World::new();
        let config = SimConfig {
            n_sims: 1000,
            seed: 42,
            elo_overrides: HashMap::new(),
        };
        let r1 = world.simulate(&config);
        let r2 = world.simulate(&config);
        assert_eq!(r1.champ_counts, r2.champ_counts);
        assert_eq!(r1.final_pairs, r2.final_pairs);
    }

    #[test]
    fn simulate_counts_sum_to_n_sims() {
        let world = World::new();
        let config = SimConfig {
            n_sims: 5000,
            seed: 7,
            elo_overrides: HashMap::new(),
        };
        let results = world.simulate(&config);

        let champ_total: usize = results.champ_counts.values().sum();
        assert_eq!(champ_total, config.n_sims);

        let finalist_total: usize = results.final_counts.values().sum();
        assert_eq!(finalist_total, config.n_sims * 2);

        let sf_total: usize = results.sf_counts.values().sum();
        assert_eq!(sf_total, config.n_sims * 4);

        let r32_total: usize = results.r32_counts.values().sum();
        assert_eq!(r32_total, config.n_sims * 32);
    }

    #[test]
    fn simulate_one_produces_valid_tournament_result() {
        let world = World::new();
        let mut rng = SmallRng::seed_from_u64(99);
        let r = world.simulate_one(&mut rng);

        assert!(r.champion < world.teams.len());
        assert_eq!(r.sf_teams.len(), 4);
        assert_eq!(r.qf_teams.len(), 8);
        assert_eq!(r.r16_teams.len(), 16);
        assert_eq!(r.r32_teams.len(), 32);
        assert_eq!(r.qual_thirds.len(), 8);
        assert!(r.slot_winners.contains_key(&crate::data::FINAL));
    }

    #[test]
    fn assign_thirds_finds_valid_assignment() {
        let qual: Vec<String> = (0..8)
            .map(|i| format!("{}", (b'A' + i as u8) as char))
            .collect();
        let slots = crate::data::third_place_slots();
        let assignment = World::assign_thirds(&qual, &slots);
        assert_eq!(assignment.len(), 8);

        let mut used = std::collections::HashSet::new();
        for (slot, group) in &assignment {
            assert!(slots[slot].contains(&group.as_str()));
            assert!(used.insert(group.clone()));
        }
    }

    #[test]
    fn elo_override_changes_ratings() {
        let mut world = World::new();
        let arg = world.idx["Argentina"];
        let original = world.elo[arg];
        let mut overrides = HashMap::new();
        overrides.insert("Argentina".to_string(), original + 200.0);
        world.apply_overrides(&overrides);
        assert!((world.elo[arg] - (original + 200.0)).abs() < 1e-9);
    }

    #[test]
    fn simulate_one_is_deterministic() {
        let world = World::new();
        let mut rng1 = SmallRng::seed_from_u64(42);
        let mut rng2 = SmallRng::seed_from_u64(42);
        let r1 = world.simulate_one(&mut rng1);
        let r2 = world.simulate_one(&mut rng2);
        assert_eq!(r1.champion, r2.champion);
        assert_eq!(r1.finalists, r2.finalists);
        assert_eq!(r1.qual_thirds, r2.qual_thirds);
    }
}
