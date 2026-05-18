import type { ProviderDefinition } from './types';

export const providers: ProviderDefinition[] = [
  {
    id: 'opencode-go',
    name: 'OpenCode',
    command: 'opencode',
    native: true,
    description: 'Primary Lil Buddy runtime. OpenCode handles model providers like OpenAI, Gemini, and OpenRouter internally.',
  },
  {
    id: 'claude',
    name: 'Claude Code',
    command: 'claude',
    description: 'Standalone Claude Code CLI wrapper using your local Claude login/session.',
  },
];
