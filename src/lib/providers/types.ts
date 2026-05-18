export type ProviderId = 'opencode-go' | 'claude';

export interface ProviderDefinition {
  id: ProviderId;
  name: string;
  command: string;
  native?: boolean;
  description: string;
}

export interface ProviderStatus extends ProviderDefinition {
  installed: boolean;
  authenticated: boolean;
  version?: string | null;
  commandPreview?: string;
}

export type DesktopEnvironment = {
  session_type: string;
  desktop: string;
  wayland_display: string | null;
  x11_display: string | null;
  is_hyprland: boolean;
  hyprland_signature: string | null;
  hyprctl_available: boolean;
};
