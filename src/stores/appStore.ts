import { create } from 'zustand';
import type { DesktopEnvironment, ProviderStatus } from '../lib/providers/types';
import type { RuntimeEventKind } from '../lib/tauri/events';
import type { WorkspaceDiff, WorkspaceInfo } from '../lib/tauri/commands';

export type TabId = 'chat' | 'tool-calls' | 'file-changes' | 'history' | 'logs';

export interface StreamLine {
  kind: RuntimeEventKind | 'user-message';
  text: string;
  ts: string;
}

interface AppState {
  activeTab: TabId;
  advancedOpen: boolean;
  commandPaletteOpen: boolean;
  confirmRunOpen: boolean;
  prompt: string;
  commandPreview: string;
  workspacePathDraft: string;
  selectedProvider: string;
  opencodeGoMode: string;
  providers: ProviderStatus[];
  messages: StreamLine[];
  running: boolean;
  desktop: DesktopEnvironment | null;
  workspace: WorkspaceInfo | null;
  workspaceDiff: WorkspaceDiff | null;
  setActiveTab: (tab: TabId) => void;
  setAdvancedOpen: (open: boolean) => void;
  setCommandPaletteOpen: (open: boolean) => void;
  setConfirmRunOpen: (open: boolean) => void;
  setPrompt: (value: string) => void;
  setCommandPreview: (value: string) => void;
  setWorkspacePathDraft: (value: string) => void;
  setSelectedProvider: (value: string) => void;
  setOpenCodeGoMode: (value: string) => void;
  setProviders: (providers: ProviderStatus[]) => void;
  setDesktop: (desktop: DesktopEnvironment) => void;
  setWorkspace: (workspace: WorkspaceInfo) => void;
  setWorkspaceDiff: (workspaceDiff: WorkspaceDiff) => void;
  pushMessage: (value: StreamLine) => void;
  setMessages: (messages: StreamLine[]) => void;
  clearMessages: () => void;
  setRunning: (value: boolean) => void;
  seedBoot: () => void;
}

function isProtocolJsonLine(line: string) {
  const trimmed = line.trim();

  if (!trimmed.startsWith('{') || !trimmed.endsWith('}')) {
    return false;
  }

  try {
    const parsed = JSON.parse(trimmed) as { type?: string; subtype?: string; hook_name?: string; hook_event?: string; session_id?: string; uuid?: string };
    const type = String(parsed.type ?? '').toLowerCase();
    const subtype = String(parsed.subtype ?? '').toLowerCase();

    return [
      'system',
      'user',
      'assistant',
      'tool_use',
      'tool_result',
      'result',
      'rate_limit_event',
      'message_start',
      'message_stop',
      'content_block_start',
      'content_block_stop',
      'input_json_delta',
      'text_delta',
      'ping',
    ].includes(type) || subtype.startsWith('hook_') || Boolean(parsed.hook_name) || Boolean(parsed.hook_event) || (Boolean(parsed.session_id) && Boolean(parsed.uuid));
  } catch {
    return /"type"\s*:\s*"(system|user|assistant|tool_use|tool_result|result|rate_limit_event|message_start|message_stop|content_block_start|content_block_stop|input_json_delta|text_delta|ping)"/i.test(trimmed) ||
      /"subtype"\s*:\s*"hook_/i.test(trimmed) ||
      /"hook_name"\s*:/i.test(trimmed);
  }
}

function stripProtocolNoise(value: string) {
  return value
    .split(/\n+/)
    .filter((line) => !isProtocolJsonLine(line))
    .join('\n')
    .trim();
}

