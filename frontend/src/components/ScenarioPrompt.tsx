import { useState } from "react";

const EXAMPLES = [
  "Lamine Yamal gets injured in training",
  "Mbappé is suspended for the semifinal",
  "Argentina's entire defense has food poisoning",
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
    <section className="panel scenario-panel" aria-label="What-if scenario">
      <header className="panel-head">
        <h3>What if…</h3>
      </header>
      <div className="panel-body">
        <textarea
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          placeholder="Describe an injury, suspension, or upset — the model adjusts team ratings and re-simulates."
          rows={3}
          disabled={disabled}
        />
        <div className="scenario-actions">
          <button
            type="button"
            className="btn btn-primary"
            onClick={() => prompt.trim() && onSubmit(prompt)}
            disabled={disabled || !prompt.trim()}
          >
            {disabled ? "Analyzing…" : "Run scenario"}
          </button>
        </div>
        <div className="chips">
          {EXAMPLES.map((ex) => (
            <button
              key={ex}
              type="button"
              className="chip"
              onClick={() => setPrompt(ex)}
              disabled={disabled}
            >
              {ex}
            </button>
          ))}
        </div>
      </div>
    </section>
  );
}
