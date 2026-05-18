import { useEffect, useMemo, useState } from 'react';
import { MarkdownMessage } from './MarkdownMessage';

function shouldAnimate(text: string) {
  return text.trim().length > 0 && text.length <= 24000;
}

function isFresh(ts?: string) {
  if (!ts) return true;

  const numeric = Number(ts);
  if (!Number.isFinite(numeric)) return true;

  return Date.now() - numeric < 45000;
}

export function StreamingMarkdownMessage(props: {
  text: string;
  active?: boolean;
  ts?: string;
  speed?: number;
}) {
  const { text, active = true, ts, speed = 10 } = props;
  const canAnimate = active && isFresh(ts) && shouldAnimate(text);
  const [visibleLength, setVisibleLength] = useState(() => canAnimate ? 0 : text.length);

  useEffect(() => {
    if (!canAnimate) {
      setVisibleLength(text.length);
      return;
    }

    setVisibleLength((current) => Math.min(current, text.length));
  }, [canAnimate, text]);

  useEffect(() => {
    if (!canAnimate) return;
    if (visibleLength >= text.length) return;

    const remaining = text.length - visibleLength;
    const step = remaining > 1200 ? 24 : remaining > 650 ? 14 : remaining > 260 ? 7 : 3;

    const timeout = window.setTimeout(() => {
      setVisibleLength((current) => Math.min(text.length, current + step));
    }, speed);

    return () => window.clearTimeout(timeout);
  }, [canAnimate, speed, text, visibleLength]);

  const visibleText = useMemo(() => {
    if (!canAnimate) return text;
    return text.slice(0, visibleLength);
  }, [canAnimate, text, visibleLength]);

  const streaming = canAnimate && visibleLength < text.length;

  return (
    <div className={streaming ? 'lm-streaming-markdown is-streaming' : 'lm-streaming-markdown'}>
      <MarkdownMessage text={visibleText} />
      {streaming ? <span className="lm-stream-caret" aria-hidden="true" /> : null}
    </div>
  );
}
