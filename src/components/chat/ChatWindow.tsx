import { useEffect, useRef, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  hideChatWindow,
  detectProviders,
  previewProviderCommand,
  runProviderCommand,
  providerIsRunning,
  saveSelectedProvider,
  saveOpenCodeGoMode,
  stopProviderCommand,
  loadSettings,
  detectDesktop,
  detectWorkspace,
  getWorkspaceDiff,
  emitWorkspaceDiffEvents,
  chooseWorkspaceFolder,
  openWorkspaceTerminal,
  saveWorkspacePath,
  saveWorkspaceSession,
  restoreWorkspaceFile,
  restoreAllWorkspaceFiles,
  requestCommandApproval,
  approveCommandApproval,
  denyCommandApproval,
  classifyCommandRisk,
  allowExternalDirectory,
  setNativeCompanionCategory,
  saveTelegramActiveExpert,
  saveTelegramExperts,
} from '../../lib/tauri/commands';
import { attachRuntimeStream } from '../../lib/tauri/events';
import { useAppStore, type StreamLine } from '../../stores/appStore';
import { PromptChips } from './PromptChips';
import { RuntimePanel } from './RuntimePanel';
import { QuickActions } from './QuickActions';
import { RestoreConfirm, type RestoreTarget } from './RestoreConfirm';
import { ApprovalConfirm } from './ApprovalConfirm';
import { ChatMessages } from './ChatMessages';
import type { PendingCommandApproval } from '../../lib/tauri/commands';
import { WorkspaceModal } from './WorkspaceModal';
import { SettingsPanel } from './SettingsPanel';
import { ModelProviderSelector } from './ModelProviderSelector';
import { ResizeHandle } from './ResizeHandle';
import { OnboardingModal } from './OnboardingModal';
import { ExpertsPanel, expertGreeting, expertPromptPrefix, loadExperts, selectedExpert, type LilExpert } from './ExpertsPanel';
import lilBuddyLogo from '../../assets/branding/lil-buddy-logo-header.png';
import {
   Maximize2,
   MoreHorizontal,
   Minus,
   NotepadTextDashed,
   Paperclip,
   Settings,
   SendHorizontal,
   X,
} from 'lucide-react';

type ExpertChatSession = {
  id: string;
  expertId: string;
  title: string;
  messages: StreamLine[];
  createdAt: number;
  updatedAt: number;
};

const EXPERT_SESSIONS_KEY = 'lil-buddy-expert-chat-sessions-v1';
const ACTIVE_EXPERT_SESSION_KEY = 'lil-buddy-active-expert-chat-session-v1';
const EXPERT_WORKSPACES_KEY = 'lil-buddy-expert-workspaces-v1';

function applyThemeAccent(value?: string) {
  const safe = value || 'honey';
  document.documentElement.dataset.themeAccent = safe;
  document.documentElement.dataset.themeMode = safe === 'charcoal' ? 'dark' : 'light';
}
const DEFAULT_LIL_BUDDY_TELEGRAM_PROMPT =
  'You are Lil Buddy, the default helpful coding companion. Help the user clearly, stay practical, and keep them updated while working.';
const MAX_EXPERT_SESSIONS = 12;
const MAX_EXPERT_SESSION_MESSAGES = 250;

function expertSessionOwner(expert: LilExpert | null) {
  return expert?.id ?? 'default';
}

function expertSessionTitle(expert: LilExpert | null, index: number) {
  const base = expert?.name ?? 'Lil Buddy';
  return `${base} Chat ${index}`;
}

