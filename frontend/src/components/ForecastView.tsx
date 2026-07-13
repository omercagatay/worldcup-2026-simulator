import type { SimResponse, LiveData, UpcomingMatch } from "../api";
import { ScenarioPrompt } from "./ScenarioPrompt";
import { ResultsTable } from "./ResultsTable";

export function ForecastView({
  data,
  liveData,
  upcoming,
  loading,
  onScenario,
  liveMatchCount,
  onShowLive,
}: {
  data: SimResponse;
  liveData: LiveData | null;
  upcoming: UpcomingMatch[];
  loading: boolean;
  onScenario: (prompt: string) => void;
  liveMatchCount: number;
  onShowLive: () => void;
}) {
  const contenders = data.top_champions.filter((t) => t.win_pct > 0).slice(0, 6);
  const maxWin = contenders[0]?.win_pct ?? 1;
  const topScorer = liveData?.goalscorers[0];
  const overrides = Object.entries(data.elo_overrides);

  return (
    <div className="forecast">
      {data.scenario_applied && (
        <div className="scenario-note">
          <strong>Scenario:</strong> {data.scenario_applied}
          {overrides.length > 0 && (
            <div className="elo-chips">
              {overrides.map(([team, elo]) => (
                <span key={team} className="elo-chip">
                  {team} → {elo.toFixed(0)}
                </span>
              ))}
            </div>
          )}
        </div>
      )}

      <div className="forecast-grid">
        <div className="forecast-main">
          <section className="panel" aria-label="Title race">
          <header className="panel-head">
            <h2>Title race</h2>
            <span className="eyebrow">
              {data.n_sims.toLocaleString()} tournaments · seed {data.seed}
            </span>
          </header>
          {contenders.map((t, i) => (
            <div key={t.team} className={`race-row${i === 0 ? " race-leader" : ""}`}>
              <span className="race-rank">{i + 1}</span>
              <div>
                <div className="race-top">
                  <span className="race-team">{t.team}</span>
                  {i === 0 && <span className="tag-champ">Most likely champion</span>}
                </div>
                <div className="race-meter">
                  <div
                    className="race-fill"
                    style={{ width: `${(t.win_pct / maxWin) * 100}%` }}
                  />
                </div>
              </div>
              <span className="race-pct">
                {t.win_pct.toFixed(1)}
                <span className="pct-sign">%</span>
              </span>
              </div>
            ))}
          </section>

          <ResultsTable teams={data.teams} />
        </div>

        <aside className="rail">
          <ScenarioPrompt onSubmit={onScenario} disabled={loading} />

          {upcoming.length > 0 && (
            <section className="panel" aria-label="Upcoming matches">
              <header className="panel-head">
                <h3>Next matches</h3>
              </header>
              {upcoming.map((m) => {
                const aFavored = m.a_win_pct >= m.b_win_pct;
                return (
                  <div key={m.match_id} className="fixture">
                    <span className="fixture-round">{m.round}</span>
                    <div className="fixture-teams">
                      <span className={`fixture-team${aFavored ? " favored" : ""}`}>
                        {m.team_a}
                      </span>
                      <span className="fixture-vs">vs</span>
                      <span className={`fixture-team${aFavored ? "" : " favored"}`}>
                        {m.team_b}
                      </span>
                    </div>
                    <div className="split-bar">
                      <div className="split-a" style={{ width: `${m.a_win_pct}%` }} />
                      <div className="split-b" style={{ width: `${m.b_win_pct}%` }} />
                    </div>
                    <div className="split-labels">
                      <span>
                        <i className="split-key a" aria-hidden="true" />
                        {m.a_win_pct.toFixed(1)}%
                      </span>
                      <span>
                        <i className="split-key b" aria-hidden="true" />
                        {m.b_win_pct.toFixed(1)}%
                      </span>
                    </div>
                  </div>
                );
              })}
            </section>
          )}

          <section className="panel" aria-label="Most likely finals">
            <header className="panel-head">
              <h3>Likely finals</h3>
            </header>
            {data.top_finals.slice(0, 5).map((f, i) => (
              <div key={i} className="finals-row">
                <span className="finals-pair">
                  <strong>{f.a}</strong> v <strong>{f.b}</strong>
                </span>
                <span className="finals-pct">{f.pct.toFixed(1)}%</span>
              </div>
            ))}
          </section>

          {liveData && (
            <section className="panel" aria-label="Tournament so far">
              <header className="panel-head">
                <h3>Tournament so far</h3>
              </header>
              <div className="snapshot-rows">
                <div className="snapshot-row">
                  <span>Matches played</span>
                  <strong>{liveMatchCount}</strong>
                </div>
                {liveData.tournament_stats && (
                  <div className="snapshot-row">
                    <span>Goals scored</span>
                    <strong>{liveData.tournament_stats.goals_scored}</strong>
                  </div>
                )}
                {topScorer && (
                  <div className="snapshot-row">
                    <span>Top scorer</span>
                    <strong>
                      {topScorer.player} · {topScorer.goals}
                    </strong>
                  </div>
                )}
              </div>
              <button type="button" className="panel-foot-link" onClick={onShowLive}>
                All live data →
              </button>
            </section>
          )}
        </aside>
      </div>
    </div>
  );
}
