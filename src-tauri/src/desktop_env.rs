//! Detect the active Linux desktop environment so the UI can adapt
//! tray strategy (real systray vs. taskbar fallback).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesktopEnv {
    Xfce,
    Kde,
    Gnome,
    Cinnamon,
    Mate,
    Lxde,
    Lxqt,
    Pantheon,
    Budgie,
    Unity,
    Deepin,
    Unknown,
}

impl DesktopEnv {
    /// Whether the DE is known to ship a working StatusNotifierItem / AppIndicator
    /// host out of the box. GNOME does NOT (extension required), so we fall back.
    pub fn has_native_tray(&self) -> bool {
        matches!(
            self,
            DesktopEnv::Xfce
                | DesktopEnv::Kde
                | DesktopEnv::Cinnamon
                | DesktopEnv::Mate
                | DesktopEnv::Lxqt
                | DesktopEnv::Pantheon
                | DesktopEnv::Budgie
                | DesktopEnv::Unity
                | DesktopEnv::Deepin
        )
    }
}

pub fn detect() -> DesktopEnv {
    let raw = std::env::var("XDG_CURRENT_DESKTOP")
        .ok()
        .or_else(|| std::env::var("DESKTOP_SESSION").ok())
        .unwrap_or_default()
        .to_ascii_lowercase();

    // XDG_CURRENT_DESKTOP can be colon-separated, e.g. "ubuntu:GNOME"
    for token in raw.split(':') {
        match token {
            "xfce" => return DesktopEnv::Xfce,
            "kde" | "plasma" => return DesktopEnv::Kde,
            "gnome" | "gnome-classic" | "gnome-flashback" => return DesktopEnv::Gnome,
            "x-cinnamon" | "cinnamon" => return DesktopEnv::Cinnamon,
            "mate" => return DesktopEnv::Mate,
            "lxde" => return DesktopEnv::Lxde,
            "lxqt" => return DesktopEnv::Lxqt,
            "pantheon" => return DesktopEnv::Pantheon,
            "budgie" | "budgie-desktop" => return DesktopEnv::Budgie,
            "unity" => return DesktopEnv::Unity,
            "deepin" => return DesktopEnv::Deepin,
            _ => {}
        }
    }
    DesktopEnv::Unknown
}

#[derive(Debug, Clone, Serialize)]
pub struct DesktopReport {
    pub env: DesktopEnv,
    pub native_tray: bool,
    pub raw_xdg_current_desktop: String,
    pub session_type: String,
}

impl DesktopReport {
    pub fn current() -> Self {
        let env = detect();
        Self {
            env,
            native_tray: env.has_native_tray(),
            raw_xdg_current_desktop: std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
            session_type: std::env::var("XDG_SESSION_TYPE").unwrap_or_default(),
        }
    }
}

#[tauri::command]
pub fn desktop_report() -> DesktopReport {
    DesktopReport::current()
}
