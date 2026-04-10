use rusqlite::params;
use tauri::{
    App, Manager,
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

use crate::db::{AppState, now_secs};

pub fn setup_tray(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let show_item    = MenuItemBuilder::with_id("show",    "Show").build(app)?;
    let log_15_item  = MenuItemBuilder::with_id("log_15",  "Log 15 min").build(app)?;
    let log_30_item  = MenuItemBuilder::with_id("log_30",  "Log 30 min").build(app)?;
    let log_60_item  = MenuItemBuilder::with_id("log_60",  "Log 1 hour").build(app)?;
    let sep1         = PredefinedMenuItem::separator(app)?;
    let sep2         = PredefinedMenuItem::separator(app)?;
    let quit_item    = MenuItemBuilder::with_id("quit",    "Quit").build(app)?;
    let tray_menu = MenuBuilder::new(app)
        .items(&[&show_item, &sep1, &log_15_item, &log_30_item, &log_60_item, &sep2, &quit_item])
        .build()?;

    let icon = app
        .default_window_icon()
        .expect("No default window icon set")
        .clone();

    let app_handle_tray = app.handle().clone();
    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("Överblick")
        .menu(&tray_menu)
        .on_menu_event(move |_tray, event| {
            let mins: Option<i64> = match event.id().as_ref() {
                "log_15" => Some(15),
                "log_30" => Some(30),
                "log_60" => Some(60),
                _ => None,
            };
            if let Some(m) = mins {
                let state = app_handle_tray.state::<AppState>();
                let db = state.db.lock().unwrap();
                let recent_repo_id: Option<i64> = db
                    .query_row(
                        "SELECT repo_id FROM commits ORDER BY committed_at DESC LIMIT 1",
                        [],
                        |row| row.get::<_, i64>(0),
                    )
                    .ok();
                let _ = db.execute(
                    "INSERT INTO time_logs (duration_minutes, repo_id, comment, logged_at) VALUES (?1, ?2, '', ?3)",
                    params![m, recent_repo_id, now_secs()],
                );
                return;
            }
            match event.id().as_ref() {
            "show" => {
                if let Some(w) = app_handle_tray.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "quit" => {
                app_handle_tray.exit(0);
            }
            _ => {}
        }})
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    if w.is_visible().unwrap_or(false) {
                        let _ = w.hide();
                    } else {
                        let _ = w.show();
                        let _ = w.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
