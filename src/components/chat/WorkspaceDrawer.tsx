import { useState } from 'react';
import type { DesktopEnvironment } from '../../lib/providers/types';
import type { WorkspaceDiff, WorkspaceInfo } from '../../lib/tauri/commands';
import { DesktopStatus } from './DesktopStatus';
import { ChangedFilesPanel } from './ChangedFilesPanel';
import { DiffPanel } from './DiffPanel';
import { SafetyPanel } from './SafetyPanel';
import { TimelinePanel } from './TimelinePanel';
import { ToolCardsPanel } from './ToolCardsPanel';
import { SessionStatsPanel } from './SessionStatsPanel';
import type { StreamLine } from '../../stores/appStore';

export function WorkspaceDrawer(props: {
  desktop: DesktopEnvironment | null;
  workspace: WorkspaceInfo | null;
  workspaceDiff: WorkspaceDiff | null;
  workspacePathDraft: string;
  messages: StreamLine[];
  selectedDiffFile: string | null;
  onWorkspacePathChange: (path: string) => void;
  onSaveWorkspace: () => void;
  onChooseWorkspace: () => void;
  onOpenTerminal: () => void;
  onRefreshWorkspace: () => void;
  onRefreshDiffs: () => void;
  onViewDiff: (path: string) => void;
  onRestore: (path: string) => void;
  onRestoreAll: () => void;
}) {
  const [open, setOpen] = useState(false);
  const changedCount = props.workspaceDiff?.changed_files.length ?? 0;

  return (
    <div className="lm-drawer">
      <button className="lm-drawer-toggle" onClick={() => setOpen(!open)}>
        <span>{open ? 'Hide workspace' : 'Workspace'}</span>
        <strong>{changedCount} changed</strong>
      </button>

      {open ? (
        <div className="lm-drawer-body">
          <DesktopStatus
            desktop={props.desktop}
            workspace={props.workspace}
            workspacePathDraft={props.workspacePathDraft}
            onWorkspacePathChange={props.onWorkspacePathChange}
            onSaveWorkspace={props.onSaveWorkspace}
            onChooseWorkspace={props.onChooseWorkspace}
            onOpenTerminal={props.onOpenTerminal}
            onRefreshWorkspace={props.onRefreshWorkspace}
          />

          <ChangedFilesPanel
            diff={props.workspaceDiff}
            onViewDiff={props.onViewDiff}
            onRestore={props.onRestore}
            onRestoreAll={props.onRestoreAll}
          />

          <DiffPanel
            diff={props.workspaceDiff}
            selectedFile={props.selectedDiffFile}
            onRefresh={props.onRefreshDiffs}
          />

          <SafetyPanel messages={props.messages} />

          <TimelinePanel messages={props.messages} />

          <ToolCardsPanel messages={props.messages} />

          <SessionStatsPanel messages={props.messages} />
        </div>
      ) : null}
    </div>
  );
}
