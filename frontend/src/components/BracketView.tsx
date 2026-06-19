import type { BracketSlot } from "../api";

const R32_MATCHES = [
  73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
];
const R16_MATCHES = [89, 90, 91, 92, 93, 94, 95, 96];
const QF_MATCHES = [97, 98, 99, 100];
const SF_MATCHES = [101, 102];

export function BracketView({
  bracket,
  champion,
}: {
  bracket: BracketSlot[];
  champion: string;
}) {
  const teamAt = (id: number) =>
    bracket.find((s) => s.match_id === id)?.team ?? "—";

  const Column = ({
    title,
    matches,
    gap,
  }: {
    title: string;
    matches: number[];
    gap?: boolean;
  }) => (
    <div className="bracket-column" style={gap ? { justifyContent: "space-around" } : undefined}>
      <h4>{title}</h4>
      {matches.map((m) => (
        <div key={m} className="bracket-match">
          <span className="match-id">M{m}</span>
          <span className="match-team">{teamAt(m)}</span>
        </div>
      ))}
    </div>
  );

  return (
    <div className="bracket">
      <Column title="Round of 32" matches={R32_MATCHES} />
      <Column title="Round of 16" matches={R16_MATCHES} gap />
      <Column title="Quarterfinals" matches={QF_MATCHES} gap />
      <Column title="Semifinals" matches={SF_MATCHES} gap />
      <div className="bracket-column" style={{ justifyContent: "flex-end" }}>
        <h4>Final</h4>
        <div className="bracket-match champion-match">
          <span className="match-id">M104</span>
          <span className="match-team champion-team">{champion}</span>
        </div>
      </div>
    </div>
  );
}
