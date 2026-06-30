import { useState } from "react";

const EXAMPLES = [
  "Lamine Yamal gets injured in Spain's second group match",
  "Mbappe is suspended for the knockout stage",
  "Argentina's entire defense has food poisoning",
  "Vinicius Jr and Rodrygo both injured for Brazil",
  "England's Harry Kane breaks his ankle in training",
];

export function ScenarioPrompt({
  onSubmit,
  disabled,
}: {
  onSubmit: (prompt: string) => void;
  disabled: boolean;
}) {
  const [prompt, setPrompt] = useState("");

  return (
    <section className="section scenario-section">
      <h2>Scenario</h2>
      <div className="scenario-input">
        <textarea
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          placeholder="Injury, suspension, weather, lineup change…"
          rows={2}
          disabled={disabled}
        />
        <button
          onClick={() => prompt.trim() && onSubmit(prompt)}
          disabled={disabled || !prompt.trim()}
        >
          {disabled ? "Analyzing…" : "Apply"}
        </button>
      </div>
      <details className="examples scenario-examples">
        <summary>Examples</summary>
        <div className="example-list">
          {EXAMPLES.map((ex, i) => (
            <button
              key={i}
              className="example-btn"
              onClick={() => setPrompt(ex)}
              disabled={disabled}
            >
              {ex}
            </button>
          ))}
        </div>
      </details>
    </section>
  );
}
