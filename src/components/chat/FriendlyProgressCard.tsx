import { MarkdownMessage } from './MarkdownMessage';

export type FriendlyProgressInfo = {
  icon: string;
  title: string;
  body: string;
  meta?: string | null;
  kind: string;
  files?: string[];
  tool?: string | null;
  status?: string | null;
  command?: string | null;
  output?: string | null;
};

const KNOWN_TOOLS = new Set([
  'bash',
  'shell',
  'read',
  'write',
  'edit',
  'grep',
  'search',
  'glob',
  'list',
  'ls',
  'patch',
  'apply_patch',
]);

function stripQuotes(value: string) {
  return value.trim().replace(/^["']|["']$/g, '');
}

function decodeEscapes(value: string) {
  return value.replace(/\\n/g, '\n').replace(/\\"/g, '"');
}

function cleanNoise(text: string) {
  return decodeEscapes(text)
    .replace(/session=ses_[A-Za-z0-9]+/g, '')
    .replace(/message=msg_[A-Za-z0-9]+/g, '')
    .replace(/callID=call_[A-Za-z0-9_]+/g, '')
    .replace(/\bcall=call_[A-Za-z0-9_]+/g, '')
    .replace(/ses_[A-Za-z0-9]+/g, '')
    .replace(/msg_[A-Za-z0-9]+/g, '')
    .replace(/call_[A-Za-z0-9_]+/g, '')
    .replace(/\s+·\s+$/g, '')
    .replace(/\s{2,}/g, ' ')
    .trim();
}

function parseLooseFields(text: string) {
  const fields: Record<string, string> = {};
  const cleaned = decodeEscapes(text);
  const parts = cleaned.split(/\s+·\s+/).map((part) => part.trim()).filter(Boolean);

  const first = parts[0]?.toLowerCase();
  if (first && !first.includes('=') && KNOWN_TOOLS.has(first)) {
    fields.tool = first;
  }

  for (const part of parts) {
    const eq = part.match(/^\s*([A-Za-z0-9_.:-]+)\s*=\s*([\s\S]*)$/);
    if (eq) fields[eq[1]] = stripQuotes(eq[2]);
  }

  for (const pair of cleaned.matchAll(/"([^"]+)"\s*:\s*"([^"]*)"/g)) {
    fields[pair[1]] = pair[2];
  }

  const path = cleaned.match(/<path>([^<]+)<\/path>/)?.[1];
  if (path) fields.path = path;

  return fields;
}

function shortenPath(path?: string | null) {
  if (!path) return null;
  const clean = path.trim();
  const parts = clean.split('/').filter(Boolean);
  if (parts.length <= 3) return clean;
  return `…/${parts.slice(-3).join('/')}`;
}

function normalizeFileName(value: string) {
  return value
    .replace(/^\d+:\s*/, '')
    .replace(/^[-*]\s*/, '')
    .trim();
}

function extractPathFromInput(input?: string | null) {
  if (!input) return null;

  try {
    const parsed = JSON.parse(input);
    return parsed.file_path ?? parsed.filePath ?? parsed.path ?? parsed.uri ?? null;
  } catch {
    return (
      input.match(/"file_path"\s*:\s*"([^"]+)"/)?.[1] ??
      input.match(/"filePath"\s*:\s*"([^"]+)"/)?.[1] ??
      input.match(/"path"\s*:\s*"([^"]+)"/)?.[1] ??
      null
    );
  }
}

function extractFileList(fields: Record<string, string>) {
  const candidates = [fields.path, fields.filePath, fields.file_path, fields.file, extractPathFromInput(fields.input)].filter(Boolean) as string[];
  const output = fields.output ?? fields.preview ?? '';

  if (output.includes('entries=')) {
    const entries = output.replace(/^entries=/, '');
    for (const line of entries.split(/\n|,/)) {
      const clean = normalizeFileName(line);
      if (clean && !clean.startsWith('(')) candidates.push(clean);
    }
  }

  if (output.includes('content=')) {
    const path = fields.path ?? fields.filePath;
    if (path) candidates.push(path);
  }

  if (/No files found/i.test(output)) {
    return [];
  }

  return Array.from(new Set(candidates.map((item) => item.trim()).filter(Boolean)));
}

