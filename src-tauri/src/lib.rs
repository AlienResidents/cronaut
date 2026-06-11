//! Cronaut — modern desktop UI for systemd --user timers and user crontab.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod crontab;
mod desktop_env;
mod error;
mod logs;
mod settings;
mod timers;
mod tray;

use tauri::{Manager, WindowEvent};
use tauri_plugin_store::StoreExt;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with_target(false)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Second launch: surface the existing window.
            tray::show_main(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(logs::LogState::new())
        .setup(|app| {
            // Load settings (creates store on first run).
            let store = app.store(settings::STORE_FILE)?;
            let start_minimized = store
                .get("start_minimized")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            tray::install(app.handle())?;

            if !start_minimized {
                if let Some(win) = app.get_webview_window("main") {
                    win.show()?;
                    win.set_focus()?;
                }
            }

            info!(
                desktop = ?desktop_env::detect(),
                native_tray = desktop_env::detect().has_native_tray(),
                "cronaut started"
            );
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    let app = window.app_handle();
                    let close_action = app
                        .store(settings::STORE_FILE)
                        .ok()
                        .and_then(|s| s.get("close_action"))
                        .and_then(|v| serde_json::from_value::<settings::CloseAction>(v).ok())
                        .unwrap_or_default();
                    match close_action {
                        settings::CloseAction::Quit => { /* let it close */ }
                        settings::CloseAction::MinimizeToTray => {
                            api.prevent_close();
                            let _ = window.hide();
                        }
                        settings::CloseAction::MinimizeToTaskbar => {
                            api.prevent_close();
                            let _ = window.minimize();
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            desktop_env::desktop_report,
            // timers
            crate::timers::list_timers,
            crate::timers::create_timer,
            crate::timers::update_timer,
            crate::timers::enable_timer,
            crate::timers::disable_timer,
            crate::timers::run_timer_now,
            crate::timers::remove_timer,
            // crontab
            crate::crontab::list_crontab,
            crate::crontab::add_crontab_entry,
            crate::crontab::update_crontab_entry,
            crate::crontab::toggle_crontab_entry,
            crate::crontab::remove_crontab_entry,
            crate::crontab::validate_cron_expression,
            // logs
            crate::logs::fetch_logs,
            crate::logs::start_tail,
            crate::logs::stop_tail,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
