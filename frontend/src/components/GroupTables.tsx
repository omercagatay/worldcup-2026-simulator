import type { GroupRow } from "../api";

export function GroupTables({
  groups,
}: {
  groups: [string, GroupRow[]][];
}) {
  return (
    <div className="groups-grid">
      {groups.map(([letter, rows]) => (
        <div key={letter} className="group-card">
          <h3>Group {letter}</h3>
          <div className="team-list">
            {rows.map((r) => {
              const fourth = Math.max(
                0,
                100 - r.first_pct - r.second_pct - r.third_q_pct - r.third_out_pct
              );
              return (
                <div key={r.team} className="team-row">
                  <div className="team-row-header">
                    <span className="team-row-name">{r.team}</span>
                    <span className="team-row-adv">{r.advance_pct.toFixed(1)}%</span>
                  </div>
                  <div className="team-row-bar">
                    <div
                      className="team-row-bar-fill"
                      style={{ width: `${r.advance_pct}%` }}
                    />
                  </div>
                  <div className="team-row-breakdown">
                    <span className="bd-pos" title="Finish 1st">1st {r.first_pct.toFixed(1)}</span>
                    <span className="bd-pos" title="Finish 2nd">2nd {r.second_pct.toFixed(1)}</span>
                    <span className="bd-pos" title="3rd, qualified">3Q {r.third_q_pct.toFixed(1)}</span>
                    <span className="bd-neg" title="3rd, eliminated">3Out {r.third_out_pct.toFixed(1)}</span>
                    <span className="bd-neg" title="4th, eliminated">4th {fourth.toFixed(1)}</span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
}
