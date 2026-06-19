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
      <h2>Scenario Analysis (LLM-powered)</h2>
      <p className="scenario-desc">
        Describe an injury, suspension, or event. The LLM (GLM-5.2) analyzes the
        impact, adjusts team Elo ratings, and re-runs the simulation.
      </p>
      <div className="scenario-input">
        <textarea
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          placeholder="e.g., Spain's star striker is injured for the rest of the tournament…"
          rows={3}
          disabled={disabled}
        />
        <button
          onClick={() => prompt.trim() && onSubmit(prompt)}
          disabled={disabled || !prompt.trim()}
        >
          {disabled ? "Analyzing…" : "Apply & Re-simulate"}
        </button>
      </div>
      <div className="examples">
        <span>Try:</span>
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
    </section>
  );
}
