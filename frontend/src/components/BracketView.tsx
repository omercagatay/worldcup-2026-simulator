import type { BracketSlot } from "../api";

const ROUNDS: { title: string; ids: number[] }[] = [
  { title: "Round of 32", ids: [73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88] },
  { title: "Round of 16", ids: [89, 90, 91, 92, 93, 94, 95, 96] },
  { title: "Quarterfinals", ids: [97, 98, 99, 100] },
  { title: "Semifinals", ids: [101, 102] },
  { title: "Final", ids: [104] },
];

const FINAL_ID = 104;

export function BracketView({
  bracket,
  champion,
}: {
  bracket: BracketSlot[];
  champion: string;
}) {
  const matchAt = (id: number) => bracket.find((s) => s.match_id === id);

  const TeamLine = ({ name, won }: { name: string | undefined; won: boolean }) => (
    <div className={`match-line${won ? " won" : ""}`}>{name || "—"}</div>
  );

  return (
    <div>
      <div className="bracket-scroll">
        <div className="bracket">
          {ROUNDS.map((round) => (
            <div key={round.title} className="round">
              <h3 className="round-title">{round.title}</h3>
              <div className="round-matches">
                {round.ids.map((id) => {
                  const slot = matchAt(id);
                  const isFinal = id === FINAL_ID;
                  const winner = slot?.winner || (isFinal ? champion : undefined);
                  const card = (
                    <div className={`match${isFinal ? " match-final" : ""}`} title={`Match ${id}`}>
                      <TeamLine
                        name={slot?.team_a}
                        won={!!slot?.team_a && slot.team_a === winner}
                      />
                      <TeamLine
                        name={slot?.team_b}
                        won={!!slot?.team_b && slot.team_b === winner}
                      />
                    </div>
                  );
                  return (
                    <div key={id} className="match-slot">
                      {isFinal && champion ? (
                        <div className="final-block">
                          {card}
                          <div className="champion-crest">
                            <span className="eyebrow">Simulated champion</span>
                            <strong>{champion}</strong>
                          </div>
                        </div>
                      ) : (
                        card
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      </div>
      <p className="source-note">
        The single simulated bracket that best matches each slot's most common outcome. Matches
        already played show their real result.
      </p>
    </div>
  );
}
