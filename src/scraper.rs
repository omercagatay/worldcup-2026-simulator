use anyhow::{Context, Result};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const ELO_RATINGS_URL: &str = "https://www.eloratings.net/World.tsv";
const ELO_TEAMS_URL: &str = "https://www.eloratings.net/en.teams.tsv";
const WIKI_API_URL: &str = "https://en.wikipedia.org/w/api.php";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiveData {
    pub elo_ratings: HashMap<String, f64>,
    pub played_matches: Vec<ScrapedMatch>,
    pub knockout_matches: Vec<ScrapedKnockoutMatch>,
    pub goalscorers: Vec<Goalscorer>,
    pub group_standings: Vec<GroupStanding>,
    pub tournament_stats: Option<TournamentStats>,
    pub fetched_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScrapedMatch {
    pub group: String,
    pub team_a: String,
    pub score_a: u16,
    pub team_b: String,
    pub score_b: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScrapedKnockoutMatch {
    pub team_a: String,
    pub score_a: u16,
    pub team_b: String,
    pub score_b: u16,
    pub winner: String,
    pub penalty_score_a: Option<u16>,
    pub penalty_score_b: Option<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Goalscorer {
    pub player: String,
    pub country: String,
    pub goals: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupStanding {
    pub group: String,
    pub team: String,
    pub played: u16,
    pub wins: u16,
    pub draws: u16,
    pub losses: u16,
    pub goals_for: u16,
    pub goals_against: u16,
    pub points: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TournamentStats {
    pub matches_played: u16,
    pub goals_scored: u16,
    pub attendance: u64,
    pub top_scorer: String,
}

pub async fn fetch_all() -> Result<LiveData> {
    let client = reqwest::Client::builder()
        .user_agent("wc2026-sim/0.1 (educational project)")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let elo_ratings = fetch_elo_ratings(&client).await?;
    let html = fetch_wikipedia_html(&client).await?;
    let (played_matches, knockout_matches) = parse_matches(&html);
    let goalscorers = parse_goalscorers(&html);
    let group_standings = parse_group_standings(&html);
    let tournament_stats = parse_tournament_stats(&html);

    Ok(LiveData {
        elo_ratings,
        played_matches,
        knockout_matches,
        goalscorers,
        group_standings,
        tournament_stats,
        fetched_at: chrono_now(),
    })
}

async fn fetch_elo_ratings(client: &reqwest::Client) -> Result<HashMap<String, f64>> {
    let ratings_text = client
        .get(ELO_RATINGS_URL)
        .send()
        .await?
        .text()
        .await
        .context("Failed to fetch Elo ratings TSV")?;

    let teams_text = client
        .get(ELO_TEAMS_URL)
        .send()
        .await?
        .text()
        .await
        .context("Failed to fetch team names TSV")?;

    let code_to_name: HashMap<String, String> = teams_text
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let code = parts[0].to_string();
                let name = parts[1].trim().to_string();
                if !name.is_empty() && !code.contains("_loc") {
                    Some((code, name))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let mut ratings = HashMap::new();
    for line in ratings_text.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 4 {
            let code = parts[2];
            let rating_str = parts[3];
            if let (Some(name), Ok(rating)) = (code_to_name.get(code), rating_str.parse::<f64>()) {
                ratings.insert(name.clone(), rating);
            }
        }
    }

    tracing::info!("Fetched {} Elo ratings from eloratings.net", ratings.len());
    Ok(ratings)
}

async fn fetch_wikipedia_html(client: &reqwest::Client) -> Result<String> {
    let url = format!(
        "{}?action=parse&page=2026_FIFA_World_Cup&format=json&prop=text&formatversion=2",
        WIKI_API_URL
    );

    let resp: serde_json::Value = client
        .get(&url)
        .send()
        .await?
        .json()
        .await
        .context("Failed to fetch Wikipedia HTML")?;

    let html = resp["parse"]["text"].as_str().unwrap_or("").to_string();

    tracing::info!("Fetched {} bytes of HTML from Wikipedia", html.len());
    Ok(html)
}

fn strip_html(s: &str) -> String {
    let fragment = Html::parse_fragment(s);
    fragment
        .root_element()
        .text()
        .collect::<String>()
        .trim()
        .to_string()
}

fn first_link_title(html: &str) -> Option<String> {
    let fragment = Html::parse_fragment(html);
    let selector = Selector::parse("a[title]").ok()?;
    fragment
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("title"))
        .map(|t| t.to_string())
}

#[cfg(test)]
fn parse_match_results(html: &str) -> Vec<ScrapedMatch> {
    parse_matches(html).0
}

fn parse_matches(html: &str) -> (Vec<ScrapedMatch>, Vec<ScrapedKnockoutMatch>) {
    let mut group_matches = Vec::new();
    let mut knockout_matches = Vec::new();
    let document = Html::parse_document(html);
    let footballbox = Selector::parse(".footballbox").expect("valid selector");
    let home_sel = Selector::parse(".fhome").expect("valid selector");
    let score_sel = Selector::parse(".fscore").expect("valid selector");
    let away_sel = Selector::parse(".faway").expect("valid selector");
    let group_link_re = Regex::new(r#"2026_FIFA_World_Cup_Group_([A-L])"#).unwrap();

    for fb in document.select(&footballbox) {
        let fb_html = fb.html();

        let team_a = fb
            .select(&home_sel)
            .next()
            .and_then(|el| first_link_title(&el.html()))
            .map(|t| clean_team_name(&t))
            .unwrap_or_default();
        let team_b = fb
            .select(&away_sel)
            .next()
            .and_then(|el| first_link_title(&el.html()))
            .map(|t| clean_team_name(&t))
            .unwrap_or_default();

        let score_text = fb
            .select(&score_sel)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        if team_a.is_empty() || team_b.is_empty() {
            continue;
        }

        let Some((score_a, score_b)) = parse_score(&score_text) else {
            continue;
        };

        if score_a > 20 || score_b > 20 {
            continue;
        }

        let explicit_group = group_link_re.captures(&fb_html).map(|c| c[1].to_string());
        let inferred_group = match (team_group(&team_a), team_group(&team_b)) {
            (Some(a), Some(b)) if a == b => Some(a),
            _ => None,
        };

        if let Some(group) = explicit_group.or(inferred_group) {
            group_matches.push(ScrapedMatch {
                group,
                team_a,
                score_a,
                team_b,
                score_b,
            });
            continue;
        }

        if let Some((winner, penalty_score_a, penalty_score_b)) =
            knockout_winner(&team_a, score_a, &team_b, score_b, &fb_html)
        {
            knockout_matches.push(ScrapedKnockoutMatch {
                team_a,
                score_a,
                team_b,
                score_b,
                winner,
                penalty_score_a,
                penalty_score_b,
            });
        }
    }

    tracing::info!(
        "Parsed {} group matches and {} knockout matches from Wikipedia HTML",
        group_matches.len(),
        knockout_matches.len()
    );
    (group_matches, knockout_matches)
}

fn parse_score(text: &str) -> Option<(u16, u16)> {
    let re = Regex::new(r"(\d+)\s*[–—-]\s*(\d+)").ok()?;
    let caps = re.captures(text)?;
    let a = caps[1].parse::<u16>().ok()?;
    let b = caps[2].parse::<u16>().ok()?;
    Some((a, b))
}

fn knockout_winner(
    team_a: &str,
    score_a: u16,
    team_b: &str,
    score_b: u16,
    fb_html: &str,
) -> Option<(String, Option<u16>, Option<u16>)> {
    if score_a > score_b {
        return Some((team_a.to_string(), None, None));
    }
    if score_b > score_a {
        return Some((team_b.to_string(), None, None));
    }

    let (penalty_score_a, penalty_score_b) = parse_penalty_score(fb_html)?;
    if penalty_score_a > penalty_score_b {
        Some((
            team_a.to_string(),
            Some(penalty_score_a),
            Some(penalty_score_b),
        ))
    } else if penalty_score_b > penalty_score_a {
        Some((
            team_b.to_string(),
            Some(penalty_score_a),
            Some(penalty_score_b),
        ))
    } else {
        None
    }
}

fn parse_penalty_score(fb_html: &str) -> Option<(u16, u16)> {
    let penalties_pos = fb_html
        .find("Penalties")
        .or_else(|| fb_html.find("penalties"))?;
    let re = Regex::new(r"<th[^>]*>\s*(\d+)\s*[–—-]\s*(\d+)\s*</th>").ok()?;
    let caps = re.captures(&fb_html[penalties_pos..])?;
    let a = caps[1].parse::<u16>().ok()?;
    let b = caps[2].parse::<u16>().ok()?;
    Some((a, b))
}

fn team_group(team: &str) -> Option<String> {
    for (letter, teams) in crate::data::groups() {
        if teams.contains(&team) {
            return Some(letter.to_string());
        }
    }
    None
}

#[cfg(test)]
fn infer_group(team: &str) -> String {
    for (letter, teams) in crate::data::groups() {
        if teams.contains(&team) {
            return letter.to_string();
        }
    }
    "?".to_string()
}

fn clean_team_name(title: &str) -> String {
    let apos = format!("{}{}{}{}{}", '&', '#', "39", ';', "");
    title
        .replace(&apos, "\x27")
        .replace("&amp;", "&")
        .replace("&#160;", " ")
        .replace(" national football team", "")
        .replace(" national soccer team", "")
        .replace(" men's national soccer team", "")
        .replace(" men's national football team", "")
        .replace(" men's", "")
        .trim()
        .to_string()
}

fn parse_goalscorers(html: &str) -> Vec<Goalscorer> {
    let mut scorers = Vec::new();

    let goalscorers_start = html.find(r#"id="Goalscorers""#);
    let discipline_start = html.find(r#"id="Discipline""#);

    if let (Some(start), Some(end)) = (goalscorers_start, discipline_start) {
        let section = &html[start..end];

        let goals_header_re = Regex::new(r"<b>(\d+)\s*goals?</b>").unwrap();
        let li_re = Regex::new(r"<li>(.*?)</li>").unwrap();
        let flag_re = Regex::new(r#"<a[^>]*title="([^"]*national[^"]*)"[^>]*>"#).unwrap();
        let player_re = Regex::new(r#"<a[^>]*href="/wiki/([^"]*)"[^>]*>([^<]*)</a>"#).unwrap();

        let mut current_goals: u16 = 0;
        for chunk in section.split("<b>") {
            if let Some(caps) = goals_header_re.captures(&format!("<b>{}", chunk)) {
                current_goals = caps[1].parse::<u16>().unwrap_or(0);
            }

            for li_caps in li_re.captures_iter(chunk) {
                let li_html = &li_caps[1];

                let country = flag_re
                    .captures(li_html)
                    .map(|c| clean_team_name(&c[1]))
                    .unwrap_or_default();

                let player = player_re
                    .captures_iter(li_html)
                    .filter(|c| !c[1].contains("national"))
                    .map(|c| c[2].trim().to_string())
                    .next()
                    .unwrap_or_else(|| strip_html(li_html));

                if !player.is_empty() && !country.is_empty() && current_goals > 0 {
                    scorers.push(Goalscorer {
                        player,
                        country,
                        goals: current_goals,
                    });
                }
            }
        }
    }

    scorers.sort_by_key(|b| std::cmp::Reverse(b.goals));
    tracing::info!("Parsed {} goalscorers from Wikipedia HTML", scorers.len());
    scorers
}

fn parse_group_standings(html: &str) -> Vec<GroupStanding> {
    let mut standings = Vec::new();

    let group_header_re = Regex::new(r#"<h3[^>]*id="Group_([A-L])""#).unwrap();
    let table_re =
        Regex::new(r#"<table[^>]*class="[^"]*wikitable[^"]*"[^>]*>(.*?)</table>"#).unwrap();
    let row_sel = Selector::parse("tr").expect("valid selector");
    let cell_sel = Selector::parse("th, td").expect("valid selector");

    let mut current_group: Option<String> = None;

    for chunk in html.split(r#"<h3"#) {
        if let Some(caps) = group_header_re.captures(&format!("<h3{}", chunk)) {
            current_group = Some(caps[1].to_string());
        }

        if let Some(ref group) = current_group {
            if let Some(table_caps) = table_re.captures(chunk) {
                let table_html = &table_caps[0];
                let table = Html::parse_fragment(table_html);

                for row in table.select(&row_sel) {
                    let cells: Vec<String> = row
                        .select(&cell_sel)
                        .map(|c| strip_html(&c.html()))
                        .collect();

                    if cells.len() < 9 {
                        continue;
                    }

                    let team = cells
                        .iter()
                        .find(|c| {
                            !c.is_empty()
                                && !c.chars().all(|ch| {
                                    ch.is_numeric()
                                        || ch == '-'
                                        || ch == '+'
                                        || ch == '('
                                        || ch == ')'
                                })
                        })
                        .cloned()
                        .unwrap_or_default();

                    let nums: Vec<u16> =
                        cells.iter().filter_map(|c| c.parse::<u16>().ok()).collect();

                    if nums.len() >= 7 && !team.is_empty() {
                        standings.push(GroupStanding {
                            group: group.clone(),
                            team,
                            played: nums[0],
                            wins: nums[1],
                            draws: nums[2],
                            losses: nums[3],
                            goals_for: nums[4],
                            goals_against: nums[5],
                            points: nums[6],
                        });
                    }
                }
            }
        }
    }

    tracing::info!(
        "Parsed {} group standings from Wikipedia HTML",
        standings.len()
    );
    standings
}

fn parse_tournament_stats(html: &str) -> Option<TournamentStats> {
    let matches_re = Regex::new(r"matches[_-]played[^>]*>.*?(\d+)").unwrap();
    let goals_re = Regex::new(r"goals[_-]scored[^>]*>.*?(\d+)").unwrap();
    let attendance_re = Regex::new(r"[Aa]tendance[^>]*>.*?([\d,]+)").unwrap();
    let scorer_link_re = Regex::new(
        r#"[Tt]op\s*scorer.*?<a[^>]*title="([^"]*national[^"]*)"[^>]*>.*?<a[^>]*>([^<]*)</a>"#,
    )
    .unwrap();

    let matches_played = matches_re
        .captures(html)
        .and_then(|c| c[1].parse::<u16>().ok());
    let goals_scored = goals_re
        .captures(html)
        .and_then(|c| c[1].parse::<u16>().ok());
    let attendance = attendance_re
        .captures(html)
        .and_then(|c| c[1].replace(',', "").parse::<u64>().ok());
    let top_scorer = scorer_link_re
        .captures(html)
        .map(|c| c[2].replace("&#39;", "\x27").trim().to_string());

    if matches_played.is_some() || goals_scored.is_some() {
        Some(TournamentStats {
            matches_played: matches_played.unwrap_or(0),
            goals_scored: goals_scored.unwrap_or(0),
            attendance: attendance.unwrap_or(0),
            top_scorer: top_scorer.unwrap_or_default(),
        })
    } else {
        None
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("unix:{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_team_name_handles_common_suffixes() {
        assert_eq!(
            clean_team_name("Argentina national football team"),
            "Argentina"
        );
        assert_eq!(
            clean_team_name("United States men's national soccer team"),
            "United States"
        );
        assert_eq!(
            clean_team_name("Bosnia and Herzegovina"),
            "Bosnia and Herzegovina"
        );
    }

    #[test]
    fn parse_score_parses_various_dashes() {
        assert_eq!(parse_score("2 – 1"), Some((2, 1)));
        assert_eq!(parse_score("3-0"), Some((3, 0)));
        assert_eq!(parse_score("1—1"), Some((1, 1)));
        assert_eq!(parse_score("no score"), None);
    }

    #[test]
    fn infer_group_finds_known_team() {
        assert_eq!(infer_group("Argentina"), "J");
        assert_eq!(infer_group("Atlantis"), "?");
    }

    #[test]
    fn parse_match_results_extracts_footballbox() {
        let html = r#"
        <section>
          <h3 id="Group_A">Group A</h3>
          <table class="footballbox">
            <tr>
              <td class="fhome"><a title="Mexico national football team">Mexico</a></td>
              <td class="fscore">2 – 1</td>
              <td class="faway"><a title="South Africa national football team">South Africa</a></td>
            </tr>
          </table>
        </section>
        "#;
        let matches = parse_match_results(html);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].team_a, "Mexico");
        assert_eq!(matches[0].team_b, "South Africa");
        assert_eq!(matches[0].score_a, 2);
        assert_eq!(matches[0].score_b, 1);
        assert_eq!(matches[0].group, "A");
    }

    #[test]
    fn parse_matches_extracts_knockout_penalty_winner() {
        let html = r#"
        <table class="footballbox">
          <tr>
            <th class="fhome"><a title="Germany national football team">Germany</a></th>
            <th class="fscore"><a href="/wiki/2026_FIFA_World_Cup_knockout_stage#Germany_vs_Paraguay">1–1</a> (a.e.t.)</th>
            <th class="faway"><a title="Paraguay national football team">Paraguay</a></th>
          </tr>
          <tr><th colspan="3">Penalties</th></tr>
          <tr><td></td><th>3–4</th><td></td></tr>
        </table>
        "#;
        let (group_matches, knockout_matches) = parse_matches(html);
        assert!(group_matches.is_empty());
        assert_eq!(knockout_matches.len(), 1);
        assert_eq!(knockout_matches[0].team_a, "Germany");
        assert_eq!(knockout_matches[0].team_b, "Paraguay");
        assert_eq!(knockout_matches[0].winner, "Paraguay");
        assert_eq!(knockout_matches[0].penalty_score_a, Some(3));
        assert_eq!(knockout_matches[0].penalty_score_b, Some(4));
    }

    #[test]
    fn parse_match_results_skips_invalid_scores() {
        let html = r#"
        <table class="footballbox">
          <tr>
            <td class="fhome"><a title="Mexico national football team">Mexico</a></td>
            <td class="fscore">TBD</td>
            <td class="faway"><a title="South Africa national football team">South Africa</a></td>
          </tr>
        </table>
        "#;
        assert!(parse_match_results(html).is_empty());
    }

    #[test]
    fn strip_html_removes_tags() {
        assert_eq!(strip_html("<b>2</b> goals"), "2 goals");
    }
}
