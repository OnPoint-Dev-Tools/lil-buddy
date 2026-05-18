import { useEffect, useMemo, useState } from 'react';
import type { StreamLine } from '../../stores/appStore';
import { StreamingMarkdownMessage } from './StreamingMarkdownMessage';

function cleanReasoningText(text: string) {
  return text
    .replace(/^Thinking:\s*/i, '')
    .replace(/session=ses_[A-Za-z0-9]+/g, '')
    .replace(/message=msg_[A-Za-z0-9]+/g, '')
    .replace(/callID=call_[A-Za-z0-9_]+/g, '')
    .trim();
}

function splitIntoParts(text: string) {
  const clean = cleanReasoningText(text);
  if (!clean) return [];

  const paragraphs = clean
    .split(/\n{2,}/)
    .map((part) => part.trim())
    .filter(Boolean);

  const source = paragraphs.length > 0 ? paragraphs : [clean];
  const chunks: string[] = [];

  for (const paragraph of source) {
    if (paragraph.length <= 520) {
      chunks.push(paragraph);
      continue;
    }

    const sentences = paragraph.split(/(?<=[.!?])\s+/).filter(Boolean);
    let current = '';

    for (const sentence of sentences) {
      const candidate = current ? `${current} ${sentence}` : sentence;

      if (candidate.length > 520 && current) {
        chunks.push(current);
        current = sentence;
      } else {
        current = candidate;
      }
    }

    if (current) {
      if (current.length <= 620) {
        chunks.push(current);
      } else {
        for (let index = 0; index < current.length; index += 520) {
          chunks.push(current.slice(index, index + 520));
        }
      }
    }
  }

  return chunks.slice(0, 18);
}

export function ThinkingStreamCard(props: {
  items: StreamLine[];
  running: boolean;
}) {
  const [open, setOpen] = useState(true);
  const [visibleParts, setVisibleParts] = useState(1);

  const parts = useMemo(() => {
    return props.items.flatMap((item) => splitIntoParts(item.text));
  }, [props.items]);

  useEffect(() => {
    if (!open) return;
    setVisibleParts((current) => Math.min(Math.max(current, 1), Math.max(parts.length, 1)));
  }, [open, parts.length]);

  useEffect(() => {
    if (!open) return;
    if (visibleParts >= parts.length) return;

    const timeout = window.setTimeout(() => {
      setVisibleParts((current) => Math.min(parts.length, current + 1));
    }, 520);

    return () => window.clearTimeout(timeout);
  }, [open, parts.length, visibleParts]);

  if (parts.length === 0) return null;

  const visible = open ? parts.slice(0, visibleParts) : [];

  return (
    <div className={open ? 'lm-thinking-card open' : 'lm-thinking-card'}>
      <button className="lm-thinking-header" onClick={() => setOpen((current) => !current)}>
        <span className="lm-thinking-dot">◌</span>
        <span>
          <strong>{props.running ? 'Model thinking' : 'Model thinking'}</strong>
          <small>{parts.length} part{parts.length === 1 ? '' : 's'}</small>
        </span>
        <i>{open ? 'Collapse' : 'Open'}</i>
      </button>

      {open ? (
        <div className="lm-thinking-parts">
          {visible.map((part, index) => (
            <div key={`${index}-${part.slice(0, 16)}`} className="lm-thinking-part">
              <span>{index + 1}</span>
              <StreamingMarkdownMessage text={part} active={index === visible.length - 1 && props.running} speed={14} />
            </div>
          ))}

          {visibleParts < parts.length ? (
            <div className="lm-thinking-more">
              Streaming model thinking…
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
