export type RestoreTarget =
  | { kind: 'file'; path: string }
  | { kind: 'all' }
  | null;

export function RestoreConfirm(props: {
  target: RestoreTarget;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  if (!props.target) return null;

  const title = props.target.kind === 'all' ? 'Restore all changed files?' : 'Restore this file?';
  const description =
    props.target.kind === 'all'
      ? 'Lil Buddy will ask Git to restore all changed files in this workspace.'
      : `Lil Buddy will ask Git to restore: ${props.target.path}`;

  return (
    <div className="lm-palette-backdrop" onClick={props.onCancel}>
      <div className="lm-confirm" onClick={(event) => event.stopPropagation()}>
        <div>
          <div className="lm-palette-title">{title}</div>
          <div className="lm-muted">{description}</div>
        </div>

        <div className="lm-warning-box">
          This discards uncommitted changes for the selected target.
        </div>

        <div className="lm-confirm-actions">
          <button className="lm-ghost" onClick={props.onCancel}>Cancel</button>
          <button className="lm-danger-run" onClick={props.onConfirm}>Restore</button>
        </div>
      </div>
    </div>
  );
}
