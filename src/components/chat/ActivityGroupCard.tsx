import type { StreamLine } from '../../stores/appStore';
import { MarkdownMessage } from './MarkdownMessage';
import { buildFriendlyProgress, prettyToolName, type FriendlyProgressInfo } from './FriendlyProgressCard';

type DisplayItem =
  | FriendlyProgressInfo
  | {
      kind: 'file-sequence' | 'tool-sequence';
      icon: string;
      title: string;
      body: string;
      meta?: string | null;
    };

function isFileLookup(info: FriendlyProgressInfo) {
  return (
    info.title === 'Read files' ||
    info.title === 'Read file' ||
    info.title === 'Searched files' ||
    info.title === 'Searched project'
  );
}

function isGenericTool(info: FriendlyProgressInfo) {
  return Boolean(
    info.tool &&
    !isFileLookup(info) &&
    info.title !== 'Run finished' &&
    info.title !== 'Started working' &&
    info.title !== 'Thinking' &&
    info.kind !== 'session-stats'
  );
}

function shortenPath(path: string) {
  const parts = path.split('/').filter(Boolean);
  if (parts.length <= 3) return path;
  return `…/${parts.slice(-3).join('/')}`;
}

function mergeActivity(items: FriendlyProgressInfo[]): DisplayItem[] {
  const out: DisplayItem[] = [];
  let fileBuffer: FriendlyProgressInfo[] = [];
  let toolBuffer: FriendlyProgressInfo[] = [];

  function flushFiles() {
    if (fileBuffer.length === 0) return;

    const files = Array.from(new Set(
      fileBuffer.flatMap((item) => item.files ?? [])
        .map((file) => file.trim())
        .filter(Boolean),
    ));

    const anySearch = fileBuffer.some((item) => item.title.includes('Search') || item.title.includes('Searched'));
    const title = anySearch ? 'Checked files' : 'Read files';

    if (files.length > 0) {
      out.push({
        kind: 'file-sequence',
        icon: anySearch ? '⌕' : '↗',
        title,
        body: files.map((file) => `- ${shortenPath(file)}`).join('\n'),
        meta: `${files.length} file${files.length === 1 ? '' : 's'}`,
      });
    } else {
      const meaningful = fileBuffer
        .map((item) => item.body)
        .filter((body) => body && !/status:/i.test(body))
        .slice(0, 4);

      out.push({
        kind: 'file-sequence',
        icon: anySearch ? '⌕' : '↗',
        title,
        body: meaningful.join('\n') || 'No matching files found.',
        meta: null,
      });
    }

    fileBuffer = [];
  }

  function flushTools() {
    if (toolBuffer.length === 0) return;

    if (toolBuffer.length === 1) {
      out.push(toolBuffer[0]);
      toolBuffer = [];
      return;
    }

    const lines = toolBuffer.map((item) => {
      const name = prettyToolName(item.tool);
      const detail =
        name === 'Model' ? ` — ${item.body}` :
        item.command ? ` — \`${item.command}\`` :
        item.files?.[0] ? ` — ${shortenPath(item.files[0])}` :
        item.output ? ` — \`${item.output}\`` :
        item.body ? ` — ${item.body.replace(/\n/g, ' ')}` :
        '';

      return `- **${name}**${detail}`;
    });

    out.push({
      kind: 'tool-sequence',
      icon: '⌘',
      title: 'Used tools',
      body: lines.join('\n'),
      meta: `${toolBuffer.length} tool call${toolBuffer.length === 1 ? '' : 's'}`,
    });

    toolBuffer = [];
  }

  function flushAll() {
    flushFiles();
    flushTools();
  }

  for (const item of items) {
    if (isFileLookup(item)) {
      flushTools();
      fileBuffer.push(item);
      continue;
    }

    if (isGenericTool(item)) {
      flushFiles();
      toolBuffer.push(item);
      continue;
    }

    flushAll();
    out.push(item);
  }

  flushAll();
  return out;
}

export function ActivityGroupCard(props: {
  items: StreamLine[];
  running: boolean;
}) {
  const visible = mergeActivity(props.items.map((item) => buildFriendlyProgress(item.kind, item.text)));

  return (
    <div className="lm-activity-card">
      <div className="lm-activity-header">
        <div>
          <strong>{props.running ? 'Working now' : 'Work summary'}</strong>
          <span>{visible.length} update{visible.length === 1 ? '' : 's'}</span>
        </div>
        {props.running ? <i className="lm-live-dot" /> : null}
      </div>

      <div className="lm-activity-list">
        {visible.map((item, index) => (
          <div key={`${item.title}-${index}`} className={`lm-activity-row kind-${item.kind}`}>
            <div className="lm-friendly-icon">{item.icon}</div>
            <div className="lm-friendly-main">
              <div className="lm-friendly-title">{item.title}</div>
              <MarkdownMessage text={item.body || 'Working…'} />
              {item.meta ? <div className="lm-friendly-meta">{item.meta}</div> : null}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
