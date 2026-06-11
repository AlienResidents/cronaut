//! Persistent app settings.
//!
//! The store is owned by the frontend (via `tauri-plugin-store`), which
//! reads and writes JSON in the platform config dir, typically
//! `~/.config/com.alienresidents.cronaut/settings.json`.
//!
//! This module only declares the small subset of the schema that Rust
//! needs to act on (currently: `close_action`). All other settings
//! (theme, accent, start_minimized, notify_on_run, log_history_lines)
//! are frontend-only — see `src/lib/theme.ts` and `src/features/settings/`.

use serde::{Deserialize, Serialize};

pub const STORE_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CloseAction {
    /// Hide the window, keep running in the tray.
    #[default]
    MinimizeToTray,
    /// Hide the window, keep running but show in taskbar (no tray needed).
    MinimizeToTaskbar,
    /// Quit the app.
    Quit,
}
