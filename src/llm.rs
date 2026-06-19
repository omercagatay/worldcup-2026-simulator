use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const GLM_API_URL: &str = "https://api.z.ai/api/paas/v4/chat/completions";
const GLM_MODEL: &str = "glm-5.2";

const SYSTEM_PROMPT: &str = r#"You are a football (soccer) analyst for the 2026 FIFA World Cup Monte Carlo simulation.

The simulation uses Elo ratings to model team strength. Each team has a rating around 1400-2100. A 50-point swing changes a team's win probability by roughly 8-10%.

When the user describes a scenario (injury, suspension, tactical change, weather, etc.), you must assess its impact on the affected team(s) and return a JSON object with Elo rating adjustments.

Rules:
- A star player injury (e.g. a team's best striker/keeper) typically reduces a team's Elo by 30-80 points.
- A key defender injury: 20-50 points.
- A squad-wide issue (illness, scandal): 50-120 points.
- A favorable scenario (rival star injured, coaching change for the better): can give positive adjustments.
- Multiple players injured: add their impacts, cap at -150 for a single team.
- Use the EXACT team names from this list:
  Argentina, France, Spain, England, Portugal, Netherlands, Brazil, Belgium, Germany, Croatia, Uruguay, Austria, Colombia, Morocco, Japan, Mexico, United States, Iran, Switzerland, Senegal, Ecuador, Australia, Norway, Turkey, Sweden, South Korea, Ivory Coast, Czech Republic, Scotland, Tunisia, Paraguay, Algeria, Canada, Bosnia and Herzegovina, Saudi Arabia, Egypt, Ghana, DR Congo, Qatar, Panama, Uzbekistan, South Africa, Iraq, Haiti, Jordan, Curaçao, Cape Verde, New Zealand

Return ONLY valid JSON (no markdown fences) in this format:
{
  "analysis": "brief explanation of the impact",
  "adjustments": {
    "TeamName": new_elo_value_as_float,
    ...
  }
}

If a team mentioned is not in the list, omit it. If no teams are affected, return empty adjustments."#;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Deserialize, Debug)]
pub struct ScenarioImpact {
    pub analysis: String,
    pub adjustments: HashMap<String, f64>,
}

pub async fn analyze_scenario(prompt: &str, api_key: &str) -> Result<ScenarioImpact> {
    let client = reqwest::Client::new();
    let req = ChatRequest {
        model: GLM_MODEL.to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: SYSTEM_PROMPT.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
        temperature: 0.3,
    };

    let resp = client
        .post(GLM_API_URL)
        .bearer_auth(api_key)
        .json(&req)
        .send()
        .await
        .context("Failed to call GLM API")?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .context("Failed to read GLM response body")?;
    if !status.is_success() {
        anyhow::bail!("GLM API error {status}: {body}");
    }

    let chat: ChatResponse =
        serde_json::from_str(&body).context("Failed to parse GLM chat response")?;
    let content = chat
        .choices
        .first()
        .context("No choices in GLM response")?
        .message
        .content
        .clone();

    let cleaned = strip_fences(&content);
    let impact: ScenarioImpact = serde_json::from_str(&cleaned)
        .context(format!("Failed to parse impact JSON: {cleaned}"))?;
    Ok(impact)
}

fn strip_fences(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.starts_with("```") {
        let inner = trimmed
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        inner.to_string()
    } else {
        trimmed.to_string()
    }
}
