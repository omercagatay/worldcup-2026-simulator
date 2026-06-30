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
  const matchAt = (id: number) => bracket.find((s) => s.match_id === id);
  const nameOrDash = (name: string | undefined) => name && name.length > 0 ? name : "—";

  const TeamLine = ({ name, winner }: { name: string | undefined; winner: boolean }) => (
    <span className={`match-team-line${winner ? " match-team-winner" : ""}`}>
      {nameOrDash(name)}
    </span>
  );

  const MatchCard = ({ id, championMatch = false }: { id: number; championMatch?: boolean }) => {
    const slot = matchAt(id);
    const winner = slot?.winner ?? (championMatch ? champion : undefined);

    return (
      <div className={`bracket-match${championMatch ? " champion-match" : ""}`}>
        <div className="match-header">
          <span className="match-id">M{id}</span>
          <span className="match-advances">Adv: {nameOrDash(winner)}</span>
        </div>
        <div className="match-team-list">
          <TeamLine name={slot?.team_a} winner={slot?.team_a === winner} />
          <TeamLine name={slot?.team_b} winner={slot?.team_b === winner} />
        </div>
      </div>
    );
  };

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
        <MatchCard key={m} id={m} />
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
        <MatchCard id={104} championMatch />
      </div>
    </div>
  );
}
