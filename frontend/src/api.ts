export interface TeamRow {
  team: string;
  win_pct: number;
  win_odds: number | null;
  third_place_pct: number;
  final_pct: number;
  sf_pct: number;
  qf_pct: number;
  r16_pct: number;
  r32_pct: number;
}

export interface GroupRow {
  team: string;
  first_pct: number;
  second_pct: number;
  third_q_pct: number;
  third_out_pct: number;
  advance_pct: number;
}

export interface BracketSlot {
  match_id: number;
  team_a: string;
  team_b: string;
  winner: string;
}

export interface FinalPair {
  a: string;
  b: string;
  pct: number;
  count: number;
}

export interface SimResponse {
  n_sims: number;
  seed: number;
  teams: TeamRow[];
  groups: [string, GroupRow[]][];
  bracket: BracketSlot[];
  consensus_champion: string;
  top_finals: FinalPair[];
  top_champions: TeamRow[];
  elo_overrides: Record<string, number>;
  scenario_applied: string | null;
}

export interface SimRequest {
  n_sims?: number;
  seed?: number;
  elo_overrides?: Record<string, number>;
}

export interface ScenarioRequest {
  prompt: string;
  n_sims?: number;
  seed?: number;
}

const API_BASE = "";

export async function runSimulation(req: SimRequest): Promise<SimResponse> {
  const resp = await fetch(`${API_BASE}/api/simulate`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  if (!resp.ok) throw new Error(await resp.text());
  return resp.json();
}

export async function runScenario(req: ScenarioRequest): Promise<SimResponse> {
  const resp = await fetch(`${API_BASE}/api/scenario`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
  if (!resp.ok) throw new Error(await resp.text());
  return resp.json();
}

export interface LiveData {
  elo_ratings: Record<string, number>;
  played_matches: {
    group: string;
    team_a: string;
    score_a: number;
    team_b: string;
    score_b: number;
  }[];
  knockout_matches?: {
    team_a: string;
    score_a: number;
    team_b: string;
    score_b: number;
    winner: string;
    penalty_score_a?: number | null;
    penalty_score_b?: number | null;
  }[];
  goalscorers: {
    player: string;
    country: string;
    goals: number;
    active?: boolean;
  }[];
  group_standings: {
    group: string;
    team: string;
    played: number;
    wins: number;
    draws: number;
    losses: number;
    goals_for: number;
    goals_against: number;
    points: number;
  }[];
  tournament_stats: {
    matches_played: number;
    goals_scored: number;
    attendance: number;
    top_scorer: string;
  } | null;
  fetched_at: string;
}

export async function refreshLiveData(): Promise<LiveData> {
  const resp = await fetch(`${API_BASE}/api/refresh`, { method: "POST" });
  if (!resp.ok) throw new Error(await resp.text());
  return resp.json();
}

/** Cached live data (kept fresh by the backend's background refresh). */
export async function getLiveData(): Promise<LiveData | null> {
  const resp = await fetch(`${API_BASE}/api/live`);
  if (!resp.ok) throw new Error(await resp.text());
  return resp.json();
}

export interface UpcomingMatch {
  match_id: number;
  round: string;
  team_a: string;
  team_b: string;
  a_win_pct: number;
  b_win_pct: number;
  decided_in_90_pct: number;
  a_win_odds: number | null;
  b_win_odds: number | null;
}

export interface UpcomingResponse {
  matches: UpcomingMatch[];
}

/** Forecasts for real bracket matches not yet played. */
export async function getUpcoming(): Promise<UpcomingResponse> {
  const resp = await fetch(`${API_BASE}/api/upcoming`);
  if (!resp.ok) throw new Error(await resp.text());
  return resp.json();
}
