import { useState } from "react";
import type { TeamRow } from "../api";

/* Heat tint for round-probability cells: one sequential hue (turf),
   opacity scaled by the value so magnitude reads at a glance. Mixed from
   the token so it tracks the active theme. */
const heat = (pct: number) =>
  pct > 0
    ? { background: `color-mix(in srgb, var(--turf) ${(5 + (pct / 100) * 22).toFixed(1)}%, transparent)` }
    : undefined;

const Pct = ({ v }: { v: number }) =>
  v > 0 ? <>{v.toFixed(1)}</> : <span className="cell-zero">–</span>;

export function ResultsTable({ teams }: { teams: TeamRow[] }) {
  const alive = teams.filter((t) => t.win_pct > 0);
  const hasEliminated = alive.length > 0 && alive.length < teams.length;
  const [showAll, setShowAll] = useState(!hasEliminated);
  const rows = showAll || !hasEliminated ? teams : alive;
  const maxWin = Math.max(...teams.map((t) => t.win_pct), 0.001);

  return (
    <section className="panel table-panel" aria-label="Round-by-round probabilities">
      <header className="panel-head">
        <h2>Road to the final</h2>
        {hasEliminated && (
          <div className="table-toggle" role="group" aria-label="Teams shown">
            <button type="button" aria-pressed={!showAll} onClick={() => setShowAll(false)}>
              In contention · {alive.length}
            </button>
            <button type="button" aria-pressed={showAll} onClick={() => setShowAll(true)}>
              All {teams.length}
            </button>
          </div>
        )}
      </header>
      <div className="table-scroll">
        <table className="data-table">
          <thead>
            <tr>
              <th aria-label="Rank">#</th>
              <th className="col-team">Team</th>
              <th className="cell-title">Champion</th>
              <th title="Decimal odds">Odds</th>
              <th title="Reaches the final">Final</th>
              <th title="Reaches the semifinals">SF</th>
              <th title="Reaches the quarterfinals">QF</th>
              <th title="Reaches the round of 16">R16</th>
              <th title="Reaches the round of 32">R32</th>
              <th title="Wins the third-place match">3rd place</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((t) => {
              const out = t.win_pct <= 0;
              return (
                <tr key={t.team} className={out ? "row-out" : undefined}>
                  <td className="cell-rank">{teams.indexOf(t) + 1}</td>
                  <td className="cell-team">{t.team}</td>
                  <td className="cell-title">
                    {out ? (
                      <span className="cell-zero">–</span>
                    ) : (
                      <div className="title-meter">
                        <div className="title-track">
                          <div
                            className="title-fill"
                            style={{ width: `${(t.win_pct / maxWin) * 100}%` }}
                          />
                        </div>
                        <span className="title-val">{t.win_pct.toFixed(1)}%</span>
                      </div>
                    )}
                  </td>
                  <td>
                    {t.win_odds != null ? (
                      t.win_odds.toFixed(2)
                    ) : (
                      <span className="cell-zero">–</span>
                    )}
                  </td>
                  <td style={heat(t.final_pct)}>
                    <Pct v={t.final_pct} />
                  </td>
                  <td style={heat(t.sf_pct)}>
                    <Pct v={t.sf_pct} />
                  </td>
                  <td style={heat(t.qf_pct)}>
                    <Pct v={t.qf_pct} />
                  </td>
                  <td style={heat(t.r16_pct)}>
                    <Pct v={t.r16_pct} />
                  </td>
                  <td style={heat(t.r32_pct)}>
                    <Pct v={t.r32_pct} />
                  </td>
                  <td style={heat(t.third_place_pct)}>
                    <Pct v={t.third_place_pct} />
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </section>
  );
}
