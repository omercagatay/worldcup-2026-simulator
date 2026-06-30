import type { LiveData } from "../api";

export function LiveStats({ liveData }: { liveData: LiveData }) {
  const topScorers = [...liveData.goalscorers].sort((a, b) => b.goals - a.goals).slice(0, 15);
  const topElo = Object.entries(liveData.elo_ratings)
    .sort(([, a], [, b]) => b - a)
    .slice(0, 10);
  const knockoutMatches = liveData.knockout_matches ?? [];

  return (
    <section className="section live-stats">
      <h2>Live Data from eloratings.net + Wikipedia</h2>

      {liveData.tournament_stats && (
        <div className="live-tournament-stats">
          <div className="stat-card">
            <span className="stat-value">{liveData.tournament_stats.matches_played}</span>
            <span className="stat-label">Matches Played</span>
          </div>
          <div className="stat-card">
            <span className="stat-value">{liveData.tournament_stats.goals_scored}</span>
            <span className="stat-label">Goals Scored</span>
          </div>
          <div className="stat-card">
            <span className="stat-value">
              {liveData.tournament_stats.attendance > 0
                ? (liveData.tournament_stats.attendance / 1_000_000).toFixed(2) + "M"
                : "—"}
            </span>
            <span className="stat-label">Attendance</span>
          </div>
          <div className="stat-card">
            <span className="stat-value stat-scorer">{liveData.tournament_stats.top_scorer || "—"}</span>
            <span className="stat-label">Top Scorer</span>
          </div>
        </div>
      )}

      <div className="live-grid">
        <div className="live-card">
          <h3>Top Goalscorers</h3>
          <div className="scorers-list">
            {topScorers.map((s, i) => (
              <div key={i} className={`scorer-row${s.active ? " scorer-active" : ""}`}>
                <span className="scorer-rank">{i + 1}</span>
                <span className="scorer-name">{s.player}</span>
                <span className="scorer-country">{s.country}</span>
                {s.active && <span className="active-chip">Active</span>}
                <span className={`scorer-goals ${s.goals >= 4 ? "scorer-top" : ""}`}>
                  {s.goals}
                </span>
              </div>
            ))}
          </div>
        </div>

        <div className="live-card">
          <h3>Live Elo Ratings (Top 10)</h3>
          <div className="elo-list">
            {topElo.map(([team, rating], i) => (
              <div key={team} className="elo-row">
                <span className="elo-rank">{i + 1}</span>
                <span className="elo-team">{team}</span>
                <div className="elo-bar-wrapper">
                  <div className="elo-bar-track">
                    <div
                      className="elo-bar-fill"
                      style={{ width: `${((rating - 1400) / 800) * 100}%` }}
                    />
                  </div>
                </div>
                <span className="elo-value">{rating.toFixed(0)}</span>
              </div>
            ))}
          </div>
        </div>

        <div className="live-card">
          <h3>Group Matches ({liveData.played_matches.length})</h3>
          <div className="matches-list">
            {liveData.played_matches.map((m, i) => (
              <div key={i} className="match-row">
                <span className="match-group">{m.group}</span>
                <span className="match-teams">
                  {m.team_a} <strong>{m.score_a}–{m.score_b}</strong> {m.team_b}
                </span>
              </div>
            ))}
          </div>
        </div>
        {knockoutMatches.length > 0 && (
          <div className="live-card">
            <h3>Knockout Results ({knockoutMatches.length})</h3>
            <div className="matches-list">
              {knockoutMatches.map((m, i) => {
                const penaltyText =
                  m.penalty_score_a != null && m.penalty_score_b != null
                    ? ` (${m.penalty_score_a}–${m.penalty_score_b} pens)`
                    : "";
                return (
                  <div key={i} className="match-row">
                    <span className="match-group">KO</span>
                    <span className="match-teams">
                      {m.team_a} <strong>{m.score_a}–{m.score_b}</strong> {m.team_b}
                      {penaltyText} · <strong>{m.winner}</strong> advanced
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        )}

      </div>
    </section>
  );
}
