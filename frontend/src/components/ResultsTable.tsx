import type { TeamRow } from "../api";

const rankClass = (i: number) =>
  i === 0 ? "rank-1" : i === 1 ? "rank-2" : i === 2 ? "rank-3" : "";

const medal = (i: number) => `${i + 1}`;

export function ResultsTable({ teams }: { teams: TeamRow[] }) {
  const maxWin = Math.max(...teams.map((t) => t.win_pct), 1);

  return (
    <table className="data-table">
      <thead>
        <tr>
          <th>#</th>
          <th>Team</th>
          <th className="bar-cell">Win Probability</th>
          <th>Odds</th>
          <th>Final%</th>
          <th title="Wins the third-place match">3rd%</th>
          <th>SF%</th>
          <th>QF%</th>
          <th>R16%</th>
          <th>R32%</th>
        </tr>
      </thead>
      <tbody>
        {teams.map((t, i) => (
          <tr key={t.team}>
            <td className={`rank ${rankClass(i)}`}>{medal(i)}</td>
            <td className="team-name">{t.team}</td>
            <td className="bar-cell">
              <div className="bar-wrapper">
                <div className="bar-track">
                  <div
                    className="bar-fill fill-accent"
                    style={{ width: `${(t.win_pct / maxWin) * 100}%` }}
                  />
                </div>
                <span className="bar-value win-cell">{t.win_pct.toFixed(2)}</span>
              </div>
            </td>
            <td className="pct-cell odds-cell">
              {t.win_odds != null ? t.win_odds.toFixed(2) : "—"}
            </td>
            <td className="pct-cell">{t.final_pct.toFixed(2)}</td>
            <td className="pct-cell">{t.third_place_pct.toFixed(2)}</td>
            <td className="pct-cell">{t.sf_pct.toFixed(2)}</td>
            <td className="pct-cell">{t.qf_pct.toFixed(2)}</td>
            <td className="pct-cell">{t.r16_pct.toFixed(2)}</td>
            <td className="pct-cell">{t.r32_pct.toFixed(2)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
