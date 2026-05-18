import { TabBar } from '../layout/TabBar';
import type { StreamLine, TabId } from '../../stores/appStore';

function filterMessages(messages: StreamLine[], tab: TabId) {
  if (tab === 'chat') return messages.filter((item) => item.kind === 'stdout' || item.kind === 'status' || item.kind === 'workspace' || item.kind === 'desktop');
  if (tab === 'tool-calls') return messages.filter((item) => item.kind === 'tool-call' || item.kind === 'provider' || item.kind === 'command-preview' || item.kind === 'shell-command' || item.kind === 'command-approval' || item.kind === 'safety' || item.kind === 'session-stats');
  if (tab === 'file-changes') return messages.filter((item) => item.kind === 'file-change' || item.kind === 'file-read' || item.kind === 'file-write' || item.kind === 'diff');
  if (tab === 'logs') return messages.filter((item) => item.kind === 'stderr' || item.kind === 'status' || item.kind === 'exit');
  return messages;
}

export function RuntimePanel(props: {
  open: boolean;
  activeTab: TabId;
  messages: StreamLine[];
  onTabChange: (tab: TabId) => void;
}) {
  if (!props.open) return null;

  const visible = filterMessages(props.messages, props.activeTab);

  return (
    <div className="lm-advanced">
      <TabBar activeTab={props.activeTab} onChange={props.onTabChange} />
      <div className="lm-runtime-panel">
        {visible.length === 0 ? (
          <div className="lm-muted">No runtime events yet.</div>
        ) : (
          visible.map((item, index) => (
            <div key={`${item.ts}-${index}`} className={`lm-runtime-line event-${item.kind}`}>
              <span className="lm-runtime-kind">[{item.kind}]</span> {item.text}
            </div>
          ))
        )}
      </div>
    </div>
  );
}
