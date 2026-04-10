use std::sync::Mutex;

use tauri::Manager;

pub mod db;
pub mod github;
pub mod ai;
pub mod commands;
pub mod tray;

use db::{AppState, open_db};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = open_db().expect("Failed to open/initialise database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState { db: Mutex::new(conn) })
        .setup(|app| {
            tray::setup_tray(app)?;

            let window = app
                .get_webview_window("main")
                .expect("main window not found");
            let win_handle = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = win_handle.hide();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::repos::list_watched_repos,
            commands::repos::add_watched_repo,
            commands::repos::remove_watched_repo,
            commands::repos::get_recent_events,
            commands::repos::add_mock_event,
            commands::repos::sync_repo_history,
            commands::repos::get_commits,
            commands::github::get_issues,
            commands::github::get_pull_requests,
            commands::github::sync_github_data,
            commands::time::log_time,
            commands::time::get_time_logs,
            commands::time::get_time_summary,
            commands::time::delete_time_log,
            commands::time::update_time_log,
            commands::time::refresh_widget_data,
            commands::ai::summarize_activity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
