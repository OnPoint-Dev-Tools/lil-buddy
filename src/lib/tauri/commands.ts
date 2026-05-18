import { invoke } from '@tauri-apps/api/core';
import type { DesktopEnvironment, ProviderStatus } from '../providers/types';

export type AppSettings = {
  companion_x: number | null;
  companion_y: number | null;
  chat_offset_x: number;
  chat_offset_y: number;
  selected_provider: string;
  opencode_go_mode: string;
  opencode_command: string;
  opencode_provider_key: string;
  claude_command: string;
  claude_output_format: string;
  workspace_path: string | null;
  default_directory: string | null;
  selected_model: string | null;
  favorite_models: string[];
  safety_mode: string;
  theme_accent: string;
  launch_at_startup: boolean;
  user_name: string | null;
  onboarding_complete: boolean;
  companion_work_animation: string;
  companion_thinking_animation: string;
  companion_command_animation: string;
  companion_done_animation: string;
  companion_error_animation: string;
  companion_idle_wave_animation: string;
  companion_character_id: string;
  companion_size: string;
  telegram_gateway_enabled: boolean;
  telegram_gateway_mode: string;
  telegram_bot_token: string;
  telegram_webhook_public_url: string;
  telegram_webhook_local_port: number;
  telegram_webhook_path_secret: string;
  telegram_webhook_secret: string;
  telegram_allowed_chat_id: string | null;
  telegram_active_expert_id: string | null;
  telegram_active_expert_name: string | null;
  telegram_active_expert_role: string | null;
  telegram_active_expert_prompt: string | null;
  telegram_active_workspace_path: string | null;
  telegram_experts_json: string;
};

export type WorkspaceInfo = {
  path: string | null;
  is_git_repo: boolean;
  git_root: string | null;
  branch: string | null;
  dirty: boolean;
  status_summary: string;
};

export type WorkspaceDiff = {
  git_root: string | null;
  status_short: string;
  diff_stat: string;
  changed_files: string[];
  diffs: Array<{ path: string; diff: string }>;
};

export type CommandRisk = {
  level: string;
  reason: string;
  command: string;
};

export type PendingCommandApproval = {
  id: string;
  command: string;
  risk_level: string;
  reason: string;
};

export type WorkspaceSession = {
  id: string;
  name: string;
  path: string;
  messages: unknown;
  updated_at: number;
};

export type ProviderModel = {
  id: string;
  provider_id: string;
  name: string;
  description: string;
  command_model: string;
  source: string;
  caps: string[];
  starred: boolean;
};

export async function showChatWindow() {
  return invoke('show_chat_window');
}

export async function hideChatWindow() {
  return invoke('hide_chat_window');
}

export async function toggleChatWindow() {
  return invoke('toggle_chat_window');
}

export type NativeCompanionMood =
  | 'hello'
  | 'idle'
  | 'walk-left'
  | 'walk-right'
  | 'thinking'
  | 'working'
  | 'command'
  | 'done'
  | 'error';

export async function startNativeCompanion() {
  return invoke('start_native_companion');
}

export async function stopNativeCompanion() {
  return invoke('stop_native_companion');
}

export async function showNativeCompanion() {
  return invoke('show_native_companion');
}

export async function hideNativeCompanion() {
  return invoke('hide_native_companion');
}

export async function setNativeCompanionMood(mood: NativeCompanionMood) {
  return invoke('set_native_companion_mood', { mood });
}

export type NativeCompanionCategory = 'work' | 'thinking' | 'command' | 'done' | 'error' | 'idle-wave';

export async function setNativeCompanionPose(pose: NativeCompanionMood) {
  return invoke('set_native_companion_pose', { pose });
}

export async function setNativeCompanionCategory(category: NativeCompanionCategory) {
  return invoke('set_native_companion_category', { category });
}

export async function walkNativeCompanion(direction: 'left' | 'right', distance: number, durationMs: number) {
  return invoke('walk_native_companion', {
    direction,
    distance: Math.round(distance),
    durationMs: Math.round(durationMs),
  });
}


export async function loadSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('load_settings');
}

export async function saveSelectedProvider(providerId: string) {
  return invoke('save_selected_provider', { providerId });
}

export async function saveOpenCodeGoMode(mode: string) {
  return invoke('save_opencode_go_mode', { mode });
}

export async function saveOpenCodeCommand(command: string) {
  return invoke('save_opencode_command', { command });
}

export async function saveOpenCodeProviderKey(providerKey: string) {
  return invoke('save_opencode_provider_key', { providerKey });
}

export async function saveClaudeCommand(command: string) {
  return invoke('save_claude_command', { command });
}

export async function saveClaudeOutputFormat(outputFormat: string) {
  return invoke('save_claude_output_format', { outputFormat });
}

export async function saveWorkspacePath(path: string) {
  return invoke('save_workspace_path', { path });
}

export async function saveDefaultDirectory(path: string) {
  return invoke('save_default_directory', { path });
}

export async function chooseWorkspaceFolder(): Promise<string | null> {
  return invoke<string | null>('choose_workspace_folder');
}

export async function openWorkspaceTerminal() {
  return invoke('open_workspace_terminal');
}

export async function detectDesktop(): Promise<DesktopEnvironment> {
  return invoke<DesktopEnvironment>('detect_desktop_environment');
}

export async function detectWorkspace(): Promise<WorkspaceInfo> {
  return invoke<WorkspaceInfo>('detect_workspace');
}

export async function getWorkspaceDiff(): Promise<WorkspaceDiff> {
  return invoke<WorkspaceDiff>('get_workspace_diff');
}

