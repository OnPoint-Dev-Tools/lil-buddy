import type { StreamLine } from '../../stores/appStore';

const timelineKinds = new Set([
  'timeline',
  'provider',
  'command-preview',
  'tool-call',
  'shell-command',
  'file-read',
  'file-write',
  'file-change',
  'diff',
  'assistant',
  'reasoning',
  'session-stats',
  'exit',
]);

function labelFor(kind: string) {
  if (kind === 'provider') return 'Provider';
  if (kind === 'command-preview') return 'Command';
  if (kind === 'tool-call') return 'Tool';
  if (kind === 'shell-command') return 'Shell';
  if (kind === 'file-read') return 'Read';
  if (kind === 'file-write') return 'Write';
  if (kind === 'file-change') return 'Files';
  if (kind === 'assistant') return 'Assistant';
  if (kind === 'reasoning') return 'Thinking';
  if (kind === 'session-stats') return 'Stats';
  if (kind === 'diff') return 'Diff';
  if (kind === 'exit') return 'Done';
  return 'Step';
}

export function TimelinePanel(props: { messages: StreamLine[] }) {
  const items = props.messages.filter((message) => timelineKinds.has(message.kind)).slice(-10);

  return (
    <div className="lm-timeline-card">
      <div className="lm-diff-header">
        <strong>Timeline</strong>
        <span className="lm-muted">{items.length} step(s)</span>
      </div>

      {items.length === 0 ? (
        <div className="lm-muted">Timeline events will appear here during a run.</div>
      ) : (
        <div className="lm-timeline-list">
          {items.map((item, index) => (
            <div key={`${item.ts}-${index}`} className={`lm-timeline-item event-${item.kind}`}>
              <div className="lm-timeline-dot" />
              <div>
                <span>{labelFor(item.kind)}</span>
                <p>{item.text}</p>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