function loadExpertSessionStore(): Record<string, ExpertChatSession[]> {
  try {
    const parsed = JSON.parse(localStorage.getItem(EXPERT_SESSIONS_KEY) ?? '{}') as Record<string, ExpertChatSession[]>;
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch {
    return {};
  }
}

function saveExpertSessionStore(store: Record<string, ExpertChatSession[]>) {
  localStorage.setItem(EXPERT_SESSIONS_KEY, JSON.stringify(store));
}

function loadActiveExpertSessionMap(): Record<string, string> {
  try {
    const parsed = JSON.parse(localStorage.getItem(ACTIVE_EXPERT_SESSION_KEY) ?? '{}') as Record<string, string>;
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch {
    return {};
  }
}

function saveActiveExpertSessionMap(map: Record<string, string>) {
  localStorage.setItem(ACTIVE_EXPERT_SESSION_KEY, JSON.stringify(map));
}

function loadExpertWorkspaceMap(): Record<string, string> {
  try {
    const parsed = JSON.parse(localStorage.getItem(EXPERT_WORKSPACES_KEY) ?? '{}') as Record<string, string>;
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch {
    return {};
  }
}

function saveExpertWorkspaceMap(map: Record<string, string>) {
  localStorage.setItem(EXPERT_WORKSPACES_KEY, JSON.stringify(map));
}

function expertWorkspacePath(expert: LilExpert | null) {
  const owner = expertSessionOwner(expert);
  const path = loadExpertWorkspaceMap()[owner];
  return typeof path === 'string' ? path : '';
}

function setExpertWorkspacePath(expert: LilExpert | null, path: string) {
  const owner = expertSessionOwner(expert);
  const map = loadExpertWorkspaceMap();
  const trimmed = path.trim();

  if (trimmed) {
    map[owner] = trimmed;
  } else {
    delete map[owner];
  }

  saveExpertWorkspaceMap(map);
}

function makeExpertSession(expert: LilExpert | null, messages: StreamLine[], index = 1): ExpertChatSession {
  const now = Date.now();
  return {
    id: `session-${now}-${Math.random().toString(36).slice(2, 8)}`,
    expertId: expertSessionOwner(expert),
    title: expertSessionTitle(expert, index),
    messages,
    createdAt: now,
    updatedAt: now,
  };
}

function renumberExpertSessions(expert: LilExpert | null, sessions: ExpertChatSession[]) {
  return sessions.map((session, index) => ({
    ...session,
    title: expertSessionTitle(expert, index + 1),
  }));
}

function coerceExpertSessions(value: unknown): ExpertChatSession[] {
  if (!Array.isArray(value)) return [];

  return value
    .filter((item): item is ExpertChatSession => {
      const candidate = item as Partial<ExpertChatSession>;
      return Boolean(candidate.id && candidate.expertId && candidate.title && Array.isArray(candidate.messages));
    })
    .slice(0, MAX_EXPERT_SESSIONS)
    .map((session) => ({
      ...session,
      messages: session.messages.slice(-MAX_EXPERT_SESSION_MESSAGES),
    }));
}


export function ChatWindow() {
  const [restoreTarget, setRestoreTarget] = useState<RestoreTarget>(null);
  const [selectedDiffFile, setSelectedDiffFile] = useState<string | null>(null);
  const [workspaceOpen, setWorkspaceOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [onboardingOpen, setOnboardingOpen] = useState(false);
  const [userName, setUserNameState] = useState('');
  const [pendingApproval, setPendingApproval] = useState<PendingCommandApproval | null>(null);
  const [pendingPrompt, setPendingPrompt] = useState<string | null>(null);
  const [pendingExternalDirectory, setPendingExternalDirectory] = useState<string | null>(null);
  const [activeExpert, setActiveExpert] = useState<LilExpert | null>(() => selectedExpert());
  const [chatSessions, setChatSessions] = useState<ExpertChatSession[]>([]);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [deletedSessionIds, setDeletedSessionIds] = useState<Set<string>>(() => new Set());
  const [bottomMenuOpen, setBottomMenuOpen] = useState(false);
  const bottomMenuRef = useRef<HTMLDivElement | null>(null);


  const {
    activeTab,
    advancedOpen,
    commandPaletteOpen,
    prompt,
    commandPreview,
    workspacePathDraft,
    selectedProvider,
    opencodeGoMode,
    providers: providerStatuses,
    messages,
    running,
    desktop,
    workspace,
    workspaceDiff,
    setActiveTab,
    setAdvancedOpen,
    setCommandPaletteOpen,
    setPrompt,
    setCommandPreview,
    setWorkspacePathDraft,
    setSelectedProvider,
    setOpenCodeGoMode,
    setProviders,
    setDesktop,
    setWorkspace,
    setWorkspaceDiff,
    pushMessage,
    setMessages,
    clearMessages,
    setRunning,
    seedBoot,
  } = useAppStore();

  useEffect(() => {
    function handlePointerDown(event: MouseEvent) {
      if (!bottomMenuRef.current?.contains(event.target as Node)) setBottomMenuOpen(false);
    }

    document.addEventListener('mousedown', handlePointerDown);
    return () => document.removeEventListener('mousedown', handlePointerDown);
  }, []);

  useEffect(() => {
    if (messages.length === 0) seedBoot();
  }, [messages.length, seedBoot]);

  useEffect(() => {
    const workspacePath = expertWorkspacePath(activeExpert);
    saveTelegramActiveExpert(
      activeExpert?.id ?? 'default-lil-buddy',
      activeExpert?.name ?? 'Lil Buddy',
      activeExpert?.role ?? 'Default Lil Buddy',
      activeExpert?.systemPrompt ?? DEFAULT_LIL_BUDDY_TELEGRAM_PROMPT,
      workspacePath,
    ).catch(() => {});
  }, [activeExpert?.id]);

  async function syncTelegramExpertRegistry() {
    const experts = loadExperts();
    const workspaceMap = loadExpertWorkspaceMap();
    const payload = experts.map((expert) => ({
      ...expert,
      workspacePath: workspaceMap[expert.id] ?? '',
    }));
    await saveTelegramExperts(JSON.stringify(payload)).catch(() => {});
  }

  const validProviderIds = ['opencode-go', 'claude'];
  const validOpenCodeModes = ['run-json', 'run-formatted', 'run-stdin', 'raw-arg'];

  function safeProvider(provider?: string) {
    return provider && validProviderIds.includes(provider) ? provider : 'opencode-go';
  }

  function safeOpenCodeMode(mode?: string) {
    return mode && validOpenCodeModes.includes(mode) ? mode : 'run-json';
  }
  function eventTs(offset = 0) {
    return String(Date.now() + offset);
  }

  function extractExternalDirectoryRequest(text: string) {
    const match =
      text.match(/permission requested:\s*external_directory\s*\(([^)]+)\)/i) ??
      text.match(/external_directory\s*\(([^)]+)\)/i);

    return match?.[1]
      ?.replace(/\*+$/g, '')
      .replace(/\/+$/g, '')
      .trim() ?? null;
  }

  function greetingLine(name?: string) {
    const trimmed = (name ?? '').trim();

    if (trimmed) {
      return `Hey ${trimmed}, I’m Lil Buddy. I’m ready to inspect your repo, explain files, run safe checks, and keep you updated while I work.`;
    }

    return 'Hey, I’m Lil Buddy. I’m ready to inspect your repo, explain files, run safe checks, and keep you updated while I work.';
  }

  function expertBootMessages(expert: LilExpert) {
    return [
      {
        kind: 'assistant' as const,
        text: expertGreeting(expert),
        ts: String(Date.now()),
      },
    ];
  }

  function bootMessages(name?: string) {
    return [
      {
        kind: 'status' as const,
        text: 'Lil Buddy ready. File diff tracking loaded.',
        ts: String(Date.now()),
      },
      {
        kind: 'assistant' as const,
        text: greetingLine(name),
        ts: String(Date.now() + 1),
      },
    ];
  }

  useEffect(() => {
    loadSettings()
      .then((settings) => {
        const provider = safeProvider(settings.selected_provider);
        const mode = 'run-json';
        setSelectedProvider(provider);
        setOpenCodeGoMode(mode);
        saveOpenCodeGoMode(mode).catch(() => {});

        const nextUserName = (settings.user_name ?? '').trim();
        setUserNameState(nextUserName);
        setOnboardingOpen(!settings.onboarding_complete || !nextUserName);
        applyThemeAccent(settings.theme_accent);

        if (messages.length === 0) {
          loadSessionsForExpert(activeExpert, nextUserName);
        }

        const expertPath = expertWorkspacePath(activeExpert);
        setWorkspacePathDraft(expertPath);
        // Do not auto-run workspace detection/diff on app boot.
        // Workspace detection is now explicit only: user chooses/refreshes workspace.
      })
      .catch(() => {});
  }, [setMessages, setOpenCodeGoMode, setSelectedProvider, setWorkspacePathDraft]);


  function loadSessionsForExpert(expert: LilExpert | null, name = userName) {
    const owner = expertSessionOwner(expert);
    const store = loadExpertSessionStore();
    const activeMap = loadActiveExpertSessionMap();
    let sessions = renumberExpertSessions(expert, coerceExpertSessions(store[owner]));

    if (sessions.length === 0) {
      const created = makeExpertSession(expert, expert ? expertBootMessages(expert) : bootMessages(name), 1);
      sessions = [created];
      store[owner] = sessions;
      saveExpertSessionStore(store);
      activeMap[owner] = created.id;
      saveActiveExpertSessionMap(activeMap);
    } else {
      store[owner] = sessions;
      saveExpertSessionStore(store);
    }

    const activeId = sessions.some((session) => session.id === activeMap[owner])
      ? activeMap[owner]
      : sessions[0].id;
    const activeSession = sessions.find((session) => session.id === activeId) ?? sessions[0];

    setChatSessions(sessions);
    setActiveSessionId(activeSession.id);
    setMessages(activeSession.messages);
  }

  function persistActiveSession(nextMessages = messages) {
    if (!activeSessionId || deletedSessionIds.has(activeSessionId)) return;

    const owner = expertSessionOwner(activeExpert);
    const store = loadExpertSessionStore();
    const sessions = coerceExpertSessions(store[owner]);
    const now = Date.now();

    const nextSessions = sessions.map((session) =>
      session.id === activeSessionId
        ? { ...session, messages: nextMessages, updatedAt: now }
        : session,
    );

    store[owner] = nextSessions;
    saveExpertSessionStore(store);
    setChatSessions(nextSessions);
  }


  useEffect(() => {
    if (!activeSessionId || messages.length === 0 || deletedSessionIds.has(activeSessionId)) return;

    const timeout = window.setTimeout(() => {
      persistActiveSession(messages);
    }, 500);

    return () => window.clearTimeout(timeout);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeSessionId, messages]);

  async function switchChatSession(sessionId: string) {
    if (sessionId === activeSessionId) return;

    persistActiveSession(messages);

    const owner = expertSessionOwner(activeExpert);
    const session = chatSessions.find((item) => item.id === sessionId);
    if (!session) return;

    const activeMap = loadActiveExpertSessionMap();
    activeMap[owner] = session.id;
    saveActiveExpertSessionMap(activeMap);

    setActiveSessionId(session.id);
    setMessages(session.messages);
  }

  function createChatSession() {
    persistActiveSession(messages);

    const owner = expertSessionOwner(activeExpert);
    const store = loadExpertSessionStore();
    const sessions = coerceExpertSessions(store[owner]);
    const nextSession = makeExpertSession(activeExpert, activeExpert ? expertBootMessages(activeExpert) : bootMessages(userName), sessions.length + 1);
    const nextSessions = renumberExpertSessions(activeExpert, [...sessions, nextSession]);

    store[owner] = nextSessions;
    saveExpertSessionStore(store);

    const activeMap = loadActiveExpertSessionMap();
    activeMap[owner] = nextSession.id;
    saveActiveExpertSessionMap(activeMap);

    setChatSessions(nextSessions);
    setActiveSessionId(nextSession.id);
    setMessages(nextSession.messages);
  }

  function deleteChatSession(sessionId: string) {
    const owner = expertSessionOwner(activeExpert);
    const store = loadExpertSessionStore();
    const sessions = coerceExpertSessions(store[owner]);
    const nextSessions = renumberExpertSessions(activeExpert, sessions.filter((session) => session.id !== sessionId));
    const nextDeleted = new Set(deletedSessionIds);
    nextDeleted.add(sessionId);
    setDeletedSessionIds(nextDeleted);

    if (nextSessions.length === 0) {
      const replacement = makeExpertSession(activeExpert, activeExpert ? expertBootMessages(activeExpert) : bootMessages(userName), 1);
      store[owner] = [replacement];
      saveExpertSessionStore(store);

      const activeMap = loadActiveExpertSessionMap();
      activeMap[owner] = replacement.id;
      saveActiveExpertSessionMap(activeMap);

      setChatSessions([replacement]);
      setActiveSessionId(replacement.id);
      setMessages(replacement.messages);
      return;
    }

    store[owner] = nextSessions;
    saveExpertSessionStore(store);
    setChatSessions(nextSessions);

    if (activeSessionId === sessionId) {
      const nextActive = nextSessions[0];
      const activeMap = loadActiveExpertSessionMap();
      activeMap[owner] = nextActive.id;
      saveActiveExpertSessionMap(activeMap);

      setActiveSessionId(nextActive.id);
      setMessages(nextActive.messages);
    }
  }

  async function completeOnboarding(name: string) {
    const trimmed = name.trim();
    setUserNameState(trimmed);
    setOnboardingOpen(false);

    if (!activeSessionId) {
      const messages = bootMessages(trimmed);
      setMessages(messages);
    }
  }

  // Release safety: workspace detection/diff is explicit only.
  // Do not call this on app boot or expert switch; users must refresh/select a workspace intentionally.
  async function refreshWorkspace() {
    const info = await detectWorkspace();
    setWorkspace(info);

    const diff = await getWorkspaceDiff();
    setWorkspaceDiff(diff);
  }

  useEffect(() => {
    syncTelegramExpertRegistry().catch(() => {});
    detectProviders().then(setProviders).catch((error) => {
      pushMessage({
        kind: 'stderr',
        text: `provider detection failed: ${String(error)}`,
        ts: new Date().toISOString(),
      });
    });

    detectDesktop().then(setDesktop).catch(() => {});
  }, [pushMessage, setProviders, setDesktop]);


  // autosave current workspace session through Rust/Tauri
  useEffect(() => {
    if (!workspace?.path || messages.length === 0) return;

    const timeout = window.setTimeout(() => {
      const parts = workspace.path?.split('/').filter(Boolean) ?? [];
      const name = parts[parts.length - 1] || 'Workspace';
      saveWorkspaceSession(name, workspace.path!, messages).catch(() => {});
    }, 800);

    return () => window.clearTimeout(timeout);
  }, [workspace?.path, messages]);
  useEffect(() => {
    let detach: (() => void) | undefined;

    attachRuntimeStream((event) => {
      const externalDirectory = extractExternalDirectoryRequest(event.text);

      if (externalDirectory) {
        setRunning(false);
        setPendingExternalDirectory(externalDirectory);
        setPendingPrompt((current) => current ?? messages[messages.length - 1]?.text ?? null);
        setPendingApproval({
          id: `external-directory-${Date.now()}`,
          command: externalDirectory,
          risk_level: 'confirm',
          reason: 'OpenCode needs permission to read a path outside the current working directory.',
        });
        pushMessage({
          kind: 'command-approval',
          text: `pending=external_directory · risk=confirm · command=${externalDirectory} · reason=OpenCode needs permission to access this external directory.`,
          ts: String(Date.now()),
        });
        return;
      }

      pushMessage(event);

      if (event.kind === 'exit') {
        setRunning(false);
        return;
      }

    }).then((unlisten) => {
      detach = unlisten;
    });

    return () => detach?.();
  }, [messages, pushMessage, setRunning]);

  async function selectExpert(expert: LilExpert | null) {
    try {
      // Save the tab we are leaving, then switch sessions/workspace in one controlled pass.
      // Default Lil Buddy has no explicit workspace unless the user sets one, so do not run heavy
      // workspace detection against the OS home directory while switching experts.
      persistActiveSession(messages);

      const owner = expertSessionOwner(expert);
      const store = loadExpertSessionStore();
      const activeMap = loadActiveExpertSessionMap();
      let sessions = renumberExpertSessions(expert, coerceExpertSessions(store[owner]));

      if (sessions.length === 0) {
        const created = makeExpertSession(expert, expert ? expertBootMessages(expert) : bootMessages(userName), 1);
        sessions = [created];
        store[owner] = sessions;
        activeMap[owner] = created.id;
        saveExpertSessionStore(store);
        saveActiveExpertSessionMap(activeMap);
      }

      const activeId = sessions.some((session) => session.id === activeMap[owner])
        ? activeMap[owner]
        : sessions[0].id;
      const activeSession = sessions.find((session) => session.id === activeId) ?? sessions[0];

      const expertPath = expertWorkspacePath(expert);

      setActiveExpert(expert);
      setChatSessions(sessions);
      setActiveSessionId(activeSession.id);
      setMessages(activeSession.messages);
      setWorkspacePathDraft(expertPath);

      // Do not auto-run workspace detection/diff while switching experts.
      // Keep the selected workspace draft and only sync Telegram context.
      await syncTelegramExpertRegistry();
      await saveTelegramActiveExpert(
        expert?.id ?? 'default-lil-buddy',
        expert?.name ?? 'Lil Buddy',
        expert?.role ?? 'Default Lil Buddy',
        expert?.systemPrompt ?? DEFAULT_LIL_BUDDY_TELEGRAM_PROMPT,
        expertPath,
      ).catch(() => {});
    } catch (error) {
      pushMessage({
        kind: 'stderr',
        text: `expert switch failed: ${String(error)}`,
        ts: String(Date.now()),
      });
    }
  }

  async function changeProvider(providerId: string) {
    const provider = safeProvider(providerId);
    setSelectedProvider(provider);
    await saveSelectedProvider(provider);
  }

  async function changeOpenCodeMode(mode: string) {
    const safeMode = safeOpenCodeMode(mode);
    setOpenCodeGoMode(safeMode);
    await saveOpenCodeGoMode(safeMode);
  }


  async function switchWorkspaceFromTitle(path: string, nextMessages: typeof messages) {
    setExpertWorkspacePath(activeExpert, path);
    await saveWorkspacePath(path);
    setWorkspacePathDraft(path);
    setMessages(nextMessages);
    // Workspace detection/diff is intentionally not automatic.
    await syncTelegramExpertRegistry();
    await saveTelegramActiveExpert(
      activeExpert?.id ?? 'default-lil-buddy',
      activeExpert?.name ?? 'Lil Buddy',
      activeExpert?.role ?? 'Default Lil Buddy',
      activeExpert?.systemPrompt ?? DEFAULT_LIL_BUDDY_TELEGRAM_PROMPT,
      path,
    ).catch(() => {});
  }

  function saveCurrentWorkspaceName(_name: string) {
  }

  async function saveWorkspace() {
    setExpertWorkspacePath(activeExpert, workspacePathDraft);
    await saveWorkspacePath(workspacePathDraft);
    // Workspace detection/diff is intentionally not automatic.
    await syncTelegramExpertRegistry();
    await saveTelegramActiveExpert(
      activeExpert?.id ?? 'default-lil-buddy',
      activeExpert?.name ?? 'Lil Buddy',
      activeExpert?.role ?? 'Default Lil Buddy',
      activeExpert?.systemPrompt ?? DEFAULT_LIL_BUDDY_TELEGRAM_PROMPT,
      workspacePathDraft,
    ).catch(() => {});
  }

  async function chooseWorkspace() {
    const selected = await chooseWorkspaceFolder();

    if (selected) {
      setExpertWorkspacePath(activeExpert, selected);
      setWorkspacePathDraft(selected);
      await saveWorkspacePath(selected);
      // Workspace detection/diff is intentionally not automatic.
      await syncTelegramExpertRegistry();
      await saveTelegramActiveExpert(
        activeExpert?.id ?? 'default-lil-buddy',
        activeExpert?.name ?? 'Lil Buddy',
        activeExpert?.role ?? 'Default Lil Buddy',
        activeExpert?.systemPrompt ?? DEFAULT_LIL_BUDDY_TELEGRAM_PROMPT,
        selected,
      ).catch(() => {});
    }
  }

  async function openTerminalHere() {
    try {
      await openWorkspaceTerminal();
    } catch (error) {
      pushMessage({
        kind: 'stderr',
        text: `open terminal failed: ${String(error)}`,
        ts: new Date().toISOString(),
      });
    }
  }

  async function refreshDiffs() {
    await emitWorkspaceDiffEvents().catch(() => {});
    const diff = await getWorkspaceDiff();
    setWorkspaceDiff(diff);
  }

  async function restoreSelectedTarget() {
    if (!restoreTarget) return;

    try {
      if (restoreTarget.kind === 'all') {
        await restoreAllWorkspaceFiles();
      } else {
        await restoreWorkspaceFile(restoreTarget.path);
      }

      setRestoreTarget(null);
      setSelectedDiffFile(null);
      await refreshDiffs();
      await refreshWorkspace();
    } catch (error) {
      pushMessage({
        kind: 'stderr',
        text: `restore failed: ${String(error)}`,
        ts: new Date().toISOString(),
      });
    }
  }


  function extractPromptCommand(value: string) {
    const fenced = value.match(/```(?:bash|sh|zsh)?\n([\s\S]*?)```/i);
    if (fenced?.[1]) return fenced[1].trim();

    const inline = value.match(/(?:run|execute|use command|command:)\s+([^\n]+)/i);
    if (inline?.[1]) return inline[1].trim();

    const risky = value.match(/\b(rm\s+-rf\s+[^\n]+|sudo\s+[^\n]+|chmod\s+-R\s+[^\n]+|chown\s+-R\s+[^\n]+)\b/i);
    if (risky?.[1]) return risky[1].trim();

    return null;
  }

  async function runPrompt(trimmed: string) {
    const provider = safeProvider(selectedProvider);
    const mode = safeOpenCodeMode(opencodeGoMode);

    if (provider !== selectedProvider) {
      setSelectedProvider(provider);
      await saveSelectedProvider(provider);
    }

    if (mode !== opencodeGoMode) {
      setOpenCodeGoMode(mode);
      await saveOpenCodeGoMode(mode);
    }

    setRunning(true);
    setNativeCompanionCategory('work').catch(() => {});
    setPrompt('');
    setAdvancedOpen(false);

    const startedAt = Date.now();

    pushMessage({
      kind: 'user-message',
      text: trimmed,
      ts: String(startedAt),
    });


    try {
      const providerPrompt = `${expertPromptPrefix(activeExpert)}${trimmed}`;
      const preview = await previewProviderCommand(provider, providerPrompt);
      setCommandPreview(preview);

      await runProviderCommand(provider, providerPrompt);
    } catch (error) {
      setRunning(false);
      pushMessage({
        kind: 'stderr',
        text: `run failed: ${String(error)}`,
        ts: String(Date.now()),
      });
    }
  }

  async function approvePendingCommand() {
    if (!pendingApproval) return;

    if (pendingExternalDirectory) {
      try {
        const pattern = await allowExternalDirectory(pendingExternalDirectory);
        pushMessage({
          kind: 'status',
          text: `Allowed external directory: ${pattern}`,
          ts: String(Date.now()),
        });
      } catch (error) {
        pushMessage({
          kind: 'stderr',
          text: `failed to allow external directory: ${String(error)}`,
          ts: String(Date.now()),
        });
      }

      const promptToRun = pendingPrompt;
      setPendingApproval(null);
      setPendingPrompt(null);
      setPendingExternalDirectory(null);

      if (promptToRun) {
        await runPrompt(promptToRun);
      }

      return;
    }

    if (!pendingPrompt) return;

    await approveCommandApproval(pendingApproval.id);
    const promptToRun = pendingPrompt;
    setPendingApproval(null);
    setPendingPrompt(null);
    await runPrompt(promptToRun);
  }

  async function denyPendingCommand() {
    if (pendingApproval) {
      await denyCommandApproval(pendingApproval.id);
    }

    setPendingApproval(null);
    setPendingPrompt(null);
  }

  async function sendMessage() {
    const trimmed = prompt.trim();
    if (!trimmed || running) return;

    const candidateCommand = extractPromptCommand(trimmed);

    if (candidateCommand) {
      const risk = await classifyCommandRisk(candidateCommand);

      if (risk.level !== 'allow') {
        const approval = await requestCommandApproval(
          candidateCommand,
          risk.reason,
          risk.level,
        );

        setPendingApproval(approval);
        setPendingPrompt(trimmed);
        return;
      }
    }

    await runPrompt(trimmed);
  }

  async function stop() {
    await stopProviderCommand();
    setRunning(false);
    await refreshWorkspace().catch(() => {});
  }

  async function closeChatSafely() {
    // Hide the chat only. Provider runs must continue in the background so
    // Lil Buddy can notify through the native companion when a reply arrives.
    await hideChatWindow();
  }

  async function minimizeChat() {
    await getCurrentWindow().minimize();
  }

  async function toggleMaximizeChat() {
    const window = getCurrentWindow();
    const maximized = await window.isMaximized();

    if (maximized) {
      await window.unmaximize();
      return;
    }

    await window.maximize();
  }

  function handleHeaderMouseDown(event: React.MouseEvent<HTMLDivElement>) {
    if (event.button !== 0) return;

    const target = event.target as HTMLElement | null;
    if (target?.closest('[data-tauri-drag-region="false"]')) {
      return;
    }

    getCurrentWindow().startDragging().catch(() => {});
  }

  const selectedInstalled = providerStatuses.some((p) => p.id === safeProvider(selectedProvider) && p.installed);

  return (
    <div className="lm-chat-stage">
      <QuickActions
        open={commandPaletteOpen}
        onClose={() => setCommandPaletteOpen(false)}
        onPick={setPrompt}
      />

      <RestoreConfirm
        target={restoreTarget}
        onCancel={() => setRestoreTarget(null)}
        onConfirm={restoreSelectedTarget}
      />

      {/* Release safety: risky commands must stay visible before execution. */}
      <ApprovalConfirm
        approval={pendingApproval}
        onApprove={approvePendingCommand}
        onDeny={denyPendingCommand}
      />

      <SettingsPanel
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        onProviderChange={changeProvider}
        onModeChange={changeOpenCodeMode}
        onWorkspaceChange={(path) => {
          setWorkspacePathDraft(path);
              }}
        onUserNameChange={(name) => {
          setUserNameState(name);
          if (messages.length <= 2) {
            setMessages(bootMessages(name));
          }
        }}
      />

      <OnboardingModal
        open={onboardingOpen}
        initialName={userName}
        canSkip
        onClose={() => setOnboardingOpen(false)}
        onComplete={completeOnboarding}
      />


      <WorkspaceModal
        open={workspaceOpen}
        onClose={() => setWorkspaceOpen(false)}
        desktop={desktop}
        workspace={workspace}
        workspaceDiff={workspaceDiff}
        workspacePathDraft={workspacePathDraft}
        messages={messages}
        selectedDiffFile={selectedDiffFile}
        onWorkspacePathChange={setWorkspacePathDraft}
        onSaveWorkspace={saveWorkspace}
        onChooseWorkspace={chooseWorkspace}
        onOpenTerminal={openTerminalHere}
        onRefreshWorkspace={refreshWorkspace}
        onRefreshDiffs={refreshDiffs}
        onViewDiff={setSelectedDiffFile}
        onRestore={(path) => setRestoreTarget({ kind: 'file', path })}
        onRestoreAll={() => setRestoreTarget({ kind: 'all' })}
      />

      <div className="lm-chat-card chat-first">
        <div className="lm-card-topline" />

        <div className="lm-header" onMouseDown={handleHeaderMouseDown}>
          <div className="lm-header-top-actions" data-tauri-drag-region="false">
            <button type="button" className="lm-icon-btn" aria-label="Minimize chat" onClick={minimizeChat}>
              <Minus />
            </button>
            <button type="button" className="lm-icon-btn" aria-label="Maximize or restore chat" onClick={toggleMaximizeChat}>
              <Maximize2 />
            </button>
          <button type="button" className="lm-icon-btn" aria-label="Close Chat" onClick={closeChatSafely}>
            <X />
          </button>
          </div>

          <div className="lm-header-brand">
            <img className="lm-app-logo" src={lilBuddyLogo} alt="Lil Buddy" draggable={false} />
            <ExpertsPanel selectedId={activeExpert?.id ?? null} onSelect={(expert) => { selectExpert(expert).catch(() => {}); }} />
          </div>
        </div>

        <div className="lm-chat-session-tabs">
          <div className="lm-chat-session-tab-list">
            {chatSessions.map((session) => (
              <div
                key={session.id}
                role="button"
                tabIndex={0}
                className={session.id === activeSessionId ? 'lm-chat-session-tab active' : 'lm-chat-session-tab'}
                onClick={() => switchChatSession(session.id)}
                onKeyDown={(event) => {
                  if (event.key === 'Enter' || event.key === ' ') {
                    event.preventDefault();
                    switchChatSession(session.id);
                  }
                }}
                title={session.title}
              >
                <span>{session.title}</span>
                <small>{session.messages.length}</small>
                <button
                  type="button"
                  className="lm-chat-session-close"
                  onClick={(event) => {
                    event.stopPropagation();
                    deleteChatSession(session.id);
                  }}
                  aria-label={`Delete ${session.title}`}
                >
                  ×
                </button>
              </div>
            ))}
          </div>
          <button className="lm-chat-session-new" type="button" onClick={createChatSession}>
            ＋ New chat
          </button>
        </div>

        <div className="lm-message-scroll chat-first-scroll">
          {messages.length <= 2 ? <PromptChips onPick={setPrompt} /> : null}

          <ChatMessages messages={messages} running={running} expert={activeExpert} />

          <RuntimePanel
            open={advancedOpen}
            activeTab={activeTab}
            messages={messages}
            onTabChange={setActiveTab}
          />
        </div>

        <div className="lm-input-wrap">
          <ModelProviderSelector
          selectedProvider={safeProvider(selectedProvider)}
          opencodeGoMode={safeOpenCodeMode(opencodeGoMode)}
          onProviderChange={changeProvider}
          onModeChange={changeOpenCodeMode}
        />

          <button
            className={workspace?.path ? 'lm-folder-pill has-workspace' : 'lm-folder-pill'}
            onClick={chooseWorkspace}
            title={workspace?.path ? `Workspace: ${workspace.path}` : 'Choose workspace folder'}
          >
            <span><Paperclip /></span>
            <strong>{workspace?.path ? workspace.path.split('/').filter(Boolean).slice(-1)[0] : 'Folder'}</strong>
          </button>

          <div className="lm-bottom-menu-wrap" ref={bottomMenuRef}>
            <button
              type="button"
              className="lm-icon-btn lm-bottom-menu-btn"
              aria-label="Open workspace and quick actions"
              onClick={() => setBottomMenuOpen((open) => !open)}
            >
              <MoreHorizontal />
            </button>

            {bottomMenuOpen ? (
              <div className="lm-bottom-menu-popover">
                <button
                  type="button"
                  className="lm-bottom-menu-item"
                  onClick={() => {
                    setWorkspaceOpen(true);
                    setBottomMenuOpen(false);
                  }}
                >
                  <span className="lm-bottom-menu-item-icon">⌂</span>
                  <strong>Workspace info</strong>
                  <small>{workspaceDiff?.changed_files.length ?? 0} changed</small>
                </button>
                <button
                  type="button"
                  className="lm-bottom-menu-item"
                  onClick={() => {
                    setCommandPaletteOpen(true);
                    setBottomMenuOpen(false);
                  }}
                >
                  <span className="lm-bottom-menu-item-icon"><NotepadTextDashed /></span>
                  <strong>Quick actions</strong>
                  <small>Open command palette</small>
                </button>
                  <button
                  type="button"
                  className="lm-bottom-menu-item"
                  onClick={() => {
                    setSettingsOpen(true);
                    setBottomMenuOpen(false);
                  }}
                >
                  <span className="lm-bottom-menu-item-icon"><Settings /></span>
                  <strong>Settings</strong>
                  <small>Theme, provider, companion</small>
                </button>
              </div>
            ) : null}
          </div>

          <textarea
            className="lm-input lm-chat-textarea"
            value={prompt}
            rows={1}
            onChange={(event) => {
              setPrompt(event.target.value);
              event.currentTarget.style.height = 'auto';
              event.currentTarget.style.height = `${Math.min(event.currentTarget.scrollHeight, 150)}px`;
            }}
            placeholder="Message Lil Buddy..."
            onKeyDown={(event) => {
              if (event.key === 'Enter' && !event.shiftKey) {
                event.preventDefault();
                sendMessage();
              }

              if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
                setCommandPaletteOpen(true);
              }
            }}
          />

          {running ? (
            <button className="lm-stop" onClick={stop}>■</button>
          ) : (
            <button className="lm-send" onClick={sendMessage}><SendHorizontal/></button>
          )}
        </div>


        <ResizeHandle />
      </div>
    </div>
  );
}
