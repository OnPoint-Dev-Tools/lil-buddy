import type { DesktopEnvironment } from '../../lib/providers/types';
import type { WorkspaceInfo } from '../../lib/tauri/commands';

export function DesktopStatus(props: {
  desktop: DesktopEnvironment | null;
  workspace: WorkspaceInfo | null;
  workspacePathDraft: string;
  onWorkspacePathChange: (path: string) => void;
  onSaveWorkspace: () => void;
  onChooseWorkspace: () => void;
  onOpenTerminal: () => void;
  onRefreshWorkspace: () => void;
}) {
  const desktopName = props.desktop?.is_hyprland
    ? 'Hyprland'
    : props.desktop?.desktop || props.desktop?.session_type || 'unknown';

  const branch = props.workspace?.branch || 'no git branch';
  const dirty = props.workspace?.dirty ? 'dirty' : 'clean';

  return (
    <div className="lm-workspace-panel">
      <div className="lm-desktop-grid">
        <div className="lm-small-card">
          <span>Desktop</span>
          <strong>{desktopName}</strong>
          <small>{props.desktop?.session_type || 'session unknown'}</small>
        </div>

        <button className="lm-small-card clickable" onClick={props.onRefreshWorkspace}>
          <span>Workspace</span>
          <strong>{branch}</strong>
          <small>{props.workspace?.is_git_repo ? dirty : 'not a git repo'} · refresh</small>
        </button>
      </div>

      <div className="lm-workspace-path">
        <span>{props.workspace?.path || 'No workspace selected'}</span>
      </div>

      <div className="lm-workspace-input-row">
        <input
          value={props.workspacePathDraft}
          onChange={(event) => props.onWorkspacePathChange(event.target.value)}
          placeholder="Workspace path, e.g. /home/me/projects/lil-buddy"
        />
        <button className="workspace-btn" onClick={props.onSaveWorkspace}>Save</button>
      </div>

      <div className="lm-workspace-actions">
        <button onClick={props.onChooseWorkspace}>Choose Folder</button>
        <button onClick={props.onOpenTerminal}>Open Terminal</button>
        <button onClick={props.onRefreshWorkspace}>Refresh</button>
      </div>
    </div>
  );
}
