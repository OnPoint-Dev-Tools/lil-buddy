import type { StreamLine } from '../../stores/appStore';

function parseStats(text: string) {
  const parts = text.split(' · ');
  return parts.map((part) => {
    const [key, ...rest] = part.split('=');
    return {
      key: rest.length ? key.trim() : 'summary',
      value: rest.length ? rest.join('=').trim() : part.trim(),
    };
  });
}

export function SessionStatsPanel(props: { messages: StreamLine[] }) {
  const latest = [...props.messages].reverse().find((message) => message.kind === 'session-stats');

  return (
    <div className="lm-stats-card">
      <div className="lm-diff-header">
        <strong>Session Stats</strong>
        <span className="lm-muted">{latest ? 'latest step' : 'waiting'}</span>
      </div>

      {!latest ? (
        <div className="lm-muted">Token usage, cost, and finish reason will appear after a step finishes.</div>
      ) : (
        <div className="lm-stats-grid">
          {parseStats(latest.text).map((item, index) => (
            <div key={`${item.key}-${index}`} className="lm-stat-pill">
              <span>{item.key}</span>
              <strong>{item.value}</strong>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
