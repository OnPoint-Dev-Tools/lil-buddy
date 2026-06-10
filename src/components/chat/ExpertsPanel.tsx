import { useEffect, useMemo, useRef, useState } from 'react';
import { TextShimmer } from '../ui/TextShimmer';
import tanExplorerIcon from '../../assets/companions/tan-explorer.png';
import pinkHoodIcon from '../../assets/companions/pink-hood.png';
import autumnVestIcon from '../../assets/companions/autumn-vest.png';
import greenScoutIcon from '../../assets/companions/green-scout.png';
import blueHoodieIcon from '../../assets/companions/blue-hoodie.png';
import lavenderBearIcon from '../../assets/companions/lavender-bear.png';
import yellowRainIcon from '../../assets/companions/yellow-rain.png';
import redBeanieIcon from '../../assets/companions/red-beanie.png';

export type LilExpert = {
  id: string;
  icon: string;
  name: string;
  role: string;
  systemPrompt: string;
};

const STORAGE_KEY = 'lil-buddy-experts-v4';
const SELECTED_KEY = 'lil-buddy-selected-expert-v4';

const LEGACY_STORAGE_KEYS = ['lil-buddy-experts-v3', 'lil-buddy-experts-v2', 'lil-buddy-experts-v1'];
const LEGACY_SELECTED_KEYS = ['lil-buddy-selected-expert-v3', 'lil-buddy-selected-expert-v2', 'lil-buddy-selected-expert-v1'];

const EXPERT_ICON_OPTIONS = [
  { id: 'tan-explorer', name: 'Tan Explorer', src: tanExplorerIcon },
  { id: 'pink-hood', name: 'Pink Hood', src: pinkHoodIcon },
  { id: 'autumn-vest', name: 'Autumn Vest', src: autumnVestIcon },
  { id: 'green-scout', name: 'Green Scout', src: greenScoutIcon },
  { id: 'blue-hoodie', name: 'Blue Hoodie', src: blueHoodieIcon },
  { id: 'lavender-bear', name: 'Lavender Bear', src: lavenderBearIcon },
  { id: 'yellow-rain', name: 'Yellow Rain', src: yellowRainIcon },
  { id: 'red-beanie', name: 'Red Beanie', src: redBeanieIcon },
];

export function expertIconSrc(icon: string) {
  return EXPERT_ICON_OPTIONS.find((option) => option.id === icon)?.src ?? tanExplorerIcon;
}

export function expertIconName(icon: string) {
  return EXPERT_ICON_OPTIONS.find((option) => option.id === icon)?.name ?? 'Tan Explorer';
}

export function normalizeExpertIcon(icon: string) {
  if (EXPERT_ICON_OPTIONS.some((option) => option.id === icon)) return icon;

  const emojiMap: Record<string, string> = {
    '🏗️': 'tan-explorer',
    '🪲': 'green-scout',
    '✨': 'pink-hood',
    '⭐': 'blue-hoodie',
  };

  return emojiMap[icon] ?? 'tan-explorer';
}

const defaultExperts: LilExpert[] = [
  {
    id: 'repo-architect',
    icon: 'tan-explorer',
    name: 'Repo Architect',
    role: 'Plans structure and safe refactors',
    systemPrompt:
      'You are Repo Architect. Focus on architecture, clear plans, dependency boundaries, refactors, and low-risk implementation steps. Be concise and show the files/commands you inspect.',
  },
  {
    id: 'bug-hunter',
    icon: 'green-scout',
    name: 'Bug Hunter',
    role: 'Finds runtime errors and broken flows',
    systemPrompt:
      'You are Bug Hunter. Focus on reproducing bugs, reading stack traces carefully, tracing root causes, and proposing minimal patches. Prefer evidence from files and commands.',
  },
  {
    id: 'ui-polish',
    icon: 'pink-hood',
    name: 'UI Polish',
    role: 'Improves UX, layout, and visual feel',
    systemPrompt:
      'You are UI Polish. Focus on interaction details, spacing, responsive behavior, readable UI states, and calm visual polish without overcomplicating the app.',
  },
];

function migrateExpert(value: LilExpert & { workspacePath?: string | null }): LilExpert {
  return {
    id: value.id,
    icon: normalizeExpertIcon(value.icon),
    name: value.name,
    role: value.role,
    systemPrompt: value.systemPrompt,
  };
}

function newExpert(): LilExpert {
  return {
    id: `expert-${Date.now()}`,
    icon: 'blue-hoodie',
    name: 'New Lil Expert',
    role: 'Custom specialist',
    systemPrompt: 'You are a helpful Lil Buddy expert. Focus on this specialty and keep the user updated while working.',
  };
}

