//! Tray icon + close-to-tray window behaviour.
//!
//! - On a DE that ships StatusNotifierItem/AppIndicator (XFCE, KDE, Cinnamon,
//!   MATE, etc.) the tray works out of the box.
//! - On GNOME without the AppIndicator extension, Tauri's tray creation
//!   may succeed but the icon is invisible to the user. We surface this via
//!   `desktop_report` so the frontend can warn + recommend the taskbar
//!   fallback in settings.

use crate::desktop_env::{detect, DesktopEnv};
use crate::error::Result;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

pub fn install(app: &AppHandle) -> Result<()> {
    let de = detect();
    if !de.has_native_tray() && de != DesktopEnv::Unknown {
        tracing::warn!(
            "desktop env {:?} does not ship a native tray; tray icon may be invisible",
            de
        );
    }

    let show = MenuItem::with_id(app, "show", "Show Cronaut", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide to tray", true, None::<&str>)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &hide, &separator, &quit])?;

    let _tray = TrayIconBuilder::with_id("cronaut-tray")
        .tooltip("Cronaut")
        .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
            // Fallback: 1x1 transparent — better than panicking
            tauri::image::Image::new_owned(vec![0, 0, 0, 0], 1, 1)
        }))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main(app),
            "hide" => hide_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                toggle_main(app);
            }
        })
        .build(app)?;
    Ok(())
}

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn hide_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

pub fn toggle_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
            }
            _ => {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
}
