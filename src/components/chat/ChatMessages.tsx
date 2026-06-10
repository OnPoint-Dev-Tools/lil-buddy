import type { StreamLine } from '../../stores/appStore';
import { ActivityGroupCard } from './ActivityGroupCard';
import { MarkdownMessage } from './MarkdownMessage';
import { StreamingMarkdownMessage } from './StreamingMarkdownMessage';
import { ThinkingStreamCard } from './ThinkingStreamCard';
import { expertIconSrc, type LilExpert } from './ExpertsPanel';

type ReplyStats = {
  tokens?: string;
  input?: string;
  output?: string;
  cost?: string;
};

type ChatItem =
  | { type: 'message'; message: StreamLine; stats?: ReplyStats | null }
  | { type: 'activity'; items: StreamLine[] }
  | { type: 'thinking'; items: StreamLine[] };

function isUserPrompt(message: StreamLine) {
  return message.kind === 'user-message' && message.text.trim().length > 0;
}

function isAssistantMessage(message: StreamLine) {
  return message.kind === 'assistant' || message.kind === 'stdout';
}

function isThinkingMessage(message: StreamLine) {
  return message.kind === 'reasoning';
}

function isActivityMessage(message: StreamLine) {
  return [
    'tool-call',
    'shell-command',
    'file-read',
    'file-write',
    'diff',
    'command-approval',
    'timeline',
  ].includes(message.kind);
}

function normalizeText(value: string) {
  return value
    .replace(/session=ses_[A-Za-z0-9]+/g, '')
    .replace(/message=msg_[A-Za-z0-9]+/g, '')
    .replace(/callID=call_[A-Za-z0-9_]+/g, '')
    .replace(/ses_[A-Za-z0-9]+/g, '')
    .replace(/msg_[A-Za-z0-9]+/g, '')
    .replace(/call_[A-Za-z0-9_]+/g, '')
    .replace(/\s+/g, ' ')
    .trim()
    .toLowerCase();
}

function parseStats(text: string): ReplyStats {
  return {
    tokens:
      text.match(/tokens\.total=(\d+)/)?.[1] ??
      text.match(/tokens total[=:]\s*(\d+)/i)?.[1] ??
      text.match(/total=(\d+)/)?.[1],
    input: text.match(/input=(\d+)/)?.[1],
    output: text.match(/output=(\d+)/)?.[1],
    cost: text.match(/cost=\$?([0-9.]+)/)?.[1],
  };
}

function visibleKey(message: StreamLine) {
  if (message.kind === 'stdout' || message.kind === 'assistant') {
    return `assistant:${normalizeText(message.text)}`;
  }

  if (message.kind === 'reasoning') {
    return `reasoning:${message.ts}:${normalizeText(message.text)}`;
  }

  if (message.kind === 'tool-call') {
    const tool =
      message.text.match(/"tool"\s*:\s*"([^"]+)"/)?.[1] ??
      message.text.match(/\btool=([A-Za-z0-9_.:-]+)/)?.[1] ??
      message.text.split(/\s+·\s+/)[0] ??
      'tool';

    const file =
      message.text.match(/"filePath"\s*:\s*"([^"]+)"/)?.[1] ??
      message.text.match(/<path>([^<]+)<\/path>/)?.[1] ??
      '';

    const title = message.text.match(/"title"\s*:\s*"([^"]+)"/)?.[1] ?? message.text.match(/\btitle=([^·\n]+)/)?.[1] ?? '';
    const status = message.text.match(/"status"\s*:\s*"([^"]+)"/)?.[1] ?? message.text.match(/\bstatus=([^·\n]+)/)?.[1] ?? '';

    return `tool:${message.ts}:${tool}:${file}:${title}:${status}`.toLowerCase();
  }

  if (message.kind === 'timeline' && /OpenCode step started|Started working/i.test(message.text)) {
    return `timeline:step-start:${message.ts}`;
  }

  if (message.kind === 'session-stats') {
    const tokens = message.text.match(/tokens\.total=(\d+)/)?.[1] ?? normalizeText(message.text);
    return `session-stats:${tokens}`;
  }

  return `${message.kind}:${normalizeText(message.text)}`;
}

