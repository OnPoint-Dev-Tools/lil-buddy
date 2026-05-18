import type { StreamLine } from '../../stores/appStore';

export function SafetyPanel(props: { messages: StreamLine[] }) {
  const safetyEvents = props.messages.filter(
    (message) => message.kind === 'safety' || message.kind === 'command-approval',
  );

  return (
    <div className="lm-safety-card">
      <div className="lm-diff-header">
        <strong>Safety Watch</strong>
        <span className="lm-muted">{safetyEvents.length} event(s)</span>
      </div>

      {safetyEvents.length === 0 ? (
        <div className="lm-muted">
          No risky command events detected yet. Lil Buddy will flag destructive patterns when they appear.
        </div>
      ) : (
        <div className="lm-safety-list">
          {safetyEvents.slice(-4).map((event, index) => (
            <div key={`${event.ts}-${index}`} className="lm-safety-item">
              <span>{event.kind}</span>
              <strong>{event.text}</strong>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
