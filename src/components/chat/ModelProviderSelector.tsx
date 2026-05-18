import { useEffect, useMemo, useRef, useState } from 'react';
import {
  listProviderModels,
  loadSettings,
  saveFavoriteModels,
  saveSelectedModel,
  type ProviderModel,
} from '../../lib/tauri/commands';
import { providers } from '../../lib/providers/registry';
import { TextShimmer } from '../ui/TextShimmer';
import { ProviderIcon } from './ProviderIcon';
import {
  Brain,
  Code2,
  GitBranch,
  GitCompare,
  Search,
  Sparkles,
  Wrench,
} from 'lucide-react';

type RuntimeMode = {
  id: string;
  name: string;
  desc: string;
};

const MODES: RuntimeMode[] = [
  {
    id: 'run-json',
    name: 'JSON',
    desc: 'Best default. Streams structured OpenCode events so Lil Buddy can show chat, tools, stats, and diffs cleanly.',
  },
];

function providerName(providerId: string) {
  const appProvider = providers.find((provider) => provider.id === providerId);

  if (appProvider) return appProvider.name;

  return providerId
    .replace(/-/g, ' ')
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function capLabel(cap: string) {
  const iconProps = {
    size: 12,
    strokeWidth: 2.4,
    'aria-hidden': true,
  };

  if (cap === 'tools') return <Wrench {...iconProps} />;
  if (cap === 'repo') return <GitBranch {...iconProps} />;
  if (cap === 'diffs') return <GitCompare {...iconProps} />;
  if (cap === 'search') return <Search {...iconProps} />;
  if (cap === 'reasoning') return <Brain {...iconProps} />;
  if (cap === 'code') return <Code2 {...iconProps} />;

  return <Sparkles {...iconProps} />;
}

export function ModelProviderSelector(props: {
  selectedProvider: string;
  opencodeGoMode: string;
  onProviderChange: (provider: string) => void;
  onModeChange: (mode: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [models, setModels] = useState<ProviderModel[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedModel, setSelectedModel] = useState<ProviderModel | null>(null);
  const [favoriteIds, setFavoriteIds] = useState<Set<string>>(new Set());
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function onPointerDown(event: PointerEvent) {
      if (!open) return;
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    document.addEventListener('pointerdown', onPointerDown);
    return () => document.removeEventListener('pointerdown', onPointerDown);
  }, [open]);

  const selectedRuntimeProvider =
    providers.find((provider) => provider.id === props.selectedProvider) ?? providers[0];

  async function loadModels(refresh = false) {
    setLoading(true);

    try {
      const [next, settings] = await Promise.all([
        listProviderModels(props.selectedProvider, refresh),
        loadSettings(),
      ]);

      setModels(next);
      setFavoriteIds(new Set(next.filter((model) => model.starred).map((model) => model.id)));

      const saved = settings.selected_model
        ? next.find((model) => model.command_model === settings.selected_model || model.id === settings.selected_model)
        : null;

      setSelectedModel(saved ?? null);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    loadModels(false).catch(() => {
      setModels([]);
      setSelectedModel(null);
    });
  }, [props.selectedProvider]);


  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();

    return models.filter((model) => {
      const matchesQuery =
        !q ||
        model.name.toLowerCase().includes(q) ||
        model.description.toLowerCase().includes(q) ||
        model.command_model.toLowerCase().includes(q);

      return matchesQuery;
    });
  }, [models, query]);

  const favoriteModels = useMemo(
    () => models.filter((model) => favoriteIds.has(model.id)),
    [models, favoriteIds],
  );

  async function pickModel(model: ProviderModel) {
    setSelectedModel(model);
    await saveSelectedModel(model.command_model);
    setOpen(false);
  }

  async function useProviderDefault() {
    setSelectedModel(null);
    await saveSelectedModel(null);
    setOpen(false);
  }

  async function toggleFavorite(model: ProviderModel) {
    const next = new Set(favoriteIds);

    if (next.has(model.id)) {
      next.delete(model.id);
    } else {
      next.add(model.id);
    }

    setFavoriteIds(next);
    await saveFavoriteModels(Array.from(next));
  }


  const triggerLabel = selectedModel?.name ?? `${selectedRuntimeProvider.name} Default`;
  const triggerProviderId = selectedModel?.provider_id ?? selectedRuntimeProvider.id;

  return (
    <div className="lm-model-selector-row" ref={rootRef}>
      <div className="lm-model-select-wrap">
        <button
          className="lm-model-trigger"
          onClick={() => {
            setOpen(!open);
          }}
        >
          <ProviderIcon providerId={triggerProviderId} label={triggerLabel} className="model-trigger-icon" />
          <span className="lm-model-main">
            <span className="lm-model-kicker">Model</span>
            <TextShimmer text={triggerLabel} />
          </span>
          <span className="lm-model-badge">{selectedModel ? providerName(selectedModel.provider_id) : 'Auto'}</span>
          <span className="lm-model-chevron">⌄</span>
        </button>

        {open ? (
          <div className="lm-model-picker-panel">
            <div className="lm-model-search">
              <span>⌕</span>
              <input
                autoFocus
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="Search models..."
              />
              {query ? <button onClick={() => setQuery('')}>×</button> : null}
            </div>

            <div className="lm-model-picker-body no-sidebar">


              <section className="lm-model-list">
                <div className="lm-model-section">
                  <div className="lm-model-section-title">Runtime Default</div>
                  <button className={!selectedModel ? 'lm-model-row active' : 'lm-model-row'} onClick={useProviderDefault}>
                    <ProviderIcon providerId={selectedRuntimeProvider.id} label={selectedRuntimeProvider.name} className="row-icon" />
                    <span className="lm-model-row-main">
                      <span>
                        <strong>{selectedRuntimeProvider.name} Default</strong>
                        <i>AUTO</i>
                      </span>
                      <small>Use the default model selected by the {selectedRuntimeProvider.name} provider.</small>
                    </span>
                    <span title="Favorite a model" className="lm-model-star placeholder">★</span>
                  </button>
                </div>

                {favoriteModels.length > 0 && !query ? (
                  <div className="lm-model-section">
                    <div className="lm-model-section-title">Favorites</div>
                    {favoriteModels.map((model) => (
                      <ModelRow
                        key={`fav-${model.id}`}
                        model={model}
                        active={selectedModel?.id === model.id}
                        favorite={favoriteIds.has(model.id)}
                        onPick={() => pickModel(model)}
                        onToggleFavorite={() => toggleFavorite(model)}
                      />
                    ))}
                  </div>
                ) : null}

                <div className="lm-model-section">
                  <div className="lm-model-section-title">
                    {loading ? 'Loading models...' : 'Models'}
                    <button onClick={() => loadModels(true)}>Refresh</button>
                  </div>

                  {filtered.map((model) => (
                    <ModelRow
                      key={model.id}
                      model={model}
                      active={selectedModel?.id === model.id}
                      favorite={favoriteIds.has(model.id)}
                      onPick={() => pickModel(model)}
                      onToggleFavorite={() => toggleFavorite(model)}
                    />
                  ))}

                  {!loading && filtered.length === 0 ? (
                    <div className="lm-provider-empty">No models found</div>
                  ) : null}
                </div>
              </section>
            </div>

            <div className="lm-model-footer">
              <span>{selectedModel?.command_model ?? `${selectedRuntimeProvider.name} provider default`}</span>
              <span>{selectedModel ? 'Pinned model' : 'No --model flag will be sent'}</span>
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
}

function ModelRow(props: {
  model: ProviderModel;
  active: boolean;
  favorite: boolean;
  onPick: () => void;
  onToggleFavorite: () => void;
}) {
  return (
    <button className={props.active ? 'lm-model-row active' : 'lm-model-row'} onClick={props.onPick}>
      <ProviderIcon providerId={props.model.provider_id} label={props.model.provider_id} className="row-icon" />

      <span className="lm-model-row-main">
        <span>
          <strong>{props.model.name}</strong>
          {props.model.source === 'opencode models' ? <i>LIVE</i> : null}
        </span>
        <small>{props.model.description}</small>
      </span>

      <span className="lm-model-row-caps">
        {props.model.caps.map((cap) => (
          <i key={cap} title={cap}>{capLabel(cap)}</i>
        ))}
      </span>

      <span
        role="button"
        tabIndex={0}
        className={props.favorite ? 'lm-model-star active' : 'lm-model-star'}
        onClick={(event) => {
          event.stopPropagation();
          props.onToggleFavorite();
        }}
      >
        ★
      </span>
    </button>
  );
}
