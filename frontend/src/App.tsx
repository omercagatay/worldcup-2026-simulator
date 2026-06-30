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

export default function App() {
  const [data, setData] = useState<SimResponse | null>(null);
  const [liveData, setLiveData] = useState<LiveData | null>(null);
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [nSims, setNSims] = useState(50000);
  const [seed, setSeed] = useState(12345);

  const handleSimulate = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await runSimulation({ n_sims: nSims, seed });
      setData(result);
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
    } catch (e) {
      setError(String(e));
    } finally {
      setRefreshing(false);
    }
  }, []);

  const liveMatchCount = liveData
    ? liveData.played_matches.length + (liveData.knockout_matches?.length ?? 0)
    : 0;

  return (
    <div className="app">
      <header className="header">
        <h1>World Cup 2026 — Monte Carlo Simulator</h1>
        <p className="subtitle">
          Elo → Poisson model · {data ? `${data.n_sims.toLocaleString()} simulations` : "ready to run"}
          {data?.scenario_applied && (
            <span className="scenario-badge"> Scenario: {data.scenario_applied}</span>
          )}
          {liveData && (
            <span className="live-badge">
              {" · "}Live: {Object.keys(liveData.elo_ratings).length} Elo ratings, {liveMatchCount} matches
            </span>
          )}
        </p>
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

      {liveData && (
        <LiveStats liveData={liveData} />
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

          <ScenarioPrompt onSubmit={handleScenario} disabled={loading} />

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

          <section className="section">
            <h2>Tournament Win Probabilities</h2>
            <ResultsTable teams={data.teams} />
          </section>

          <section className="section">
            <h2>Representative Bracket</h2>
            <BracketView bracket={data.bracket} champion={data.consensus_champion} />
          </section>

          <section className="section">
            <h2>Group Stage Probabilities</h2>
            <GroupTables groups={data.groups} />
          </section>

          <section className="section">
            <h2>Top Final Matchups</h2>
            <div className="finals-list">
              {data.top_finals.map((f, i) => (
                <div key={i} className="final-pair">
                  <span>
                    <strong>{f.a}</strong> vs <strong>{f.b}</strong>
                    <span style={{ color: "var(--text-dim)", marginLeft: "0.6rem", fontSize: "0.8rem" }}>
                      {f.count.toLocaleString()} sims
                    </span>
                  </span>
                  <span className="pct-tag">{f.pct.toFixed(2)}%</span>
                </div>
              ))}
            </div>
          </section>
        </>
      )}
    </div>
  );
}
