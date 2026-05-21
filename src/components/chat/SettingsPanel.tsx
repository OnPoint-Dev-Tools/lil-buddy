import { useEffect, useState } from 'react';
import {
  AlertTriangle,
  Bot,
  Brain,
  CheckCircle2,
  Cloud,
  Eye,
  Flower2,
  Footprints,
  Hand,
  Leaf,
  Maximize2,
  MessageCircle,
  Minimize2,
  Moon,
  Palette,
  Radio,
  ShieldCheck,
  ShieldOff,
  Sparkles,
  Terminal,
  Webhook,
  WandSparkles,
  X,
  type LucideIcon,
} from 'lucide-react';
import {
  loadSettings,
  saveLaunchAtStartup,
  saveOpenCodeGoMode,
  saveClaudeCommand,
  saveClaudeOutputFormat,
  saveSafetyMode,
  saveSelectedProvider,
  saveThemeAccent,
  saveUserName,
  saveDefaultDirectory,
  saveCompanionAnimationSettings,
  saveCompanionCharacter,
  saveCompanionSize,
  saveTelegramGatewaySettings,
  startTelegramGateway,
  stopTelegramGateway,
  setNativeCompanionPose,
  type AppSettings,
  type NativeCompanionMood,
} from '../../lib/tauri/commands';
import { providers } from '../../lib/providers/registry';
import { SettingsDropdown } from './SettingsDropdown';
import { ProviderIcon } from './ProviderIcon';
import idlePreview from '../../assets/companions/tan-explorer-v2/idle/01.png';
import blinkPreview from '../../assets/companions/tan-explorer-v2/idle/03.png';
import helloPreview from '../../assets/companions/tan-explorer-v2/hello/03.png';
import workingPreview from '../../assets/companions/tan-explorer-v2/working/03.png';
import thinkingPreview from '../../assets/companions/tan-explorer-v2/thinking/03.png';
import commandPreview from '../../assets/companions/tan-explorer-v2/command/03.png';
import donePreview from '../../assets/companions/tan-explorer-v2/done/03.png';
import errorPreview from '../../assets/companions/tan-explorer-v2/error/03.png';
import { companionCharacters, getSavedCompanionCharacterId, saveCompanionCharacterId } from '../../lib/companions';

function settingsIcon(Icon: LucideIcon) {
  return <Icon size={16} strokeWidth={2.2} />;
}

const SAFETY_OPTIONS = [
  { id: 'confirm', name: 'Confirm risky actions', desc: 'Ask before destructive or risky work.', icon: settingsIcon(ShieldCheck) },
  { id: 'watch', name: 'Watch only', desc: 'Observe activity and surface warnings without approvals.', icon: settingsIcon(Eye) },
  { id: 'off', name: 'Off', desc: 'No extra Lil Buddy safety checks.', icon: settingsIcon(ShieldOff) },
];

const THEME_OPTIONS = [
  { id: 'honey', name: 'Honey', desc: 'Warm cream background with amber accents.', icon: settingsIcon(Palette) },
  { id: 'mint', name: 'Mint', desc: 'Fresh green background with calm contrast.', icon: settingsIcon(Leaf) },
  { id: 'sky', name: 'Sky', desc: 'Soft blue background with readable contrast.', icon: settingsIcon(Cloud) },
  { id: 'rose', name: 'Rose', desc: 'Gentle rose background with warm contrast.', icon: settingsIcon(Flower2) },
  { id: 'lavender', name: 'Lavender', desc: 'Soft purple background with balanced contrast.', icon: settingsIcon(Sparkles) },
  { id: 'charcoal', name: 'Charcoal', desc: 'True dark mode with high contrast.', icon: settingsIcon(Moon) },
];

function applyThemeAccent(value?: string) {
  const safe = THEME_OPTIONS.some((item) => item.id === value) ? value || 'honey' : 'honey';
  document.documentElement.dataset.themeAccent = safe;
  document.documentElement.dataset.themeMode = safe === 'charcoal' ? 'dark' : 'light';
}

const TELEGRAM_GATEWAY_MODE_OPTIONS = [
  { id: 'webhook', name: 'Webhook', desc: 'Best for long-term always-on use with an HTTPS tunnel or domain.', icon: settingsIcon(Webhook) },
  { id: 'polling', name: 'Polling', desc: 'Simple local fallback that does not require a public HTTPS URL.', icon: settingsIcon(Radio) },
];

