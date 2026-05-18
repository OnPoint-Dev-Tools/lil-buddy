const ACTIONS = [
  'Run a safe project health check',
  'Find dead code from old registry/path state',
  'Show the plan before editing files',
];

export function QuickActions(props: {
  open: boolean;
  onClose: () => void;
  onPick: (prompt: string) => void;
}) {
  if (!props.open) return null;

  function pick(prompt: string) {
    props.onPick(prompt);
    props.onClose();
  }

  return (
    <div
      className="lm-modal-backdrop quick-actions-backdrop"
      onMouseDown={(event) => event.stopPropagation()}
      onClick={props.onClose}
    >
      <div className="lm-quick-modal" onClick={(event) => event.stopPropagation()}>
        <div className="lm-modal-header compact">
          <div>
            <div className="lm-modal-title">Quick Actions</div>
            <div className="lm-muted">Pick a prompt to start fast.</div>
          </div>

          <button className="lm-icon-btn" onClick={props.onClose}>×</button>
        </div>

        <div className="lm-quick-list">
          {ACTIONS.map((action) => (
            <button key={action} onClick={() => pick(action)}>
              <span>▫</span>
              {action}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