export async function emitWorkspaceDiffEvents() {
  return invoke('emit_workspace_diff_events');
}

export async function startWorkspaceWatch() {
  return invoke('start_workspace_watch');
}

export async function stopWorkspaceWatch() {
  return invoke('stop_workspace_watch');
}

export async function restoreWorkspaceFile(path: string) {
  return invoke('restore_workspace_file', { path });
}

export async function restoreAllWorkspaceFiles() {
  return invoke('restore_all_workspace_files');
}

export async function classifyCommandRisk(command: string): Promise<CommandRisk> {
  return invoke<CommandRisk>('classify_command_risk', { command });
}

export async function detectProviders(): Promise<ProviderStatus[]> {
  return invoke<ProviderStatus[]>('detect_providers');
}

export async function previewProviderCommand(providerId: string, prompt: string) {
  return invoke<string>('preview_provider_command', { providerId, prompt });
}

export async function runProviderCommand(providerId: string, prompt: string) {
  return invoke('run_provider_command', { providerId, prompt });
}

export async function stopProviderCommand() {
  return invoke('stop_provider_command');
}

export async function providerIsRunning(): Promise<boolean> {
  return invoke<boolean>('provider_is_running');
}

export async function listProviderModels(providerId: string, refresh = false): Promise<ProviderModel[]> {
  return invoke<ProviderModel[]>('list_provider_models', { providerId, refresh });
}

export async function saveSelectedModel(model: string | null) {
  return invoke('save_selected_model', { model });
}

export async function saveFavoriteModels(models: string[]) {
  return invoke('save_favorite_models', { models });
}


export async function saveSafetyMode(safetyMode: string) {
  return invoke('save_safety_mode', { safetyMode });
}

export async function saveThemeAccent(themeAccent: string) {
  return invoke('save_theme_accent', { themeAccent });
}

export async function saveLaunchAtStartup(launchAtStartup: boolean) {
  return invoke('save_launch_at_startup', { launchAtStartup });
}

export async function saveUserName(userName: string) {
  return invoke('save_user_name', { userName });
}

export async function saveOnboardingComplete(complete: boolean) {
  return invoke('save_onboarding_complete', { complete });
}

export async function saveCompanionCharacter(characterId: string) {
  return invoke('save_companion_character', { characterId });
}

export async function saveCompanionSize(companionSize: string) {
  return invoke('save_companion_size', { companionSize });
}

export async function saveCompanionAnimationSettings(settings: {
  workAnimation: NativeCompanionMood;
  thinkingAnimation: NativeCompanionMood;
  commandAnimation: NativeCompanionMood;
  doneAnimation: NativeCompanionMood;
  errorAnimation: NativeCompanionMood;
  idleWaveAnimation: NativeCompanionMood;
}) {
  return invoke('save_companion_animation_settings', settings);
}


export async function requestCommandApproval(command: string, reason: string, riskLevel: string): Promise<PendingCommandApproval> {
  return invoke<PendingCommandApproval>('request_command_approval', { command, reason, riskLevel });
}

export async function approveCommandApproval(id: string) {
  return invoke('approve_command_approval', { id });
}


export async function allowExternalDirectory(path: string): Promise<string> {
  return invoke<string>('allow_external_directory', { path });
}

export async function denyCommandApproval(id: string) {
  return invoke('deny_command_approval', { id });
}


export async function listWorkspaceSessions(): Promise<WorkspaceSession[]> {
  return invoke<WorkspaceSession[]>('list_workspace_sessions');
}

export async function saveWorkspaceSession(name: string, path: string, messages: unknown): Promise<WorkspaceSession> {
  return invoke<WorkspaceSession>('save_workspace_session', { name, path, messages });
}

export async function loadWorkspaceSession(id: string): Promise<WorkspaceSession | null> {
  return invoke<WorkspaceSession | null>('load_workspace_session', { id });
}

export async function deleteWorkspaceSession(id: string) {
  return invoke('delete_workspace_session', { id });
}


export async function trayShowLilMan() {
  return invoke('tray_show_lil_buddy');
}

export async function trayHideLilMan() {
  return invoke('tray_hide_lil_buddy');
}

export async function trayOpenChat() {
  return invoke('tray_open_chat');
}

export async function trayQuit() {
  return invoke('tray_quit');
}

export async function restoreMonitorAwarePositions() {
  return invoke('restore_monitor_aware_positions');
}


export async function saveTelegramGatewaySettings(
  gatewayMode: string,
  botToken: string,
  allowedChatId: string,
  webhookPublicUrl: string,
  webhookLocalPort: number,
  webhookPathSecret: string,
  webhookSecret: string,
) {
  return invoke('save_telegram_gateway_settings', {
    gatewayMode,
    botToken,
    allowedChatId,
    webhookPublicUrl,
    webhookLocalPort,
    webhookPathSecret,
    webhookSecret,
  });
}

export async function saveTelegramActiveExpert(
  expertId: string,
  name: string,
  role: string,
  systemPrompt: string,
  workspacePath: string,
) {
  return invoke('save_telegram_active_expert', { expertId, name, role, systemPrompt, workspacePath });
}

export async function startTelegramGateway() {
  return invoke('start_telegram_gateway');
}

export async function stopTelegramGateway() {
  return invoke('stop_telegram_gateway');
}

export async function telegramGatewayRunning(): Promise<boolean> {
  return invoke<boolean>('telegram_gateway_running');
}

export async function saveTelegramExperts(expertsJson: string) {
  return invoke('save_telegram_experts', { expertsJson });
}