export function prettyToolName(value?: string | null) {
  if (!value) return 'Tool';
  const lower = value.toLowerCase();

  if (lower === 'bash' || lower.includes('shell')) return 'Shell';
  if (lower.includes('read') || lower === 'view') return 'Read';
  if (lower.includes('write')) return 'Write';
  if (lower.includes('edit')) return 'Edit';
  if (lower.includes('patch')) return 'Patch';
  if (lower.includes('grep') || lower.includes('search')) return 'Search';
  if (lower.includes('glob')) return 'Find files';
  if (lower.includes('list') || lower === 'ls') return 'List';

  return value.replace(/[_-]/g, ' ').replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function iconFor(kind: string, tool?: string | null) {
  const lower = tool?.toLowerCase() ?? '';

  if (kind === 'shell-command' || lower === 'bash' || lower.includes('shell')) return '$';
  if (kind === 'tool-call' || kind === 'file-read') {
    if (lower.includes('read')) return '↗';
    if (lower.includes('write') || lower.includes('edit') || lower.includes('patch')) return '✎';
    if (lower.includes('search') || lower.includes('grep')) return '⌕';
    if (lower.includes('list') || lower.includes('glob')) return '⌂';
    return '⌘';
  }

  if (kind === 'reasoning') return '◌';
  if (kind === 'file-write') return '✎';
  if (kind === 'diff') return 'Δ';
  if (kind === 'session-stats') return '◎';
  if (kind === 'command-approval') return '!';
  return '•';
}

function commandFromTitle(title?: string | null) {
  if (!title) return null;
  const match = title.match(/^run\s+(.+?)\s+command$/i);
  if (match?.[1]) return match[1];
  return null;
}

function titleWithoutCommand(title?: string | null) {
  if (!title) return null;
  return title.replace(/^Run\s+/i, '').replace(/\s+command$/i, '').trim();
}

function extractStats(text: string) {
  const total =
    text.match(/tokens\.total=(\d+)/)?.[1] ??
    text.match(/tokens total[=:]\s*(\d+)/i)?.[1] ??
    text.match(/total=(\d+)/)?.[1];

  const input = text.match(/input=(\d+)/)?.[1];
  const output = text.match(/output=(\d+)/)?.[1];
  const cost = text.match(/cost=\$?([0-9.]+)/)?.[1];

  return { total, input, output, cost };
}

function maybeCode(value?: string | null) {
  if (!value) return null;
  const clean = value.trim();
  if (!clean) return null;
  return `\`${clean}\``;
}

function isRawProtocolJson(value: string) {
  const trimmed = value.trim();
  return trimmed.startsWith('{') && (
    /"type"\s*:\s*"(system|user|assistant|tool_use|tool_result|result|rate_limit_event|message_start|message_stop|content_block_start|content_block_stop|input_json_delta|text_delta|ping)"/i.test(trimmed) ||
    /"subtype"\s*:\s*"hook_/i.test(trimmed) ||
    /"hook_name"\s*:/i.test(trimmed) ||
    /"hook_event"\s*:/i.test(trimmed) ||
    (/"session_id"\s*:/i.test(trimmed) && /"uuid"\s*:/i.test(trimmed))
  );
}

function outputSummary(output?: string | null) {
  if (!output) return null;
  const clean = output
    .replace(/^entries=/, '')
    .replace(/^content=/, '')
    .trim();

  if (!clean) return null;
  if (clean.length > 260) return `${clean.slice(0, 260)}…`;
  return clean;
}

export function buildFriendlyProgress(kind: string, rawText: string): FriendlyProgressInfo {
  const fields = parseLooseFields(rawText);
  const cleaned = isRawProtocolJson(rawText) ? '' : cleanNoise(rawText);

  const tool = fields.tool ?? fields.name;
  const status = fields.status;
  const file = fields.filePath ?? fields.file_path ?? fields.path ?? fields.file ?? extractPathFromInput(fields.input);
  const shortFile = shortenPath(file);
  const title = fields.title;
  const command = fields.command ?? commandFromTitle(title) ?? fields.cmd;
  const output = fields.output;
  const preview = fields.preview;
  const files = extractFileList(fields);
  const lowerTool = tool?.toLowerCase();

  if (kind === 'timeline' && /step started/i.test(rawText)) {
    return {
      kind,
      icon: '•',
      title: 'Started working',
      body: 'Planning the next step.',
      meta: null,
      tool,
      status,
    };
  }

  if (kind === 'reasoning') {
    return {
      kind,
      icon: '◌',
      title: 'Thinking',
      body: cleaned || 'Reasoning through the request.',
      meta: null,
      tool,
      status,
    };
  }

  if (kind === 'tool-call' || kind === 'file-read') {
    const toolName = prettyToolName(tool);

    if (lowerTool === 'bash' || toolName === 'Shell') {
      return {
        kind,
        icon: '$',
        title: status === 'completed' ? 'Ran command' : 'Running command',
        body: [
          maybeCode(command ?? titleWithoutCommand(title) ?? preview),
          output ? maybeCode(outputSummary(output)) : null,
        ].filter(Boolean).join('\n\n') || 'Running a shell command.',
        meta: null,
        tool,
        status,
        command: command ?? titleWithoutCommand(title) ?? preview ?? null,
        output: outputSummary(output),
      };
    }

    if (lowerTool === 'glob') {
      return {
        kind,
        icon: '⌂',
        title: 'Searched files',
        body: files.length > 0 ? files.map((item) => `- ${shortenPath(item) ?? item}`).join('\n') : (outputSummary(output) || preview || title || 'No files found'),
        meta: null,
        files,
        tool,
        status,
        output: outputSummary(output),
      };
    }

    if (toolName === 'Read') {
      return {
        kind,
        icon: '↗',
        title: 'Read files',
        body: files.length > 0
          ? files.map((item) => `- ${shortenPath(item) ?? item}`).join('\n')
          : shortFile || title || outputSummary(output) || 'Read project file content.',
        meta: null,
        files: files.length > 0 ? files : file ? [file] : [],
        tool,
        status,
        output: outputSummary(output),
      };
    }

    if (toolName === 'Search') {
      return {
        kind,
        icon: '⌕',
        title: 'Searched project',
        body: files.length > 0 ? files.map((item) => `- ${shortenPath(item) ?? item}`).join('\n') : (title || outputSummary(output) || preview || 'Searched the workspace.'),
        meta: null,
        files,
        tool,
        status,
        output: outputSummary(output),
      };
    }

    if (toolName === 'Write' || toolName === 'Edit' || toolName === 'Patch') {
      return {
        kind,
        icon: '✎',
        title: toolName === 'Patch' ? 'Applied patch' : `${toolName} file`,
        body: shortFile || title || outputSummary(output) || 'Applied workspace changes.',
        meta: null,
        files: file ? [file] : files,
        tool,
        status,
        output: outputSummary(output),
      };
    }

    const fallbackBody =
      title ||
      shortFile ||
      command ||
      preview ||
      outputSummary(output) ||
      (cleaned && cleaned.toLowerCase() !== 'tool' && cleaned !== '{' ? cleaned : null) ||
      'Claude used a tool.';

    return {
      kind,
      icon: iconFor(kind, tool),
      title: toolName === 'Tool' ? 'Used tool' : `Used ${toolName}`,
      body: fallbackBody,
      meta: null,
      files,
      tool,
      status,
      output: outputSummary(output),
    };
  }

  if (kind === 'shell-command') {
    return {
      kind,
      icon: '$',
      title: status === 'completed' ? 'Ran command' : 'Running command',
      body: [
        maybeCode(command ?? titleWithoutCommand(title) ?? preview),
        output ? maybeCode(outputSummary(output)) : null,
      ].filter(Boolean).join('\n\n') || cleaned || 'Running a shell command.',
      meta: null,
      tool,
      status,
      command: command ?? titleWithoutCommand(title) ?? preview ?? null,
      output: outputSummary(output),
    };
  }

  if (kind === 'file-write') {
    return {
      kind,
      icon: '✎',
      title: 'Edited file',
      body: shortFile || cleaned || 'Applied file changes.',
      meta: null,
      files: file ? [file] : [],
      tool,
      status,
    };
  }

  if (kind === 'diff') {
    return {
      kind,
      icon: 'Δ',
      title: 'Checked changes',
      body: cleaned.includes('diff stat') ? cleaned : 'Checked changed files and diffs.',
      meta: null,
      tool,
      status,
    };
  }

  if (kind === 'session-stats') {
    const stats = extractStats(rawText);
    return {
      kind,
      icon: '◎',
      title: 'Run finished',
      body: [
        stats.total ? `${Number(stats.total).toLocaleString()} tokens` : null,
        stats.cost ? `$${stats.cost}` : null,
      ].filter(Boolean).join(' · ') || 'The provider finished this run.',
      meta: [
        stats.input ? `in: ${Number(stats.input).toLocaleString()}` : null,
        stats.output ? `out: ${Number(stats.output).toLocaleString()}` : null,
      ].filter(Boolean).join(' · ') || null,
      tool,
      status,
    };
  }

  if (kind === 'command-approval') {
    return {
      kind,
      icon: '!',
      title: 'Needs approval',
      body: cleaned
        .replace(/risk=/i, 'Risk: ')
        .replace(/command=/i, 'Command: ')
        .replace(/reason=/i, 'Reason: '),
      meta: null,
      tool,
      status,
    };
  }

  return {
    kind,
    icon: iconFor(kind, tool),
    title: 'Working',
    body: cleaned || 'Working…',
    meta: null,
    tool,
    status,
  };
}

export function FriendlyProgressCard(props: { kind: string; text: string }) {
  const info = buildFriendlyProgress(props.kind, props.text);

  return (
    <div className={`lm-friendly-progress kind-${props.kind}`}>
      <div className="lm-friendly-icon">{info.icon}</div>
      <div className="lm-friendly-main">
        <div className="lm-friendly-title">{info.title}</div>
        <MarkdownMessage text={info.body || 'Working…'} />
        {info.meta ? <div className="lm-friendly-meta">{info.meta}</div> : null}
      </div>
    </div>
  );
}
