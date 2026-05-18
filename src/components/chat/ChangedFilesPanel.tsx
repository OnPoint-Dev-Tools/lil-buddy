import type { WorkspaceDiff } from '../../lib/tauri/commands';

export function ChangedFilesPanel(props: {
  diff: WorkspaceDiff | null;
  onViewDiff: (path: string) => void;
  onRestore: (path: string) => void;
  onRestoreAll: () => void;
}) {
  if (!props.diff || props.diff.changed_files.length === 0) {
    return (
      <div className="lm-changed-card">
        <strong>Changed Files</strong>
        <div className="lm-muted">No changed files detected.</div>
      </div>
    );
  }

  return (
    <div className="lm-changed-card">
      <div className="lm-diff-header">
        <strong>Changed Files</strong>
        <button className="lm-link-btn danger-link" onClick={props.onRestoreAll}>
          Restore All
        </button>
      </div>

      <div className="lm-changed-list">
        {props.diff.changed_files.map((file) => (
          <div key={file} className="lm-changed-row">
            <span>{file}</span>
            <div>
              <button onClick={() => props.onViewDiff(file)}>View Diff</button>
              <button className="danger-link" onClick={() => props.onRestore(file)}>Restore</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
