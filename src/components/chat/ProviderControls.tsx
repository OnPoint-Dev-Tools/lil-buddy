import { providers } from '../../lib/providers/registry';

export function ProviderControls(props: {
  selectedProvider: string;
  opencodeGoMode: string;
  onProviderChange: (provider: string) => void;
  onModeChange: (mode: string) => void;
}) {
  const selected = providers.find((provider) => provider.id === props.selectedProvider);

  return (
    <div className="lm-provider-controls">
      <label className="lm-provider-control">
        <span>Provider</span>
        <select
          value={props.selectedProvider}
          onChange={(event) => props.onProviderChange(event.target.value)}
          aria-label="Provider"
        >
          {providers.map((provider) => (
            <option key={provider.id} value={provider.id}>
              {provider.name}
            </option>
          ))}
        </select>
      </label>

      <label className="lm-provider-control compact">
        <span>Mode</span>
        <select
          value={props.opencodeGoMode}
          onChange={(event) => props.onModeChange(event.target.value)}
          aria-label="Provider mode"
        >
          <option value="run-json">JSON</option>
          <option value="run-formatted">Formatted</option>
          <option value="run-stdin">STDIN</option>
          <option value="raw-arg">Raw</option>
        </select>
      </label>

      <div className="lm-provider-note">
        {selected?.native ? 'Native' : 'External'} · {selected?.command}
      </div>
    </div>
  );
}