const COMPANION_SIZE_OPTIONS = [
  { id: 'small', name: 'Small', desc: 'Compact companion size.', icon: settingsIcon(Minimize2) },
  { id: 'medium', name: 'Medium', desc: 'Default companion size.', icon: settingsIcon(Bot) },
  { id: 'large', name: 'Large', desc: 'Bigger companion for easier visibility.', icon: settingsIcon(Maximize2) },
];

const ANIMATION_LIBRARY = {
  idle: { id: 'idle', name: 'Idle', desc: 'Neutral idle.', preview: idlePreview, icon: settingsIcon(Bot) },
  blink: { id: 'idle', name: 'Blink base', desc: 'Neutral idle with blink source.', preview: blinkPreview, icon: settingsIcon(Eye) },
  hello: { id: 'hello', name: 'Wave', desc: 'Hello/wave one-shot.', preview: helloPreview, icon: settingsIcon(Hand) },
  working: { id: 'working', name: 'Focused work', desc: 'Focused work loop.', preview: workingPreview, icon: settingsIcon(WandSparkles) },
  thinking: { id: 'thinking', name: 'Thinking', desc: 'Thinking loop.', preview: thinkingPreview, icon: settingsIcon(Brain) },
  command: { id: 'command', name: 'Command', desc: 'Command one-shot.', preview: commandPreview, icon: settingsIcon(Terminal) },
  done: { id: 'done', name: 'Done', desc: 'Done one-shot.', preview: donePreview, icon: settingsIcon(CheckCircle2) },
  error: { id: 'error', name: 'Error', desc: 'Error one-shot.', preview: errorPreview, icon: settingsIcon(AlertTriangle) },
  walkRight: { id: 'walk-right', name: 'Walk', desc: 'Walk around while idle.', preview: workingPreview, icon: settingsIcon(Footprints) },
  walkLeft: { id: 'walk-left', name: 'Walk left', desc: 'Walk left cycle.', preview: workingPreview, icon: settingsIcon(Footprints) },
} as const;

const ANIMATION_OPTIONS = Object.values(ANIMATION_LIBRARY);

type AnimationChoice = (typeof ANIMATION_OPTIONS)[number];

const CATEGORY_ANIMATION_CHOICES = {
  work: [
    ANIMATION_LIBRARY.working,
    ANIMATION_LIBRARY.thinking,
    ANIMATION_LIBRARY.command,
  ],
  thinking: [
    ANIMATION_LIBRARY.thinking,
    ANIMATION_LIBRARY.working,
    ANIMATION_LIBRARY.idle,
    ANIMATION_LIBRARY.hello,
    ANIMATION_LIBRARY.command,
  ],
  command: [
    ANIMATION_LIBRARY.command,
    ANIMATION_LIBRARY.working,
    ANIMATION_LIBRARY.thinking,
    ANIMATION_LIBRARY.error,
  ],
  done: [
    ANIMATION_LIBRARY.done,
    ANIMATION_LIBRARY.hello,
    ANIMATION_LIBRARY.idle,
    ANIMATION_LIBRARY.working,
    ANIMATION_LIBRARY.command,
  ],
  error: [
    ANIMATION_LIBRARY.error,
    ANIMATION_LIBRARY.command,
    ANIMATION_LIBRARY.thinking,
    ANIMATION_LIBRARY.idle,
    ANIMATION_LIBRARY.done,
  ],
  idleWave: [
    ANIMATION_LIBRARY.walkRight,
    ANIMATION_LIBRARY.hello,
    ANIMATION_LIBRARY.idle,
    ANIMATION_LIBRARY.done,
    ANIMATION_LIBRARY.command,
    ANIMATION_LIBRARY.thinking,
    ANIMATION_LIBRARY.working,
    ANIMATION_LIBRARY.error,
  ],
} satisfies Record<string, AnimationChoice[]>;

function safeAnimation(value: string | undefined, fallback: NativeCompanionMood): NativeCompanionMood {
  return ANIMATION_OPTIONS.some((item) => item.id === value) ? value as NativeCompanionMood : fallback;
}

