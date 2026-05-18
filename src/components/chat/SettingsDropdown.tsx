import { type ReactNode, useEffect, useMemo, useRef, useState } from 'react';
import { TextShimmer } from '../ui/TextShimmer';

export type SettingsDropdownOption = {
  id: string;
  name: string;
  desc?: string;
  badge?: string;
  icon?: ReactNode;
};

export function SettingsDropdown(props: {
  label: string;
  value: string;
  options: SettingsDropdownOption[];
  onChange: (value: string) => void;
  placeholder?: string;
  searchable?: boolean;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function onPointerDown(event: PointerEvent) {
      if (!open) return;
      if (!rootRef.current?.contains(event.target as Node)) {
        setOpen(false);
        setQuery('');
      }
    }
    document.addEventListener('pointerdown', onPointerDown);
    return () => document.removeEventListener('pointerdown', onPointerDown);
  }, [open]);

  const selected = props.options.find((option) => option.id === props.value);
  const optionName = (option: SettingsDropdownOption) =>
    option.name ?? (option as unknown as { label?: string }).label ?? option.id;
  const optionDesc = (option: SettingsDropdownOption) =>
    option.desc ?? (option as unknown as { hint?: string }).hint;
  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();

    if (!q) return props.options;

    return props.options.filter((option) => {
      return (
        optionName(option).toLowerCase().includes(q) ||
        option.id.toLowerCase().includes(q) ||
        optionDesc(option)?.toLowerCase().includes(q)
      );
    });
  }, [props.options, query]);

  function pick(value: string) {
    props.onChange(value);
    setOpen(false);
    setQuery('');
  }

  return (
    <div className="lm-settings-dropdown-wrap" ref={rootRef}>
      <button
        type="button"
        className="lm-settings-dropdown-trigger"
        onClick={() => setOpen(!open)}
      >
        <span className="lm-model-icon">{selected?.icon ?? '◈'}</span>
        <span className="lm-model-main">
          <span className="lm-model-kicker">{props.label}</span>
          <TextShimmer text={selected ? optionName(selected) : props.placeholder ?? 'Select'} />
        </span>
        {selected?.badge ? <span className="lm-model-badge">{selected.badge}</span> : null}
        <span className="lm-model-chevron">⌄</span>
      </button>

      {open ? (
        <div className="lm-settings-dropdown-popover">
          {props.searchable ? (
            <div className="lm-model-search">
              <span>⌕</span>
              <input
                autoFocus
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder={`Search ${props.label.toLowerCase()}...`}
              />
              {query ? <button onClick={() => setQuery('')}>×</button> : null}
            </div>
          ) : null}

          <div className="lm-settings-dropdown-list">
            {filtered.map((option) => (
              <button
                key={option.id}
                className={option.id === props.value ? 'lm-settings-dropdown-row active' : 'lm-settings-dropdown-row'}
                type="button"
                onClick={() => pick(option.id)}
              >
                <span className="lm-provider-row-icon">{option.icon ?? '•'}</span>
                <span className="lm-provider-row-main">
                  <strong>{optionName(option)}</strong>
                  {optionDesc(option) ? <small>{optionDesc(option)}</small> : null}
                </span>
                {option.badge ? <span className="lm-provider-command">{option.badge}</span> : null}
              </button>
            ))}

            {filtered.length === 0 ? (
              <div className="lm-provider-empty">No options found</div>
            ) : null}
          </div>
        </div>
      ) : null}
    </div>
  );
}
