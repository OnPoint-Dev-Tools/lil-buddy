use serde::Serialize;
use std::{env, process::Command};

#[derive(Clone, Debug, Serialize)]
pub struct DesktopEnvironment {
    pub session_type: String,
    pub desktop: String,
    pub wayland_display: Option<String>,
    pub x11_display: Option<String>,
    pub is_hyprland: bool,
    pub hyprland_signature: Option<String>,
    pub hyprctl_available: bool,
}

pub fn detect_desktop_environment() -> DesktopEnvironment {
    let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let desktop = env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| env::var("DESKTOP_SESSION"))
        .unwrap_or_default();

    let wayland_display = env::var("WAYLAND_DISPLAY").ok();
    let x11_display = env::var("DISPLAY").ok();
    let hyprland_signature = env::var("HYPRLAND_INSTANCE_SIGNATURE").ok();
    let hyprctl_available = command_exists("hyprctl");

    let is_hyprland = desktop.to_lowercase().contains("hyprland")
        || hyprland_signature.is_some()
        || env::var("HYPRLAND_CMD").is_ok();

    DesktopEnvironment {
        session_type,
        desktop,
        wayland_display,
        x11_display,
        is_hyprland,
        hyprland_signature,
        hyprctl_available,
    }
}

fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", command))
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
