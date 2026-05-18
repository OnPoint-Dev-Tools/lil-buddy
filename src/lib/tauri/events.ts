import { listen } from '@tauri-apps/api/event';

export type RuntimeEventKind =
  | 'assistant'
  | 'stdout'
  | 'stderr'
  | 'status'
  | 'reasoning'
  | 'timeline'
  | 'session-stats'
  | 'tool-call'
  | 'file-read'
  | 'file-write'
  | 'file-change'
  | 'shell-command'
  | 'diff'
  | 'provider'
  | 'exit'
  | 'command-preview'
  | 'workspace'
  | 'desktop'
  | 'command-approval'
  | 'safety';

export type RuntimeStreamEvent = {
  kind: RuntimeEventKind;
  text: string;
  ts: string;
};

export async function attachRuntimeStream(
  onMessage: (event: RuntimeStreamEvent) => void,
) {
  return listen<RuntimeStreamEvent>('runtime://stream', (event) => {
    onMessage(event.payload);
  });
}