function dedupeVisible(messages: StreamLine[]) {
  const seen = new Set<string>();

  return messages.filter((message) => {
    const key = visibleKey(message);
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function attachStatsToLastAssistant(items: ChatItem[], stats: ReplyStats) {
  for (let index = items.length - 1; index >= 0; index -= 1) {
    const item = items[index];

    if (item.type === 'message' && isAssistantMessage(item.message)) {
      item.stats = stats;
      return true;
    }

    if (item.type === 'message' && isUserPrompt(item.message)) {
      break;
    }
  }

  return false;
}

function buildChatItems(messages: StreamLine[]) {
  const filtered = dedupeVisible(messages.filter(
    (message) =>
      (isAssistantMessage(message) && message.text.trim().length > 0) ||
      isUserPrompt(message) ||
      isActivityMessage(message) ||
      isThinkingMessage(message) ||
      message.kind === 'session-stats' ||
      message.kind === 'stderr',
  ));

  const items: ChatItem[] = [];
  let activityBuffer: StreamLine[] = [];
  let thinkingBuffer: StreamLine[] = [];
  let pendingStats: ReplyStats | null = null;

  function flushActivity() {
    if (activityBuffer.length > 0) {
      items.push({ type: 'activity', items: activityBuffer });
      activityBuffer = [];
    }
  }

  function flushThinking() {
    if (thinkingBuffer.length > 0) {
      items.push({ type: 'thinking', items: thinkingBuffer });
      thinkingBuffer = [];
    }
  }

  function flushAll() {
    flushThinking();
    flushActivity();
  }

  for (const message of filtered) {
    if (message.kind === 'session-stats') {
      const stats = parseStats(message.text);

      if (!attachStatsToLastAssistant(items, stats)) {
        pendingStats = stats;
      }

      continue;
    }

    if (isThinkingMessage(message)) {
      flushActivity();
      thinkingBuffer.push(message);
      continue;
    }

    if (isActivityMessage(message)) {
      flushThinking();
      activityBuffer.push(message);
      continue;
    }

    flushAll();

    if (isAssistantMessage(message)) {
      items.push({ type: 'message', message, stats: pendingStats });
      pendingStats = null;
      continue;
    }

    items.push({ type: 'message', message });
  }

  flushAll();
  return items;
}

function StatsLine(props: { stats?: ReplyStats | null }) {
  if (!props.stats) return null;

  const { stats } = props;
  const parts = [
    stats.tokens ? `${Number(stats.tokens).toLocaleString()} tokens` : null,
    stats.cost ? `$${stats.cost}` : null,
    stats.input && stats.output
      ? `in ${Number(stats.input).toLocaleString()} · out ${Number(stats.output).toLocaleString()}`
      : null,
  ].filter(Boolean);

  if (parts.length === 0) return null;

  return <div className="lm-ai-stats">{parts.join(' · ')}</div>;
}


function ExpertAvatar(props: { expert?: LilExpert | null }) {
  if (!props.expert) {
    return <div className="lm-chat-avatar">LM</div>;
  }

  return (
    <div className="lm-chat-avatar expert">
      <img src={expertIconSrc(props.expert.icon)} alt="" draggable={false} />
    </div>
  );
}

function assistantName(expert?: LilExpert | null) {
  return expert?.name ?? 'Lil Buddy';
}

function isFresh(ts: string) {
  const numeric = Number(ts);
  if (!Number.isFinite(numeric)) return true;
  return Date.now() - numeric < 45000;
}

export function ChatMessages(props: { messages: StreamLine[]; running: boolean; expert?: LilExpert | null }) {
  const chatItems = buildChatItems(props.messages);
  const hasAnyRealMessage = chatItems.length > 0;

  return (
    <div className="lm-chat-thread">
      {!hasAnyRealMessage ? (
        <div className="lm-chat-bubble assistant">
          <ExpertAvatar expert={props.expert} />
          <div className="lm-chat-content">
            <div className="lm-chat-author">{assistantName(props.expert)}</div>
            <MarkdownMessage text="Hey, I’m ready to help you plan, organize ideas, explain files, or what ever you want 😁 and keep you updated while I work." />
          </div>
        </div>
      ) : null}

      {chatItems.map((item, index) => {
        if (item.type === 'thinking') {
          return (
            <div key={`thinking-${index}`} className="lm-chat-bubble thinking-stream">
              <ThinkingStreamCard items={item.items} running={props.running && index === chatItems.length - 1} />
            </div>
          );
        }

        if (item.type === 'activity') {
          return (
            <div key={`activity-${index}`} className="lm-chat-bubble progress friendly grouped">
              <ActivityGroupCard items={item.items} running={props.running && index === chatItems.length - 1} />
            </div>
          );
        }

        const message = item.message;

        if (isUserPrompt(message)) {
          return (
            <div key={`${message.ts}-${index}`} className="lm-chat-bubble user">
              <div className="lm-chat-content">
                <MarkdownMessage text={message.text} />
              </div>
            </div>
          );
        }

        if (message.kind === 'stderr') {
          return (
            <div key={`${message.ts}-${index}`} className="lm-chat-bubble system error">
              <div className="lm-chat-content">
                <div className="lm-chat-author">Runtime</div>
                <MarkdownMessage text={message.text} />
              </div>
            </div>
          );
        }

        return (
          <div key={`${message.ts}-${index}`} className="lm-chat-bubble assistant">
            <ExpertAvatar expert={props.expert} />
            <div className="lm-chat-content">
              <div className="lm-chat-author">{assistantName(props.expert)}</div>
              <StreamingMarkdownMessage text={message.text} ts={message.ts} active={isFresh(message.ts)} />
              <StatsLine stats={item.stats} />
            </div>
          </div>
        );
      })}

      {props.running ? (
        <div className="lm-chat-bubble assistant thinking">
          <ExpertAvatar expert={props.expert} />
          <div className="lm-chat-content">
            <div className="lm-chat-author">{assistantName(props.expert)}</div>
            <div className="lm-typing"><span /><span /><span /></div>
          </div>
        </div>
      ) : null}
    </div>
  );
}
