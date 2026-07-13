import type { LiveData } from "../api";

export function LiveStats({ liveData }: { liveData: LiveData }) {
  const topScorers = [...liveData.goalscorers].sort((a, b) => b.goals - a.goals).slice(0, 15);
  const eloEntries = Object.entries(liveData.elo_ratings)
    .sort(([, a], [, b]) => b - a)
    .slice(0, 10);
  const knockoutMatches = liveData.knockout_matches ?? [];
  const stats = liveData.tournament_stats;

  return (
    <div>
      {stats && (
        <div className="tiles">
          <div className="tile">
            <span className="tile-label">Matches played</span>
            <span className="tile-value">{stats.matches_played}</span>
          </div>
          <div className="tile">
            <span className="tile-label">Goals scored</span>
            <span className="tile-value">{stats.goals_scored}</span>
          </div>
          <div className="tile">
            <span className="tile-label">Attendance</span>
            <span className="tile-value">
              {stats.attendance > 0 ? `${(stats.attendance / 1_000_000).toFixed(2)}M` : "—"}
            </span>
          </div>
          <div className="tile">
            <span className="tile-label">Top scorer</span>
            <span className="tile-value">{stats.top_scorer || "—"}</span>
          </div>
        </div>
      )}

      <div className="live-grid">
        <section className="panel" aria-label="Top goalscorers">
          <header className="panel-head">
            <h3>Golden boot race</h3>
          </header>
          <div className="roster">
            {topScorers.map((s, i) => (
              <div key={`${s.player}-${i}`} className="roster-row">
                <span className="roster-rank">{i + 1}</span>
                <span className="roster-name">
                  {s.player}
                  {s.active && <span className="active-dot" title="Still in the tournament" />}
                </span>
                <span className="roster-sub">{s.country}</span>
                <span className="roster-val">{s.goals}</span>
              </div>
            ))}
          </div>
        </section>

        <section className="panel" aria-label="Elo ratings">
          <header className="panel-head">
            <h3>Elo top 10</h3>
          </header>
          <div className="roster">
            {eloEntries.map(([team, rating], i) => (
              <div key={team} className="roster-row">
                <span className="roster-rank">{i + 1}</span>
                <span className="roster-name">{team}</span>
                <div className="elo-meter">
                  <div className="elo-track">
                    <div
                      className="elo-fill"
                      style={{
                        width: `${Math.min(100, Math.max(0, ((rating - 1400) / 800) * 100))}%`,
                      }}
                    />
                  </div>
                </div>
                <span className="roster-val">{rating.toFixed(0)}</span>
              </div>
            ))}
          </div>
        </section>

        {liveData.played_matches.length > 0 && (
          <section className="panel" aria-label="Group stage results">
            <header className="panel-head">
              <h3>Group results · {liveData.played_matches.length}</h3>
            </header>
            <div className="roster">
              {liveData.played_matches.map((m, i) => (
                <div key={i} className="result-row">
                  <span>
                    {m.team_a}{" "}
                    <span className="result-score">
                      {m.score_a}–{m.score_b}
                    </span>{" "}
                    {m.team_b}
                  </span>
                  <span className="result-note">Group {m.group}</span>
                </div>
              ))}
            </div>
          </section>
        )}

        {knockoutMatches.length > 0 && (
          <section className="panel" aria-label="Knockout results">
            <header className="panel-head">
              <h3>Knockout results · {knockoutMatches.length}</h3>
            </header>
            <div className="roster">
              {knockoutMatches.map((m, i) => {
                const pens =
                  m.penalty_score_a != null && m.penalty_score_b != null
                    ? ` (${m.penalty_score_a}–${m.penalty_score_b} pens)`
                    : "";
                return (
                  <div key={i} className="result-row">
                    <span>
                      {m.team_a}{" "}
                      <span className="result-score">
                        {m.score_a}–{m.score_b}
                      </span>{" "}
                      {m.team_b}
                      {pens}
                    </span>
                    <span className="result-note">{m.winner} through</span>
                  </div>
                );
              })}
            </div>
          </section>
        )}
      </div>

      <p className="source-note">Live data scraped from eloratings.net and Wikipedia.</p>
    </div>
  );
}