export function loadExperts() {
  try {
    const raw =
      localStorage.getItem(STORAGE_KEY) ??
      LEGACY_STORAGE_KEYS.map((key) => localStorage.getItem(key)).find(Boolean) ??
      null;

    const parsed = raw ? JSON.parse(raw) : null;
    if (Array.isArray(parsed) && parsed.length > 0) {
      const migrated = parsed.map(migrateExpert);
      localStorage.setItem(STORAGE_KEY, JSON.stringify(migrated));
      return migrated;
    }
  } catch {
    // ignore malformed localStorage
  }

  localStorage.setItem(STORAGE_KEY, JSON.stringify(defaultExperts));
  return defaultExperts;
}

export function selectedExpertId() {
  return (
    localStorage.getItem(SELECTED_KEY) ??
    LEGACY_SELECTED_KEYS.map((key) => localStorage.getItem(key)).find(Boolean) ??
    null
  );
}

export function selectedExpert() {
  const id = selectedExpertId();
  if (!id) return null;
  return loadExperts().find((expert) => expert.id === id) ?? null;
}

export function expertGreeting(expert: LilExpert) {
  return `I'm ${expert.name}. What would you like to dig into?`;
}

export function expertPromptPrefix(expert?: LilExpert | null) {
  if (!expert) return '';

  return `Lil Buddy expert profile active:
Name: ${expert.name}
Role: ${expert.role}
System prompt:
${expert.systemPrompt}

Use this expert profile as the active working style for this request.

User request:
`;
}

