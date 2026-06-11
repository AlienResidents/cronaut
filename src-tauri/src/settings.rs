//! Persistent app settings, stored as JSON in the platform config dir
//! (typically ~/.config/com.alienresidents.cronaut/settings.json on Linux)
//! via tauri-plugin-store.
//!
//! The store is the source of truth — these structs are typed views over it.

use serde::{Deserialize, Serialize};

pub const STORE_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::System
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccentColor {
    Indigo,
    Violet,
    Emerald,
    Amber,
    Rose,
    Cyan,
    Slate,
}

impl Default for AccentColor {
    fn default() -> Self {
        Self::Violet
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CloseAction {
    /// Hide the window, keep running in the tray.
    MinimizeToTray,
    /// Hide the window, keep running but show in taskbar (no tray needed).
    MinimizeToTaskbar,
    /// Quit the app.
    Quit,
}

impl Default for CloseAction {
    fn default() -> Self {
        Self::MinimizeToTray
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub theme: ThemeMode,
    #[serde(default)]
    pub accent: AccentColor,
    #[serde(default)]
    pub close_action: CloseAction,
    /// Start the app minimised (window hidden, tray icon only).
    #[serde(default)]
    pub start_minimized: bool,
    /// How many lines of journalctl history to fetch by default.
    #[serde(default = "default_log_lines")]
    pub log_history_lines: u32,
    /// Show a desktop notification when a managed timer/cron fires.
    #[serde(default)]
    pub notify_on_run: bool,
}

fn default_log_lines() -> u32 {
    500
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::default(),
            accent: AccentColor::default(),
            close_action: CloseAction::default(),
            start_minimized: false,
            log_history_lines: default_log_lines(),
            notify_on_run: false,
        }
    }
}
