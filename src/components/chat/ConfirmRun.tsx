export function ConfirmRun(props: {
  open: boolean;
  provider: string;
  commandPreview: string;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  if (!props.open) return null;

  return (
    <div className="lm-palette-backdrop" onClick={props.onCancel}>
      <div className="lm-confirm" onClick={(event) => event.stopPropagation()}>
        <div>
          <div className="lm-palette-title">Run with {props.provider}?</div>
          <div className="lm-muted">
            Lil Buddy will snapshot the workspace before running, then capture diffs after completion.
          </div>
        </div>

        <div className="lm-command-preview">
          {props.commandPreview || 'No preview available.'}
        </div>

        <div className="lm-confirm-actions">
          <button className="lm-ghost" onClick={props.onCancel}>Cancel</button>
          <button className="lm-run" onClick={props.onConfirm}>Run</button>
        </div>
      </div>
    </div>
  );
}
