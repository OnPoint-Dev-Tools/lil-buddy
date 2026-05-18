import type { PendingCommandApproval } from '../../lib/tauri/commands';

export function ApprovalConfirm(props: {
  approval: PendingCommandApproval | null;
  onApprove: () => void;
  onDeny: () => void;
}) {
  if (!props.approval) return null;

  return (
    <div
      className="lm-modal-backdrop approval-backdrop"
      onMouseDown={(event) => event.stopPropagation()}
      onClick={props.onDeny}
    >
      <div className="lm-approval-modal" onClick={(event) => event.stopPropagation()}>
        <div className="lm-modal-header compact">
          <div>
            <div className="lm-modal-title">{props.approval.command.startsWith('/') ? 'Directory Approval' : 'Command Approval'}</div>
            <div className="lm-muted">{props.approval.command.startsWith('/') ? 'OpenCode needs access outside the current working directory.' : 'Lil Buddy detected a risky command pattern.'}</div>
          </div>
          <button className="lm-icon-btn" onClick={props.onDeny}>×</button>
        </div>

        <div className="lm-approval-body">
          <div className="lm-risk-pill">{props.approval.risk_level}</div>

          <div>
            <strong>{props.approval.command.startsWith('/') ? 'OpenCode wants to access:' : 'OpenCode wants to run:'}</strong>
            <pre>{props.approval.command}</pre>
          </div>

          <div>
            <strong>Reason:</strong>
            <p>{props.approval.reason}</p>
          </div>

          <div className="lm-confirm-actions">
            <button className="lm-ghost" onClick={props.onDeny}>Deny</button>
            <button className="lm-danger-run" onClick={props.onApprove}>Approve</button>
          </div>
        </div>
      </div>
    </div>
  );
}
