#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod desktop;
mod providers;
mod runtime;
mod safety;
mod settings;
mod ui_geometry;
mod workspace;
mod native_companion;
mod native_companion_manager;
mod gateways;

mod tray;
use tauri::Emitter;

fn main() {
    if std::env::args().any(|arg| arg == "--native-companion") {
        native_companion::run_from_args();
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            tray::setup_tray(app)?;
            let app_handle = app.handle().clone();
            if let Err(error) = native_companion_manager::start(&app_handle) {
                let _ = app.emit(
                    "runtime://stream",
                    runtime::stream_event("stderr", &format!("native companion failed to start: {error}")),
                );
            }

            if settings::load_settings(&app_handle).telegram_gateway_enabled {
                let _ = gateways::telegram::start(app_handle.clone());
            }

            let env = desktop::detect_desktop_environment();
            let env_msg = if env.is_hyprland {
                "Hyprland detected. Using Hyprland-aware desktop scaffold."
            } else if env.session_type == "wayland" {
                "Wayland detected. Using portable Tauri positioning."
            } else if env.session_type == "x11" {
                "X11 detected. Using portable Tauri positioning."
            } else {
                "Desktop session detected with fallback behavior."
            };

            let _ = app.emit("runtime://stream", runtime::stream_event("desktop", env_msg));
            let _ = app.emit("runtime://stream", runtime::stream_event("status", "Lil Buddy booted"));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::window::show_chat_window,
            commands::window::hide_chat_window,
            commands::window::toggle_chat_window,
            commands::window::anchor_chat_to_companion,
            commands::window::save_companion_position,
            commands::window::restore_companion_position,
            commands::window::reset_companion_position,
            commands::window::set_companion_menu_mode,
            commands::window::set_companion_walk_mode,
            commands::window::animate_companion_walk,
            commands::window::commit_companion_walk,
            commands::window::force_companion_repaint,
            commands::window::nudge_companion,
            commands::window::walk_native_companion,
            commands::window::set_native_companion_mood,
            commands::window::set_native_companion_pose,
            commands::window::set_native_companion_category,
            commands::window::hide_native_companion,
            commands::window::show_native_companion,
            commands::window::stop_native_companion,
            commands::window::start_native_companion,
            commands::providers::detect_providers,
            commands::providers::preview_provider_command,
            commands::providers::run_provider_command,
            commands::providers::stop_provider_command,
            commands::providers::provider_is_running,
            commands::approvals::deny_command_approval,
            commands::approvals::approve_command_approval,
            commands::approvals::request_command_approval,
            commands::approvals::allow_external_directory,
            commands::settings::save_favorite_models,
            commands::settings::save_launch_at_startup,
            commands::settings::save_user_name,
            commands::settings::save_onboarding_complete,
            commands::settings::save_companion_animation_settings,
            commands::settings::save_companion_character,
            commands::settings::save_companion_size,
            commands::settings::save_theme_accent,
            commands::settings::save_safety_mode,
            commands::settings::save_selected_model,
            commands::providers::list_provider_models,
            commands::settings::load_settings,
            commands::settings::save_selected_provider,
            commands::settings::save_opencode_go_mode,
            commands::settings::save_opencode_command,
            commands::settings::save_opencode_provider_key,
            commands::settings::save_claude_command,
            commands::settings::save_claude_output_format,
            commands::desktop::detect_desktop_environment,
            commands::workspace::detect_workspace,
            commands::workspace::get_workspace_diff,
            commands::workspace::emit_workspace_diff_events,
            commands::workspace_sessions::delete_workspace_session,
            commands::tray::restore_monitor_aware_positions,
            commands::tray::tray_quit,
            commands::tray::tray_open_chat,
            commands::tray::tray_hide_lil_buddy,
            commands::tray::tray_show_lil_buddy,
            commands::gateways::save_telegram_gateway_settings,
            commands::gateways::save_telegram_active_expert,
            commands::gateways::save_telegram_experts,
            commands::gateways::start_telegram_gateway,
            commands::gateways::stop_telegram_gateway,
            commands::gateways::telegram_gateway_running,
            commands::workspace_sessions::load_workspace_session,
            commands::workspace_sessions::save_workspace_session,
            commands::workspace_sessions::list_workspace_sessions,
            commands::settings::save_workspace_path,
            commands::settings::save_default_directory,
            commands::workspace::restore_all_workspace_files,
            commands::workspace::stop_workspace_watch,
            commands::workspace::start_workspace_watch,
            commands::workspace::restore_workspace_file,
            commands::workspace::open_workspace_terminal,
            commands::workspace::choose_workspace_folder,
            commands::safety::classify_command_risk,
        ])
        .run(tauri::generate_context!())
        .expect("error while running lil man");
}
