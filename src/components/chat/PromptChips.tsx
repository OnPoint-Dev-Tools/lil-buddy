const chips = [
  'Run a safe project health check',
  'Find dead code from old registry/path state',
  'Show the plan before editing files',
];

export function PromptChips(props: { onPick: (value: string) => void }) {
  return (
    <div className="lm-chip-grid">
      {chips.map((chip) => (
        <button key={chip} className="lm-chip" onClick={() => props.onPick(chip)}>
          <span className="lm-chip-icon">▣</span>
          <span>{chip}</span>
        </button>
      ))}
    </div>
  );
}
