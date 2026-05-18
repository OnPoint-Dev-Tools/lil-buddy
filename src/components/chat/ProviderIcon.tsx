import opencodeIcon from '../../assets/provider-icons/opencode.svg';
import openrouterIcon from '../../assets/provider-icons/openrouter.svg';
import openaiIcon from '../../assets/provider-icons/openai.svg';
import claudeIcon from '../../assets/provider-icons/claude.svg';
import geminiIcon from '../../assets/provider-icons/gemini.svg';

function iconForProvider(providerId: string) {
  const id = providerId.toLowerCase();

  if (id.includes('opencode')) return opencodeIcon;
  if (id.includes('openrouter')) return openrouterIcon;
  if (id.includes('openai') || id.includes('gpt') || id.includes('codex')) return openaiIcon;
  if (id.includes('anthropic') || id.includes('claude')) return claudeIcon;
  if (id.includes('google') || id.includes('gemini')) return geminiIcon;

  return opencodeIcon;
}

export function ProviderIcon(props: {
  providerId: string;
  label?: string;
  className?: string;
}) {
  return (
    <span className={props.className ? `lm-provider-icon ${props.className}` : 'lm-provider-icon'}>
      <img src={iconForProvider(props.providerId)} alt={props.label ?? props.providerId} />
    </span>
  );
}
