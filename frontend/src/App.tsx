import { useState, useCallback } from "react";
import {
  runSimulation,
  runScenario,
  refreshLiveData,
  type SimResponse,
  type LiveData,
} from "./api";
import { ResultsTable } from "./components/ResultsTable";
import { GroupTables } from "./components/GroupTables";
import { BracketView } from "./components/BracketView";
import { ScenarioPrompt } from "./components/ScenarioPrompt";
import { LiveStats } from "./components/LiveStats";

type DashboardView = "forecast" | "bracket" | "groups" | "live";

export default function App() {
  const [data, setData] = useState<SimResponse | null>(null);
  const [liveData, setLiveData] = useState<LiveData | null>(null);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [nSims, setNSims] = useState(50000);
  const [seed, setSeed] = useState(12345);
  const [activeView, setActiveView] = useState<DashboardView>("forecast");

  const handleSimulate = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await runSimulation({ n_sims: nSims, seed });
      setData(result);
      setActiveView("forecast");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [nSims, seed]);

  const handleScenario = useCallback(
    async (prompt: string) => {
      setLoading(true);
      setError(null);
      try {
        const result = await runScenario({ prompt, n_sims: nSims, seed });
        setData(result);
        setActiveView("forecast");
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    },
    [nSims, seed]
  );

  const handleRefresh = useCallback(async () => {
    setRefreshing(true);
    setError(null);
    try {
      const live = await refreshLiveData();
      setLiveData(live);
      if (!data) setActiveView("live");
    } catch (e) {
      setError(String(e));
    } finally {
      setRefreshing(false);
    }
  }, [data]);

  const liveMatchCount = liveData
    ? liveData.played_matches.length + (liveData.knockout_matches?.length ?? 0)
    : 0;
  const topChampion = data?.top_champions[0];
  const topScorer = liveData?.goalscorers[0];
  const topFinal = data?.top_finals[0];
  const topContenders = data?.top_champions.slice(0, 5) ?? [];
  const recentKnockouts = liveData?.knockout_matches?.slice(-4).reverse() ?? [];

  return (
    <div className="app">
      <header className="header dashboard-header">
        <div>
          <span className="eyebrow">Forecast console</span>
          <h1>World Cup 2026 Simulator</h1>
        </div>
        <div className="status-pills">
          <span className="status-pill">{data ? `${data.n_sims.toLocaleString()} sims` : "Ready"}</span>
          {liveData && <span className="status-pill live">Live {liveMatchCount} matches</span>}
          {data?.scenario_applied && <span className="status-pill scenario">Scenario applied</span>}
        </div>
      </header>

      <div className="controls">
        <label>
          Simulations:
          <input
            type="number"
            value={nSims}
            onChange={(e) => setNSims(Number(e.target.value))}
            min={100}
            max={200000}
            step={1000}
          />
        </label>
        <label>
          Seed:
          <input
            type="number"
            value={seed}
            onChange={(e) => setSeed(Number(e.target.value))}
          />
        </label>
        <button onClick={handleSimulate} disabled={loading}>
          {loading ? "Running…" : "Run Simulation"}
        </button>
        <button
          className="refresh-btn"
          onClick={handleRefresh}
          disabled={refreshing}
        >
          {refreshing ? "Scraping…" : "Refresh Live Data"}
        </button>
      </div>

      {error && <div className="error">{error}</div>}

      {loading && !data && <div className="loading">Running Monte Carlo simulation…</div>}

      {!data && !loading && !error && (
        <div className="empty-state">
          <p>Click <strong>Run Simulation</strong> to start.</p>
          <p className="empty-hint">50,000 tournaments will be simulated in parallel.</p>
        </div>
      )}

      {(data || liveData) && (
        <section className="summary-grid">
          <div className="summary-item primary">
            <span className="summary-label">Champion Mode</span>
            <strong>{topChampion?.team ?? "-"}</strong>
            <span>{topChampion ? `${topChampion.win_pct.toFixed(2)}% win` : "Run a simulation"}</span>
          </div>
          <div className="summary-item">
            <span className="summary-label">Golden Boot</span>
            <strong>{topScorer?.player ?? "-"}</strong>
            <span>{topScorer ? `${topScorer.goals} goals · ${topScorer.country}` : "Refresh live data"}</span>
          </div>
          <div className="summary-item">
            <span className="summary-label">Top Final</span>
            <strong>{topFinal ? `${topFinal.a} vs ${topFinal.b}` : "-"}</strong>
            <span>{topFinal ? `${topFinal.pct.toFixed(2)}% of sims` : "Run a simulation"}</span>
          </div>
          <div className="summary-item">
            <span className="summary-label">Live Coverage</span>
            <strong>{liveData ? liveMatchCount : "-"}</strong>
            <span>{liveData ? `${Object.keys(liveData.elo_ratings).length} Elo ratings` : "Not loaded"}</span>
          </div>
        </section>
      )}

      {data && (
        <>
          {data.scenario_applied && (
            <div className="scenario-info">
              <strong>LLM Analysis:</strong> {data.scenario_applied}
              {Object.keys(data.elo_overrides).length > 0 && (
                <div className="elo-deltas">
                  <strong>Elo adjustments:</strong>
                  {Object.entries(data.elo_overrides).map(([team, elo]) => (
                    <span key={team} className="elo-delta">
                      {team}: {elo}
                    </span>
                  ))}
                </div>
              )}
            </div>
          )}

          <div className="analysis-layout">
            <main className="forecast-panel">
              {data.consensus_champion && (
                <div className="champion-banner">
                  <div className="champion-label">Consensus Champion</div>
                  <div className="champion-name">{data.consensus_champion}</div>
                  <div className="champion-odds">
                    {data.top_champions[0]?.win_pct.toFixed(2)}% win rate
                    {" · "}
                    {data.n_sims.toLocaleString()} simulations
                  </div>
                </div>
              )}

              <section className="contender-strip">
                <div className="strip-heading">
                  <span className="summary-label">Win Leaders</span>
                  <strong>Top contenders</strong>
                </div>
                <div className="contender-list">
                  {topContenders.map((team, i) => (
                    <div key={team.team} className="contender-row">
                      <span className="contender-rank">{i + 1}</span>
                      <span className="contender-team">{team.team}</span>
                      <span className="contender-pct">{team.win_pct.toFixed(2)}%</span>
                    </div>
                  ))}
                </div>
              </section>
            </main>

            <aside className="insight-rail">
              <ScenarioPrompt onSubmit={handleScenario} disabled={loading} />

              <section className="rail-panel">
                <h3>Likely Finals</h3>
                <div className="finals-list compact">
                  {data.top_finals.slice(0, 5).map((f, i) => (
                    <div key={i} className="final-pair">
                      <span>
                        <strong>{f.a}</strong> vs <strong>{f.b}</strong>
                      </span>
                      <span className="pct-tag">{f.pct.toFixed(2)}%</span>
                    </div>
                  ))}
                </div>
              </section>

              {liveData && (
                <section className="rail-panel live-snapshot">
                  <h3>Live Snapshot</h3>
                  <div className="snapshot-grid">
                    <span>Matches</span>
                    <strong>{liveMatchCount}</strong>
                    <span>Goals</span>
                    <strong>{liveData.tournament_stats?.goals_scored ?? "-"}</strong>
                    <span>Golden Boot</span>
                    <strong>{topScorer ? `${topScorer.player} (${topScorer.goals})` : "-"}</strong>
                  </div>
                  {recentKnockouts.length > 0 && (
                    <div className="recent-knockouts">
                      {recentKnockouts.map((m, i) => (
                        <span key={i}>{m.winner} advanced</span>
                      ))}
                    </div>
                  )}
                </section>
              )}
            </aside>
          </div>
        </>
      )}

      {(data || liveData) && (
        <>
          <nav className="view-tabs" aria-label="Dashboard views">
            {data && (
              <button
                type="button"
                className={activeView === "forecast" ? "active" : ""}
                onClick={() => setActiveView("forecast")}
              >
                Forecast
              </button>
            )}
            {data && (
              <button
                type="button"
                className={activeView === "bracket" ? "active" : ""}
                onClick={() => setActiveView("bracket")}
              >
                Bracket
              </button>
            )}
            {data && (
              <button
                type="button"
                className={activeView === "groups" ? "active" : ""}
                onClick={() => setActiveView("groups")}
              >
                Groups
              </button>
            )}
            {liveData && (
              <button
                type="button"
                className={activeView === "live" ? "active" : ""}
                onClick={() => setActiveView("live")}
              >
                Live Data
              </button>
            )}
          </nav>

          {data && activeView === "forecast" && (
            <section className="section tab-panel">
              <h2>Tournament Win Probabilities</h2>
              <ResultsTable teams={data.teams} />
            </section>
          )}

          {data && activeView === "bracket" && (
            <section className="section tab-panel">
              <h2>Representative Bracket</h2>
              <BracketView bracket={data.bracket} champion={data.consensus_champion} />
            </section>
          )}

          {data && activeView === "groups" && (
            <section className="section tab-panel">
              <h2>Group Stage Probabilities</h2>
              <GroupTables groups={data.groups} />
            </section>
          )}

          {liveData && activeView === "live" && (
            <LiveStats liveData={liveData} />
          )}
        </>
      )}
    </div>
  );
}
