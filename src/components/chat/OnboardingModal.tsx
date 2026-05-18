import { useEffect, useMemo, useState } from 'react';
import { saveOnboardingComplete, saveUserName } from '../../lib/tauri/commands';

type OnboardingStep = 'welcome' | 'name' | 'provider' | 'workspace' | 'experts' | 'ready';

const steps: OnboardingStep[] = ['welcome', 'name', 'provider', 'workspace', 'experts', 'ready'];

const stepCopy: Record<OnboardingStep, { kicker: string; title: string; body: string }> = {
  welcome: {
    kicker: 'Welcome',
    title: 'Meet Lil Buddy',
    body: 'Lil Buddy is your local-first desktop AI helper for planning, search, writing, learning, light coding, and everyday work.',
  },
  name: {
    kicker: 'Personalize',
    title: 'What should Lil Buddy call you?',
    body: 'This is only used for greetings and a more personal experience. You can change it later in Settings.',
  },
  provider: {
    kicker: 'Provider',
    title: 'Choose your AI provider later',
    body: 'Lil Buddy works with local or CLI-based providers. Pick your default provider and model in Settings whenever you are ready.',
  },
  workspace: {
    kicker: 'Safety',
    title: 'Workspace safety',
    body: 'Lil Buddy does not scan or edit your folders automatically. When you choose a workspace, you stay in control of what folder is used.',
  },
  experts: {
    kicker: 'Experts',
    title: 'Meet experts',
    body: 'Experts are custom helpers with different roles. Each expert can have its own chat sessions and workspace.',
  },
  ready: {
    kicker: 'Ready',
    title: 'You are ready',
    body: 'Start with a simple message or use a quick-start prompt below. Telegram setup is optional and can wait until later.',
  },
};

const quickStarts = [
  'Help me plan my day',
  'Help me research something',
  'Create a writing outline',
  'Inspect a project safely',
];

export function OnboardingModal(props: {
  open: boolean;
  initialName?: string;
  canSkip?: boolean;
  onClose: () => void;
  onComplete: (name: string) => void;
}) {
  const [name, setName] = useState(props.initialName ?? '');
  const [saving, setSaving] = useState(false);
  const [stepIndex, setStepIndex] = useState(0);
  const step = steps[stepIndex];
  const copy = stepCopy[step];
  const progress = useMemo(() => Math.round(((stepIndex + 1) / steps.length) * 100), [stepIndex]);

  useEffect(() => {
    if (props.open) {
      setName(props.initialName ?? '');
      setStepIndex(0);
    }
  }, [props.initialName, props.open]);

  if (!props.open) return null;

  async function finish(nextName = name.trim()) {
    setSaving(true);
    try {
      if (nextName) {
        await saveUserName(nextName);
      } else {
        await saveOnboardingComplete(true);
      }
      props.onComplete(nextName);
    } finally {
      setSaving(false);
    }
  }

  async function skip() {
    setSaving(true);
    try {
      await saveOnboardingComplete(true);
      props.onClose();
    } finally {
      setSaving(false);
    }
  }

  function next() {
    if (step === 'ready') {
      finish();
      return;
    }

    setStepIndex((current) => Math.min(current + 1, steps.length - 1));
  }

  function back() {
    setStepIndex((current) => Math.max(current - 1, 0));
  }

  return (
    <div
      className="lm-onboarding-overlay"
      data-tauri-drag-region="false"
      onMouseDown={(event) => event.stopPropagation()}
    >
      <div
        className="lm-modal-card lm-onboarding-card lm-onboarding-flow-card"
        data-tauri-drag-region="false"
        onMouseDown={(event) => event.stopPropagation()}
        onClick={(event) => event.stopPropagation()}
      >
        <div className="lm-modal-header lm-onboarding-header">
          <div>
            <div className="lm-onboarding-kicker">{copy.kicker}</div>
            <h3>{copy.title}</h3>
            <small>{copy.body}</small>
          </div>
          <button className="lm-secondary-btn lm-onboarding-skip" disabled={saving} onClick={skip}>
            Skip
          </button>
        </div>

        <div className="lm-onboarding-progress" aria-hidden="true">
          <span style={{ width: `${progress}%` }} />
        </div>

        <div className="lm-onboarding-content">
          {step === 'welcome' ? (
            <div className="lm-onboarding-panel">
              <div className="lm-onboarding-icon">LB</div>
              <p>
                Lil Buddy can help you plan, search, organize, write, learn, and work through lightweight expert workflows.
              </p>
            </div>
          ) : null}

          {step === 'name' ? (
            <div className="lm-settings-section">
              <label className="lm-settings-field">
                <span>Your name</span>
                <input
                  className="lm-settings-input"
                  value={name}
                  placeholder="Type your name"
                  onChange={(event) => setName(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === 'Enter') {
                      event.preventDefault();
                      next();
                    }
                  }}
                  autoFocus
                  onMouseDown={(event) => event.stopPropagation()}
                  onPointerDown={(event) => event.stopPropagation()}
                />
              </label>

              <div className="lm-onboarding-preview">
                <strong>Preview</strong>
                <p>
                  {name.trim()
                    ? `Hey ${name.trim()}, I’m Lil Buddy. I can help you plan, search, write, learn, and get work done.`
                    : 'Hey, I’m Lil Buddy. I can help you plan, search, write, learn, and get work done.'}
                </p>
              </div>
            </div>
          ) : null}

          {step === 'provider' ? (
            <div className="lm-onboarding-panel">
              <ul className="lm-onboarding-list">
                <li>Pick OpenCode, Claude, or another supported provider in Settings.</li>
                <li>Choose a default model for that provider when you are ready.</li>
                <li>You can change provider/model choices anytime.</li>
              </ul>
            </div>
          ) : null}

          {step === 'workspace' ? (
            <div className="lm-onboarding-panel">
              <div className="lm-onboarding-warning">
                Lil Buddy only uses a workspace when you choose one or explicitly refresh it.
              </div>
              <p>
                Your configured AI provider may receive prompts, files, or workspace context. Review your provider’s privacy terms and only select folders you trust.
              </p>
            </div>
          ) : null}

          {step === 'experts' ? (
            <div className="lm-onboarding-panel">
              <div className="lm-onboarding-examples">
                <span>Default Lil Buddy</span>
                <span>Planner</span>
                <span>Research Helper</span>
                <span>Bug Hunter</span>
                <span>Repo Architect</span>
                <span>Linux Helper</span>
              </div>
              <p>
                Start with the default expert, then create your own helpers when your workflow is ready.
              </p>
            </div>
          ) : null}

          {step === 'ready' ? (
            <div className="lm-onboarding-panel">
              <div className="lm-onboarding-quick-starts">
                {quickStarts.map((prompt) => (
                  <button key={prompt} type="button">
                    {prompt}
                  </button>
                ))}
              </div>
              <p className="lm-onboarding-note">
                Telegram is optional. Start with local desktop chat first, then connect Telegram later in Settings.
              </p>
            </div>
          ) : null}
        </div>

        <div className="lm-modal-footer lm-onboarding-footer">
          <button className="lm-secondary-btn" disabled={saving || stepIndex === 0} onClick={back}>
            Back
          </button>
          <div className="lm-onboarding-step-count">
            {stepIndex + 1} / {steps.length}
          </div>
          <button className="lm-primary-btn" disabled={saving} onClick={next}>
            {saving ? 'Saving…' : step === 'ready' ? 'Open Lil Buddy' : 'Next'}
          </button>
        </div>
      </div>
    </div>
  );
}
