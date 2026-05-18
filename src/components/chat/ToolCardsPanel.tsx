import type { StreamLine } from '../../stores/appStore';

const toolKinds = new Set([
  'tool-call',
  'shell-command',
  'file-read',
  'file-write',
  'command-approval',
  'safety',
]);

export function ToolCardsPanel(props: { messages: StreamLine[] }) {
  const tools = props.messages.filter((message) => toolKinds.has(message.kind)).slice(-6);

  return (
    <div className="lm-tools-card">
      <div className="lm-diff-header">
        <strong>Tool Cards</strong>
        <span className="lm-muted">{tools.length} event(s)</span>
      </div>

      {tools.length === 0 ? (
        <div className="lm-muted">Tool calls, shell commands, and file operations will appear here.</div>
      ) : (
        <div className="lm-tool-card-list">
          {tools.map((tool, index) => (
            <div key={`${tool.ts}-${index}`} className={`lm-tool-card event-${tool.kind}`}>
              <div className="lm-tool-kind">{tool.kind}</div>
              <pre>{tool.text}</pre>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
