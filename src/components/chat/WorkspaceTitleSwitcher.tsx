import { useEffect, useMemo, useRef, useState } from 'react';
import {
  chooseWorkspaceFolder,
  listWorkspaceSessions,
  saveWorkspaceSession,
  type WorkspaceSession,
} from '../../lib/tauri/commands';
import type { StreamLine } from '../../stores/appStore';
import { TextShimmer } from '../ui/TextShimmer';

function workspaceNameFromPath(path?: string | null) {
  if (!path) return 'Lil Buddy';
  const parts = path.split('/').filter(Boolean);
  return parts.at(-1) || 'Workspace';
}

function coerceMessages(value: unknown): StreamLine[] {
  if (Array.isArray(value)) return value as StreamLine[];
  return [];
}

export function WorkspaceTitleSwitcher(props: {
  workspacePath: string | null | undefined;
  messages: StreamLine[];
  onSwitchWorkspace: (path: string, messages: StreamLine[]) => void;
  onSaveCurrent: (name: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [workspaces, setWorkspaces] = useState<WorkspaceSession[]>([]);
  const rootRef = useRef<HTMLDivElement>(null);

  async function refreshSessions() {
    const sessions = await listWorkspaceSessions();
    setWorkspaces(sessions);
  }

  useEffect(() => {
    refreshSessions().catch(() => {});
  }, []);

  useEffect(() => {
    function onPointerDown(event: PointerEvent) {
      if (!open) return;
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    }

    document.addEventListener('pointerdown', onPointerDown);
    return () => document.removeEventListener('pointerdown', onPointerDown);
  }, [open]);

  const currentName = workspaceNameFromPath(props.workspacePath);
  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return workspaces;

    return workspaces.filter(
      (workspace) =>
        workspace.name.toLowerCase().includes(q) ||
        workspace.path.toLowerCase().includes(q),
    );
  }, [workspaces, query]);

  async function addWorkspace() {
    const selected = await chooseWorkspaceFolder();
    if (!selected) return;

    const name = workspaceNameFromPath(selected);
    const session = await saveWorkspaceSession(name, selected, []);
    await refreshSessions();
    props.onSwitchWorkspace(session.path, []);
    setOpen(false);
  }

  async function rememberCurrent() {
    if (!props.workspacePath) {
      await addWorkspace();
      return;
    }

    const name = prompt('Name this workspace', currentName)?.trim();
    if (!name) return;

    await saveWorkspaceSession(name, props.workspacePath, props.messages);
    await refreshSessions();
    props.onSaveCurrent(name);
  }

  async function switchWorkspace(workspace: WorkspaceSession) {
    if (props.workspacePath) {
      await saveWorkspaceSession(
        workspaceNameFromPath(props.workspacePath),
        props.workspacePath,
        props.messages,
      );
    }

    props.onSwitchWorkspace(workspace.path, coerceMessages(workspace.messages));
    setOpen(false);
  }

  return (
    <div className="lm-title-switcher" ref={rootRef}>
      <button className="lm-title-trigger" onClick={() => setOpen(!open)}>
        <span className="lm-title-main"><TextShimmer text={currentName} /></span>
        <span className={open ? 'lm-title-caret open' : 'lm-title-caret'}>⌃</span>
      </button>

      <div className="lm-muted">workspace · {props.workspacePath ? 'active' : 'none selected'}</div>

      {open ? (
        <div className="lm-title-popover">
          <div className="lm-model-search">
            <span>⌕</span>
            <input
              autoFocus
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search workspaces..."
            />
            {query ? <button onClick={() => setQuery('')}>×</button> : null}
          </div>

          <div className="lm-workspace-title-actions">
            <button className="lm-workspace-add-row" onClick={addWorkspace}>
              <span>＋</span>
              <strong>Add workspace</strong>
              <small>Choose a folder</small>
            </button>

            <button className="lm-workspace-save-row" onClick={rememberCurrent}>
              <span>✓</span>
              <strong>Save current session</strong>
              <small>{props.workspacePath || 'Choose a folder first'}</small>
            </button>
          </div>

          <div className="lm-workspace-switch-list">
            {filtered.map((workspace) => (
              <button key={workspace.id} className="lm-workspace-switch-row" onClick={() => switchWorkspace(workspace)}>
                <span className="lm-provider-row-icon">⌂</span>
                <span className="lm-provider-row-main">
                  <strong>{workspace.name}</strong>
                  <small>{workspace.path}</small>
                </span>
                <span className="lm-provider-command">{coerceMessages(workspace.messages).length} items</span>
              </button>
            ))}

            {filtered.length === 0 ? (
              <div className="lm-provider-empty">No saved workspaces yet</div>
            ) : null}
          </div>
        </div>
      ) : null}
    </div>
  );
}