function providerOptions() {
  return providers.map((provider) => ({
    id: provider.id,
    name: provider.name,
    desc: provider.description,
    icon: <ProviderIcon providerId={provider.id} label={provider.name} className="settings-provider-icon" />,
  }));
}

export function SettingsPanel(props: {
  open: boolean;
  onClose: () => void;
  onProviderChange: (provider: string) => void;
  onModeChange: (mode: string) => void;
  onWorkspaceChange: (path: string) => void;
  onUserNameChange?: (name: string) => void;
}) {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [saving, setSaving] = useState(false);
  const [selectedCharacterId, setSelectedCharacterId] = useState(() => getSavedCompanionCharacterId());
  const [animationsOpen, setAnimationsOpen] = useState(false);
  const [telegramGatewayBusy, setTelegramGatewayBusy] = useState(false);
  const [telegramGatewayError, setTelegramGatewayError] = useState<string | null>(null);
  const [telegramGatewayNotice, setTelegramGatewayNotice] = useState<string | null>(null);
  const [telegramOpen, setTelegramOpen] = useState(false);

  useEffect(() => {
    if (settings?.companion_character_id) {
      setSelectedCharacterId(settings.companion_character_id);
    }
  }, [settings?.companion_character_id]);

  useEffect(() => {
    if (!props.open) return;
    setSelectedCharacterId(getSavedCompanionCharacterId());
    loadSettings().then((nextSettings) => { setSettings({ ...nextSettings, opencode_go_mode: 'run-json', claude_output_format: 'stream-json' }); applyThemeAccent(nextSettings.theme_accent); }).catch(() => {});
  }, [props.open]);

  function patch(next: Partial<AppSettings>) {
    setSettings((current) => (current ? { ...current, ...next } : current));
  }

  function patchTelegramField<K extends keyof AppSettings>(field: K, value: AppSettings[K], sourceField: keyof AppSettings) {
    patch({
      [field]: value,
      [sourceField]: false,
    } as Partial<AppSettings>);
  }

  function patchThemeAccent(value: string) {
    applyThemeAccent(value);
    patch({ theme_accent: value });
  }

  async function saveAnimationMapping(nextSettings: AppSettings) {
    await saveCompanionAnimationSettings({
      workAnimation: safeAnimation(nextSettings.companion_work_animation, 'working'),
      thinkingAnimation: safeAnimation(nextSettings.companion_thinking_animation, 'thinking'),
      commandAnimation: safeAnimation(nextSettings.companion_command_animation, 'command'),
      doneAnimation: safeAnimation(nextSettings.companion_done_animation, 'done'),
      errorAnimation: safeAnimation(nextSettings.companion_error_animation, 'error'),
      idleWaveAnimation: safeAnimation(nextSettings.companion_idle_wave_animation, 'walk-right'),
    });
  }

  async function updateCategoryAnimation(key: keyof AppSettings, value: NativeCompanionMood) {
    if (!settings) return;
    const nextSettings = { ...settings, [key]: value };
    setSettings(nextSettings);
    await saveAnimationMapping(nextSettings);
    await setNativeCompanionPose(value).catch(() => {});
  }

  function AnimationChoiceGrid(props: {
    label: string;
    description: string;
    value: NativeCompanionMood;
    choices: AnimationChoice[];
    onChange: (value: NativeCompanionMood) => void;
  }) {
    return (
      <div className="lm-animation-category">
        <div className="lm-animation-category-heading">
          <strong>{props.label}</strong>
          <small>{props.description}</small>
        </div>
        <div className="lm-animation-choice-grid">
          {props.choices.map((choice) => (
            <button
              key={`${props.label}-${choice.name}-${choice.id}`}
              type="button"
              className={props.value === choice.id ? 'active' : ''}
              onClick={() => props.onChange(choice.id as NativeCompanionMood)}
              title={choice.desc}
            >
              <img src={choice.preview} alt="" draggable={false} />
              <span>
                <strong>{choice.name}</strong>
                <small>{choice.desc}</small>
              </span>
            </button>
          ))}
        </div>
      </div>
    );
  }


  async function saveTelegramGatewayConfig(nextSettings = settings) {
    if (!nextSettings) return;
    await saveTelegramGatewaySettings({
      gatewayMode: nextSettings.telegram_gateway_mode ?? 'webhook',
      botToken: nextSettings.telegram_bot_token ?? '',
      botTokenFromEnv: nextSettings.telegram_bot_token_from_env ?? false,
      allowedChatId: nextSettings.telegram_allowed_chat_id ?? '',
      allowedChatIdFromEnv: nextSettings.telegram_allowed_chat_id_from_env ?? false,
      webhookPublicUrl: nextSettings.telegram_webhook_public_url ?? '',
      webhookPublicUrlFromEnv: nextSettings.telegram_webhook_public_url_from_env ?? false,
      webhookLocalPort: nextSettings.telegram_webhook_local_port ?? 8787,
      webhookPathSecret: nextSettings.telegram_webhook_path_secret ?? '',
      webhookPathSecretFromEnv: nextSettings.telegram_webhook_path_secret_from_env ?? false,
      webhookSecret: nextSettings.telegram_webhook_secret ?? '',
      webhookSecretFromEnv: nextSettings.telegram_webhook_secret_from_env ?? false,
    });
  }

  async function toggleTelegramGateway(enabled: boolean) {
    if (!settings) return;
    setTelegramGatewayBusy(true);
    setTelegramGatewayError(null);
    setTelegramGatewayNotice(null);

    try {
      await saveTelegramGatewayConfig(settings);
      if (enabled) {
        await startTelegramGateway();
      } else {
        await stopTelegramGateway();
      }
      patch({ telegram_gateway_enabled: enabled });
      setTelegramGatewayNotice(enabled ? 'Telegram connected successfully.' : 'Telegram gateway stopped.');
    } catch (error) {
      const message = String(error);
      setTelegramGatewayError(message);
      patch({ telegram_gateway_enabled: false });
    } finally {
      setTelegramGatewayBusy(false);
    }
  }

  async function persist() {
    if (!settings) return;
    setSaving(true);
    try {
      const provider = providers.some((item) => item.id === settings.selected_provider)
        ? settings.selected_provider
        : 'opencode-go';
      const mode = 'run-json';
      const safety = SAFETY_OPTIONS.some((item) => item.id === settings.safety_mode)
        ? settings.safety_mode
        : 'confirm';
      const accent = THEME_OPTIONS.some((item) => item.id === settings.theme_accent)
        ? settings.theme_accent
        : 'honey';
      await saveSelectedProvider(provider);
      await saveOpenCodeGoMode(mode);
      await saveClaudeCommand(settings.claude_command ?? 'claude');
      await saveClaudeOutputFormat('stream-json');
      await saveSafetyMode(safety);
      await saveThemeAccent(accent);
      await saveLaunchAtStartup(settings.launch_at_startup);
      await saveUserName(settings.user_name ?? '');
      await saveDefaultDirectory(settings.default_directory ?? '');
      await saveAnimationMapping(settings);
      saveCompanionCharacterId(selectedCharacterId);
      await saveCompanionCharacter(selectedCharacterId);
      await saveCompanionSize(settings.companion_size ?? 'medium');
      await saveTelegramGatewayConfig(settings);
      props.onProviderChange(provider);
      props.onModeChange(mode);
      props.onUserNameChange?.(settings.user_name ?? '');
      props.onClose();
    } finally {
      setSaving(false);
    }
  }

  if (!props.open) return null;

  return (
    <div className="lm-settings-overlay" data-tauri-drag-region="false" onMouseDown={(event) => event.stopPropagation()} onClick={props.onClose}>
      <div className="lm-modal-card lm-settings-card" data-tauri-drag-region="false" onMouseDown={(event) => event.stopPropagation()} onClick={(event) => event.stopPropagation()}>
        <div className="lm-modal-header">
          <div>
            <h3>Settings</h3>
            <small>Personalize your Lil Buddy.</small>
          </div>
          <div className='close-btn'>
          <button type="button" className="lm-icon-btn" aria-label="Close Chat" onClick={props.onClose}>
            <X />
          </button>
          </div>
        </div>

        {!settings ? (
          <div className="lm-provider-empty">Loading settings…</div>
        ) : (
          <div className="lm-settings-sections">
            <section className="lm-settings-section">
              <h3>Profile</h3>
              <label className="lm-settings-field">
                <span>Your name</span>
                <input
                  className="lm-settings-input"
                  value={settings.user_name ?? ''}
                  placeholder="Type your name"
                  onChange={(event) => patch({ user_name: event.target.value })}
                />
                <small>Used in Lil Buddy’s intro greeting.</small>
              </label>
            </section>

            <section className="lm-settings-section">
              <h3>Appearance</h3>
              <SettingsDropdown
                label="Theme accent"
                value={settings.theme_accent}
                options={THEME_OPTIONS}
                onChange={patchThemeAccent}
              />
            </section>

            <section className="lm-settings-section">
              <h3>Provider</h3>
              <SettingsDropdown
                label="Default provider"
                value={settings.selected_provider}
                options={providerOptions()}
                onChange={(value) => patch({ selected_provider: value })}
                searchable
              />
            </section>


            <section className="lm-settings-section">
              <h3>Default directory</h3>
              <label className="lm-settings-field">
                <span>Fallback directory</span>
                <input
                  className="lm-settings-input"
                  value={settings.default_directory ?? ''}
                  placeholder="Leave blank to use OS home folder"
                  onChange={(event) => patch({ default_directory: event.target.value })}
                />
                <small>Used when no expert/workspace folder is selected. This prevents Lil Buddy from defaulting to your home folder if you prefer another safe directory.</small>
              </label>
            </section>

            <section className="lm-settings-section">
              <h3>Claude CLI</h3>
              <label className="lm-settings-field">
                <span>Claude command/path</span>
                <input
                  className="lm-settings-input"
                  value={settings.claude_command ?? 'claude'}
                  placeholder="claude"
                  onChange={(event) => patch({ claude_command: event.target.value })}
                />
                <small>Leave as <code>claude</code> to detect from PATH, or use a full path.</small>
              </label>
            </section>


            <section className="lm-settings-section">
              <h3>Telegram Gateway</h3>
              <div className="lm-settings-dropdown-wrap">
                <button
                  type="button"
                  className="lm-settings-dropdown-trigger"
                  onClick={() => setTelegramOpen((open) => !open)}
                >
                  <span className="lm-model-icon">{settingsIcon(MessageCircle)}</span>
                  <span className="lm-model-main">
                    <span className="lm-model-kicker">Telegram Gateway</span>
                    <span>{settings.telegram_gateway_enabled ? 'Connected' : 'Configure Telegram'}</span>
                  </span>
                  <span className="lm-model-chevron">{telegramOpen ? '⌃' : '⌄'}</span>
                </button>

                {telegramOpen ? (
                  <div className="lm-settings-dropdown-popover lm-telegram-settings-popover">
              <SettingsDropdown
                label="Gateway mode"
                value={settings.telegram_gateway_mode ?? 'webhook'}
                options={TELEGRAM_GATEWAY_MODE_OPTIONS}
                onChange={(value) => patch({ telegram_gateway_mode: value })}
              />

              <label className="lm-settings-field">
                <span>Bot token</span>
                <input
                  className="lm-settings-input"
                  type="password"
                  value={settings.telegram_bot_token ?? ''}
                  placeholder="123456:ABC..."
                  onChange={(event) => patchTelegramField('telegram_bot_token', event.target.value, 'telegram_bot_token_from_env')}
                />
                <small>Create this with BotFather. Leave blank to use <code>LIL_BUDDY_TELEGRAM_BOT_TOKEN</code> from <code>.env</code>.</small>
              </label>

              <label className="lm-settings-field">
                <span>Allowed chat ID</span>
                <input
                  className="lm-settings-input"
                  value={settings.telegram_allowed_chat_id ?? ''}
                  placeholder="Leave empty to pair first /start"
                  onChange={(event) => patchTelegramField('telegram_allowed_chat_id', event.target.value, 'telegram_allowed_chat_id_from_env')}
                />
                <small>Leave empty, send <code>/start</code> to the bot, and Lil Buddy will pair the first Telegram chat.</small>
              </label>

              {(settings.telegram_gateway_mode ?? 'webhook') === 'webhook' ? (
                <>
                  <label className="lm-settings-field">
                    <span>Public webhook HTTPS URL</span>
                    <input
                      className="lm-settings-input"
                      value={settings.telegram_webhook_public_url ?? ''}
                      placeholder="https://your-domain-or-tunnel.example"
                      onChange={(event) => patchTelegramField('telegram_webhook_public_url', event.target.value, 'telegram_webhook_public_url_from_env')}
                    />
                    <small>Telegram requires a public HTTPS URL. Point your tunnel/domain to local port <code>{settings.telegram_webhook_local_port ?? 8787}</code>. Leave blank to use <code>LIL_BUDDY_TELEGRAM_WEBHOOK_PUBLIC_URL</code> from <code>.env</code>.</small>
                  </label>

                  <label className="lm-settings-field">
                    <span>Local webhook port</span>
                    <input
                      className="lm-settings-input"
                      value={String(settings.telegram_webhook_local_port ?? 8787)}
                      placeholder="8787"
                      onChange={(event) => patch({ telegram_webhook_local_port: Number(event.target.value) || 8787 })}
                    />
                    <small>Lil Buddy listens locally on <code>127.0.0.1:{settings.telegram_webhook_local_port ?? 8787}</code>.</small>
                  </label>

                  <label className="lm-settings-field">
                    <span>Webhook path secret</span>
                    <input
                      className="lm-settings-input"
                      value={settings.telegram_webhook_path_secret ?? ''}
                      placeholder="Set in Lil Buddy or .env"
                      onChange={(event) => patchTelegramField('telegram_webhook_path_secret', event.target.value, 'telegram_webhook_path_secret_from_env')}
                    />
                    <small>Final webhook path: <code>/telegram/{(settings.telegram_webhook_path_secret ?? '').trim() || 'your-secret-path'}</code>. Leave blank to let Lil Buddy fall back to <code>.env</code>.</small>
                  </label>

                  <label className="lm-settings-field">
                    <span>Webhook header secret</span>
                    <input
                      className="lm-settings-input"
                      type="password"
                      value={settings.telegram_webhook_secret ?? ''}
                      placeholder="Set in Lil Buddy or .env"
                      onChange={(event) => patchTelegramField('telegram_webhook_secret', event.target.value, 'telegram_webhook_secret_from_env')}
                    />
                    <small>Sent to Telegram as <code>secret_token</code> and verified from <code>X-Telegram-Bot-Api-Secret-Token</code>. Leave blank to let Lil Buddy fall back to <code>.env</code>.</small>
                  </label>
                </>
              ) : null}

              <div className="lm-workspace-actions">
                <button
                  type="button"
                  className="lm-ghost"
                  disabled={telegramGatewayBusy || !(settings.telegram_bot_token ?? '').trim()}
                  onClick={() => toggleTelegramGateway(true)}
                >
                  Start Telegram
                </button>
                <button
                  type="button"
                  className="lm-ghost"
                  disabled={telegramGatewayBusy}
                  onClick={() => toggleTelegramGateway(false)}
                >
                  Stop Telegram
                </button>
              </div>
              {telegramGatewayError ? (
                <div className="lm-settings-error">
                  {telegramGatewayError}
                </div>
              ) : null}

              {telegramGatewayNotice ? (
                <div className="lm-settings-success">
                  {telegramGatewayNotice}
                </div>
              ) : null}

              <small className="lm-settings-note">
                Webhook mode is best long-term, but you must expose the local port with an HTTPS tunnel or domain. Polling remains available as a local fallback.
              </small>

                  </div>
                ) : null}
              </div>
            </section>

            <section className="lm-settings-section">
              <h3>Safety</h3>
              <SettingsDropdown
                label="Safety mode"
                value={settings.safety_mode}
                options={SAFETY_OPTIONS}
                onChange={(value) => patch({ safety_mode: value })}
              />
            </section>

            <section className="lm-settings-section">
              <h3>Companion</h3>

              <SettingsDropdown
                label="Companion size"
                value={settings.companion_size ?? 'medium'}
                options={COMPANION_SIZE_OPTIONS}
                onChange={(value) => patch({ companion_size: value })}
              />

              <div className="lm-settings-dropdown-wrap">
                <button
                  type="button"
                  className="lm-settings-dropdown-trigger"
                  onClick={() => setAnimationsOpen((open) => !open)}
                >
                  <span className="lm-model-icon">{settingsIcon(Sparkles)}</span>
                  <span className="lm-model-main">
                    <span className="lm-model-kicker">Animation categories</span>
                    <span>Open animation mapping</span>
                  </span>
                  <span className="lm-model-chevron">{animationsOpen ? '⌃' : '⌄'}</span>
                </button>

                {animationsOpen ? (
                  <div className="lm-settings-dropdown-popover lm-animation-category-popover">
                    <AnimationChoiceGrid
                      label="Work"
                      description="Pick the one animation Lil Buddy plays when a chat run starts or work events arrive."
                      value={safeAnimation(settings.companion_work_animation, 'working')}
                      choices={CATEGORY_ANIMATION_CHOICES.work}
                      onChange={(value) => updateCategoryAnimation('companion_work_animation', value)}
                    />
                    <AnimationChoiceGrid
                      label="Thinking"
                      description="Pick the one animation used for reasoning/thinking events."
                      value={safeAnimation(settings.companion_thinking_animation, 'thinking')}
                      choices={CATEGORY_ANIMATION_CHOICES.thinking}
                      onChange={(value) => updateCategoryAnimation('companion_thinking_animation', value)}
                    />
                    <AnimationChoiceGrid
                      label="Command"
                      description="Pick the one animation used for shell/command events."
                      value={safeAnimation(settings.companion_command_animation, 'command')}
                      choices={CATEGORY_ANIMATION_CHOICES.command}
                      onChange={(value) => updateCategoryAnimation('companion_command_animation', value)}
                    />
                    <AnimationChoiceGrid
                      label="Done"
                      description="Pick the one animation used when a run exits."
                      value={safeAnimation(settings.companion_done_animation, 'done')}
                      choices={CATEGORY_ANIMATION_CHOICES.done}
                      onChange={(value) => updateCategoryAnimation('companion_done_animation', value)}
                    />
                    <AnimationChoiceGrid
                      label="Error"
                      description="Pick the one animation used for error/stderr events."
                      value={safeAnimation(settings.companion_error_animation, 'error')}
                      choices={CATEGORY_ANIMATION_CHOICES.error}
                      onChange={(value) => updateCategoryAnimation('companion_error_animation', value)}
                    />
                    <AnimationChoiceGrid
                      label="Idle behavior"
                      description="Pick what Lil Buddy does on the idle timer. Choose Walk if you want walking; choose another animation to disable idle walking."
                      value={safeAnimation(settings.companion_idle_wave_animation, 'walk-right')}
                      choices={CATEGORY_ANIMATION_CHOICES.idleWave}
                      onChange={(value) => updateCategoryAnimation('companion_idle_wave_animation', value)}
                    />
                  </div>
                ) : null}
              </div>

              <div className="lm-character-picker-grid">
                {companionCharacters.map((character) => (
                  <button
                    key={character.id}
                    className={selectedCharacterId === character.id ? 'active' : ''}
                    onClick={() => setSelectedCharacterId(character.id)}
                    type="button"
                  >
                    <img src={character.image} alt="" draggable={false} />
                    <span>
                      <strong>{character.name}</strong>
                      <small>{character.palette}</small>
                    </span>
                  </button>
                ))}
              </div>
              <small className="lm-settings-note">All companions now have matching idle, blink, walk, hello, thinking, work, command, done, and error frames. Idle behavior can be set to walking or any available animation.</small>
            </section>

            <section className="lm-settings-section">
              <h3>Startup</h3>
              <label className="lm-settings-toggle">
                <span>
                  <strong>Launch at startup</strong>
                  <small>Save the preference now and wire deeper desktop startup behavior later.</small>
                </span>
                <input
                  type="checkbox"
                  checked={settings.launch_at_startup ?? false}
                  onChange={(event) => patch({ launch_at_startup: event.target.checked })}
                />
              </label>
            </section>

            <div className="lm-modal-footer">
              <button className="lm-secondary-btn" onClick={props.onClose}>
                Cancel
              </button>
              <button className="lm-primary-btn" disabled={saving} onClick={persist}>
                {saving ? 'Saving…' : 'Save settings'}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
