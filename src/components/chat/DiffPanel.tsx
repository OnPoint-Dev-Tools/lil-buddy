import type { WorkspaceDiff } from '../../lib/tauri/commands';

export function DiffPanel(props: {
  diff: WorkspaceDiff | null;
  selectedFile?: string | null;
  onRefresh: () => void;
}) {
  if (!props.diff) {
    return (
      <div className="lm-diff-card">
        <div className="lm-diff-header">
          <strong>Diff Watch</strong>
          <button className="lm-link-btn" onClick={props.onRefresh}>Refresh</button>
        </div>
        <div className="lm-muted">No diff captured yet.</div>
      </div>
    );
  }

  const visibleDiffs = props.selectedFile
    ? props.diff.diffs.filter((entry) => entry.path === props.selectedFile)
    : props.diff.diffs.slice(0, 2);

  return (
    <div className="lm-diff-card">
      <div className="lm-diff-header">
        <strong>Diff Watch</strong>
        <button className="lm-link-btn" onClick={props.onRefresh}>Refresh</button>
      </div>

      <div className="lm-diff-meta">
        {props.diff.changed_files.length} changed file(s)
      </div>

      {props.diff.diff_stat ? (
        <pre className="lm-diff-stat">{props.diff.diff_stat}</pre>
      ) : (
        <div className="lm-muted">No diff stat available.</div>
      )}

      {props.diff.changed_files.length > 0 ? (
        <div className="lm-file-list">
          {props.diff.changed_files.map((file) => (
            <span key={file}>{file}</span>
          ))}
        </div>
      ) : null}

      {visibleDiffs.map((entry) => (
        <details key={entry.path} className="lm-diff-details">
          <summary>{entry.path}</summary>
          <pre>{entry.diff || 'No textual diff available.'}</pre>
        </details>
      ))}
    </div>
  );
}
