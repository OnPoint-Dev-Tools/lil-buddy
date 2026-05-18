import type { StreamLine } from '../../stores/appStore';

export function ActivityRail(props: { messages: StreamLine[]; running: boolean }) {
  const latest = props.messages.slice(-4).reverse();

  return (
    <div className="lm-activity-rail">
      <div className="lm-rail-title">
        <span className={props.running ? 'lm-dot live' : 'lm-dot'} />
        Activity
      </div>

      {latest.length === 0 ? (
        <div className="lm-muted">No activity yet.</div>
      ) : (
        latest.map((item, index) => (
          <div key={`${item.ts}-${index}`} className="lm-rail-item">
            <span className="lm-rail-kind">{item.kind}</span>
            <span>{item.text}</span>
          </div>
        ))
      )}
    </div>
  );
}
