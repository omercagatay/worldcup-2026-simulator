import type { GroupRow } from "../api";

const SEGMENTS = [
  { cls: "seg-first", label: "Wins group" },
  { cls: "seg-second", label: "Runner-up" },
  { cls: "seg-thirdq", label: "Third — qualifies" },
  { cls: "seg-thirdout", label: "Third — eliminated" },
  { cls: "seg-fourth", label: "Fourth" },
] as const;

export function GroupTables({ groups }: { groups: [string, GroupRow[]][] }) {
  return (
    <div>
      <div className="seg-legend">
        {SEGMENTS.map((s) => (
          <span key={s.cls}>
            <i className={s.cls} aria-hidden="true" />
            {s.label}
          </span>
        ))}
      </div>
      <div className="groups-grid">
        {groups.map(([letter, rows]) => (
          <section key={letter} className="panel group-card" aria-label={`Group ${letter}`}>
            <header className="panel-head">
              <h3>Group {letter}</h3>
            </header>
            <div className="group-rows">
              {rows.map((r) => {
                const fourth = Math.max(
                  0,
                  100 - r.first_pct - r.second_pct - r.third_q_pct - r.third_out_pct
                );
                const values = [r.first_pct, r.second_pct, r.third_q_pct, r.third_out_pct, fourth];
                const out = r.advance_pct <= 0;
                return (
                  <div key={r.team}>
                    <div className="group-row-top">
                      <span className={`group-team${out ? " team-out" : ""}`}>{r.team}</span>
                      <span className="group-adv">advance {r.advance_pct.toFixed(0)}%</span>
                    </div>
                    <div className="seg-bar">
                      {SEGMENTS.map((s, i) =>
                        values[i] > 0.05 ? (
                          <div
                            key={s.cls}
                            className={`seg ${s.cls}`}
                            style={{ width: `${values[i]}%` }}
                            title={`${s.label}: ${values[i].toFixed(1)}%`}
                          />
                        ) : null
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </section>
        ))}
      </div>
    </div>
  );
}
