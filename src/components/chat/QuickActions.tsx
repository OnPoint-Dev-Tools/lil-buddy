import { useEffect, useState } from 'react';

const QUICK_ACTIONS_KEY = 'lil-buddy-quick-actions-v1';
const DEFAULT_ACTIONS = [
  'Run a safe project health check',
  'Find dead code from old registry/path state',
  'Show the plan before editing files',
];

function loadActions() {
  try {
    const parsed = JSON.parse(localStorage.getItem(QUICK_ACTIONS_KEY) ?? 'null') as unknown;
    if (Array.isArray(parsed)) {
      const items = parsed
        .filter((value): value is string => typeof value === 'string')
        .map((value) => value.trim())
        .filter(Boolean);

      if (items.length > 0) {
        return items;
      }
    }
  } catch {
    // Ignore malformed localStorage and fall back to defaults.
  }

  return DEFAULT_ACTIONS;
}

function saveActions(actions: string[]) {
  localStorage.setItem(QUICK_ACTIONS_KEY, JSON.stringify(actions));
}

export function QuickActions(props: {
  open: boolean;
  onClose: () => void;
  onPick: (prompt: string) => void;
}) {
  const [actions, setActions] = useState<string[]>(() => loadActions());

  useEffect(() => {
    if (!props.open) return;
    setActions(loadActions());
  }, [props.open]);

  if (!props.open) return null;

  function pick(prompt: string) {
    props.onPick(prompt);
    props.onClose();
  }

  function updateAction(index: number, value: string) {
    setActions((current) => current.map((action, actionIndex) => (actionIndex === index ? value : action)));
  }

  function addAction() {
    setActions((current) => [...current, '']);
  }

  function removeAction(index: number) {
    setActions((current) => current.filter((_, actionIndex) => actionIndex !== index));
  }

  function persistActions() {
    const next = actions.map((action) => action.trim()).filter(Boolean);
    const saved = next.length > 0 ? next : DEFAULT_ACTIONS;
    setActions(saved);
    saveActions(saved);
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
            <div className="lm-muted">Pick a prompt, edit it, then save your shortcuts.</div>
          </div>

          <button className="lm-icon-btn" onClick={props.onClose}>×</button>
        </div>

        <div className="lm-quick-list">
          {actions.map((action, index) => (
            <div key={index} className="lm-quick-editor-card">
              <textarea
                className="lm-quick-editor-input"
                value={action}
                rows={2}
                placeholder="Enter a quick action prompt..."
                onChange={(event) => updateAction(index, event.target.value)}
              />
              <div className="lm-quick-editor-actions">
                <button type="button" onClick={() => pick(action.trim())} disabled={!action.trim()}>
                  Use
                </button>
                <button type="button" onClick={() => removeAction(index)}>
                  Remove
                </button>
              </div>
            </div>
          ))}
          <div className="lm-quick-footer-actions">
            <button type="button" className="lm-secondary-btn" onClick={addAction}>Add prompt</button>
            <button type="button" className="lm-primary-btn" onClick={persistActions}>Save prompts</button>
          </div>
        </div>
      </div>
    </div>
  );
}
