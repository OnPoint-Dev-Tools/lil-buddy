function stripProtocolJsonLines(value: string) {
  return value
    .split(/\n+/)
    .filter((line) => {
      const trimmed = line.trim();

      if (!trimmed.startsWith('{') || !trimmed.endsWith('}')) return true;

      return !(
        /"type"\s*:\s*"(system|user|assistant|tool_use|tool_result|result|rate_limit_event|message_start|message_stop|content_block_start|content_block_stop|input_json_delta|text_delta|ping)"/i.test(trimmed) ||
        /"subtype"\s*:\s*"hook_/i.test(trimmed) ||
        /"hook_name"\s*:/i.test(trimmed) ||
        /"hook_event"\s*:/i.test(trimmed) ||
        (/"session_id"\s*:/i.test(trimmed) && /"uuid"\s*:/i.test(trimmed))
      );
    })
    .join('\n')
    .trim();
}

function cleanDisplayText(value: string) {
  return stripProtocolJsonLines(value)
    .replace(/\u001b\[[0-9;?]*[A-Za-z]/g, '')
    .replace(/\x1b\[[0-9;?]*[A-Za-z]/g, '')
    .replace(/[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F]/g, '')
    .replace(/â\uFFFD\uFFFD/g, '')
    .replace(/â□□/g, '')
    .replace(/â\[\]/g, '');
}

function escapeHtml(value: string) {
  const cleaned = cleanDisplayText(value);
  return cleaned
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}

function renderInline(value: string) {
  return escapeHtml(value)
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/`([^`]+)`/g, '<code>$1</code>');
}

function markdownToHtml(markdown: string) {
  return markdown.split(/(```[\s\S]*?```)/g).map((block) => {
    if (block.startsWith('```')) {
      const code = block.replace(/^```[a-zA-Z0-9_-]*\n?/, '').replace(/```$/, '');
      return `<pre><code>${escapeHtml(code)}</code></pre>`;
    }

    const lines = block.split('\n');
    const out: string[] = [];
    let inList = false;

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) {
        if (inList) {
          out.push('</ul>');
          inList = false;
        }
        out.push('<br />');
        continue;
      }

      const bullet = trimmed.match(/^[-*]\s+(.+)/);
      if (bullet) {
        if (!inList) {
          out.push('<ul>');
          inList = true;
        }
        out.push(`<li>${renderInline(bullet[1])}</li>`);
        continue;
      }

      if (inList) {
        out.push('</ul>');
        inList = false;
      }

      if (trimmed.startsWith('### ')) out.push(`<h3>${renderInline(trimmed.slice(4))}</h3>`);
      else if (trimmed.startsWith('## ')) out.push(`<h2>${renderInline(trimmed.slice(3))}</h2>`);
      else if (trimmed.startsWith('# ')) out.push(`<h1>${renderInline(trimmed.slice(2))}</h1>`);
      else out.push(`<p>${renderInline(line)}</p>`);
    }

    if (inList) out.push('</ul>');
    return out.join('');
  }).join('');
}

export function MarkdownMessage(props: { text: string }) {
  return <div className="lm-markdown" dangerouslySetInnerHTML={{ __html: markdownToHtml(props.text) }} />;
}