export function ExpertsPanel(props: {
  selectedId: string | null;
  onSelect: (expert: LilExpert | null) => void;
}) {
  const [open, setOpen] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [experts, setExperts] = useState<LilExpert[]>(() => loadExperts());
  const [editingId, setEditingId] = useState<string | null>(experts[0]?.id ?? null);
  const rootRef = useRef<HTMLDivElement>(null);

  const active = useMemo(
    () => experts.find((expert) => expert.id === props.selectedId) ?? null,
    [experts, props.selectedId],
  );

  const editing = useMemo(
    () => experts.find((expert) => expert.id === editingId) ?? experts[0] ?? null,
    [experts, editingId],
  );

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return experts;

    return experts.filter(
      (expert) =>
        expert.name.toLowerCase().includes(q) ||
        expert.role.toLowerCase().includes(q) ||
        expert.systemPrompt.toLowerCase().includes(q),
    );
  }, [experts, query]);

  useEffect(() => {
    function onPointerDown(event: PointerEvent) {
      if (!open) return;
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false);
    }

    document.addEventListener('pointerdown', onPointerDown);
    return () => document.removeEventListener('pointerdown', onPointerDown);
  }, [open]);

  function persist(nextExperts: LilExpert[]) {
    setExperts(nextExperts);
    localStorage.setItem(STORAGE_KEY, JSON.stringify(nextExperts));
  }

  function choose(expert: LilExpert | null) {
    if (expert) {
      localStorage.setItem(SELECTED_KEY, expert.id);
    } else {
      localStorage.removeItem(SELECTED_KEY);
    }

    props.onSelect(expert);
    setOpen(false);
  }

  function addExpert() {
    const expert = newExpert();
    const next = [...experts, expert];
    persist(next);
    setEditingId(expert.id);
    setModalOpen(true);
    setOpen(false);
  }

  function updateEditing(patch: Partial<LilExpert>) {
    if (!editing) return;

    const next = experts.map((expert) =>
      expert.id === editing.id ? { ...expert, ...patch } : expert,
    );
    persist(next);
  }

  function removeEditing() {
    if (!editing) return;

    const next = experts.filter((expert) => expert.id !== editing.id);
    persist(next);
    if (props.selectedId === editing.id) choose(null);
    setEditingId(next[0]?.id ?? null);
  }

  const title = active?.name ?? 'Lil Buddy';
  const subtitle = active ? `expert · ${active.role}` : 'expert · default';

  return (
    <div className="lm-title-switcher lm-experts-title" ref={rootRef} data-tauri-drag-region="false">
      <button className="lm-title-trigger" onClick={() => setOpen(!open)} data-tauri-drag-region="false">
        <span className="lm-title-icon lm-expert-face-icon">
          {active ? <img src={expertIconSrc(active.icon)} alt="" /> : 'LM'}
        </span>
        <span className="lm-title-main"><TextShimmer text={title} /></span>
        <span className={open ? 'lm-title-caret open' : 'lm-title-caret'}>⌃</span>
      </button>

      <div className="lm-muted">{subtitle}</div>

      {open ? (
        <div className="lm-title-popover lm-experts-title-popover" data-tauri-drag-region="false">
          <div className="lm-model-search">
            <span>⌕</span>
            <input
              autoFocus
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Search experts..."
            />
            {query ? <button onClick={() => setQuery('')}>×</button> : null}
          </div>

          <div className="lm-workspace-title-actions">
            <button className="lm-workspace-add-row" onClick={addExpert}>
              <span>＋</span>
              <strong>Add Lil expert</strong>
              <small>Create a focused agent</small>
            </button>

            <button className="lm-workspace-save-row" onClick={() => { setModalOpen(true); setOpen(false); }}>
              <span>✎</span>
              <strong>Manage experts</strong>
              <small>Edit icon, role, and prompt</small>
            </button>
          </div>

          <div className="lm-workspace-switch-list">
            <button className={!active ? 'lm-workspace-switch-row active' : 'lm-workspace-switch-row'} onClick={() => choose(null)}>
              <span className="lm-provider-row-icon">LM</span>
              <span className="lm-provider-row-main">
                <strong>Default Lil Buddy</strong>
                <small>No custom expert profile</small>
              </span>
            </button>

            {filtered.map((expert) => (
              <button
                key={expert.id}
                className={active?.id === expert.id ? 'lm-workspace-switch-row active' : 'lm-workspace-switch-row'}
                onClick={() => choose(expert)}
              >
                <span className="lm-provider-row-icon lm-expert-face-icon"><img src={expertIconSrc(expert.icon)} alt="" /></span>
                <span className="lm-provider-row-main">
                  <strong>{expert.name}</strong>
                  <small>{expert.role}</small>
                </span>
              </button>
            ))}

            {filtered.length === 0 ? (
              <div className="lm-provider-empty">No matching experts</div>
            ) : null}
          </div>
        </div>
      ) : null}

      {modalOpen ? (
        <div className="lm-modal-overlay-solid" data-tauri-drag-region="false" onClick={() => setModalOpen(false)}>
          <div className="lm-modal-card lm-experts-modal" data-tauri-drag-region="false" onClick={(event) => event.stopPropagation()}>
            <div className="lm-modal-header">
              <div>
                <h3>Lil Buddy Experts</h3>
                <small>Create focused agents with their own icon, role, and system prompt.</small>
              </div>
              <button className="lm-icon-btn" onClick={() => setModalOpen(false)}>×</button>
            </div>

            {experts.length === 0 ? (
              <div className="lm-provider-empty">
                No Lil Buddy experts yet.
                <button className="lm-primary-btn" onClick={addExpert}>Add Lil expert</button>
              </div>
            ) : (
              <div className="lm-experts-editor">
                <aside className="lm-experts-list">
                  {experts.map((expert) => (
                    <button
                      key={expert.id}
                      className={editing?.id === expert.id ? 'active' : ''}
                      onClick={() => setEditingId(expert.id)}
                    >
                      <span className="lm-expert-face-icon"><img src={expertIconSrc(expert.icon)} alt="" /></span>
                      <strong>{expert.name}</strong>
                      <small>{expert.role}</small>
                    </button>
                  ))}

                  <button className="add" onClick={addExpert}>＋ Add Lil expert</button>
                </aside>

                {editing ? (
                  <section className="lm-expert-form">
                    <label>
                      <span>Companion face icon</span>
                      <select
                        value={normalizeExpertIcon(editing.icon)}
                        onChange={(event) => updateEditing({ icon: event.target.value })}
                      >
                        {EXPERT_ICON_OPTIONS.map((option) => (
                          <option key={option.id} value={option.id}>{option.name}</option>
                        ))}
                      </select>
                    </label>

                    <div className="lm-expert-icon-preview">
                      <img src={expertIconSrc(editing.icon)} alt="" />
                      <span>{expertIconName(editing.icon)}</span>
                    </div>
                    <label>
                      <span>Name</span>
                      <input value={editing.name} onChange={(event) => updateEditing({ name: event.target.value })} />
                    </label>
                    <label>
                      <span>Role</span>
                      <input value={editing.role} onChange={(event) => updateEditing({ role: event.target.value })} />
                    </label>

                    <label>
                      <span>System prompt</span>
                      <textarea
                        value={editing.systemPrompt}
                        onChange={(event) => updateEditing({ systemPrompt: event.target.value })}
                        rows={8}
                      />
                    </label>

                    <div className="lm-modal-footer">
                      <button className="lm-secondary-btn" onClick={removeEditing}>Delete</button>
                      <button className="lm-primary-btn" onClick={() => { choose(editing); setModalOpen(false); }}>Use this expert</button>
                    </div>
                  </section>
                ) : null}
              </div>
            )}
          </div>
        </div>
      ) : null}
    </div>
  );
}