export const useAppStore = create<AppState>((set) => ({
  activeTab: 'chat',
  advancedOpen: false,
  commandPaletteOpen: false,
  confirmRunOpen: false,
  prompt: '',
  commandPreview: '',
  workspacePathDraft: '',
  selectedProvider: 'opencode-go',
  opencodeGoMode: 'run-json',
  providers: [],
  messages: [],
  running: false,
  desktop: null,
  workspace: null,
  workspaceDiff: null,
  setActiveTab: (activeTab) => set({ activeTab }),
  setAdvancedOpen: (advancedOpen) => set({ advancedOpen }),
  setCommandPaletteOpen: (commandPaletteOpen) => set({ commandPaletteOpen }),
  setConfirmRunOpen: (confirmRunOpen) => set({ confirmRunOpen }),
  setPrompt: (prompt) => set({ prompt }),
  setCommandPreview: (commandPreview) => set({ commandPreview }),
  setWorkspacePathDraft: (workspacePathDraft) => set({ workspacePathDraft }),
  setSelectedProvider: (selectedProvider) => set({ selectedProvider }),
  setOpenCodeGoMode: (opencodeGoMode) => set({ opencodeGoMode }),
  setProviders: (providers) => set({ providers }),
  setDesktop: (desktop) => set({ desktop }),
  setWorkspace: (workspace) => set({ workspace }),
  setWorkspaceDiff: (workspaceDiff) => set({ workspaceDiff }),
  pushMessage: (value) => set((state) => {
    const normalize = (input: string) =>
      input
        .replace(/session=ses_[A-Za-z0-9]+/g, '')
        .replace(/message=msg_[A-Za-z0-9]+/g, '')
        .replace(/callID=call_[A-Za-z0-9_]+/g, '')
        .replace(/ses_[A-Za-z0-9]+/g, '')
        .replace(/msg_[A-Za-z0-9]+/g, '')
        .replace(/call_[A-Za-z0-9_]+/g, '')
        .replace(/\s+/g, ' ')
        .trim()
        .toLowerCase();

    const text = ['assistant', 'stdout', 'tool-call', 'stderr'].includes(value.kind)
      ? stripProtocolNoise(value.text)
      : value.text.trim();
    const normalizedText = normalize(text);

    if (!normalizedText) {
      return state;
    }

    const now = Number(value.ts) || Date.now();
    const recent = state.messages.slice(-20);

    if (value.kind === 'command-preview') {
      const previewPrompt = value.text.match(/'([^']+)'\s*$/)?.[1] ?? value.text;
      const previewNorm = normalize(previewPrompt);

      if (recent.some((message) => message.kind === 'user-message' && normalize(message.text) === previewNorm)) {
        return state;
      }
    }

    const duplicateRecent = recent.some((message) => {
      const messageTime = Number(message.ts) || now;
      const closeInTime = Math.abs(now - messageTime) < 12000;
      const sameKind =
        message.kind === value.kind ||
        ((message.kind === 'assistant' || message.kind === 'stdout') && (value.kind === 'assistant' || value.kind === 'stdout'));

      return sameKind && closeInTime && normalize(message.text) === normalizedText;
    });

    if (duplicateRecent) {
      return state;
    }

    const last = state.messages[state.messages.length - 1];

    if (
      last &&
      (last.kind === 'assistant' || last.kind === 'stdout') &&
      (value.kind === 'assistant' || value.kind === 'stdout') &&
      Math.abs(now - (Number(last.ts) || now)) < 30000
    ) {
      const lastNorm = normalize(last.text);

      if (lastNorm === normalizedText || lastNorm.endsWith(normalizedText)) {
        return state;
      }

      const separator = last.text.endsWith('\n') || text.startsWith('\n') ? '' : '\n';
      return {
        messages: [
          ...state.messages.slice(0, -1),
          {
            ...last,
            kind: 'assistant',
            text: `${last.text}${separator}${text}`,
            ts: value.ts,
          },
        ],
      };
    }

    return { messages: [...state.messages, { ...value, text }] };
  }),
  setMessages: (messages) => set({ messages }),
  clearMessages: () => set({ messages: [] }),
  setRunning: (running) => set({ running }),
  seedBoot: () => set({
    messages: [
      {
        kind: 'status',
        text: 'Lil Buddy ready. File diff tracking loaded.',
        ts: new Date().toISOString(),
      },
      {
        kind: 'stdout',
        text: "After provider runs, I can capture git status, changed files, and diffs.",
        ts: new Date().toISOString(),
      },
    ],
  }),
}));
