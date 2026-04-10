use rusqlite::params;
use tauri::State;

use crate::db::{AppError, AppState, now_secs};
use crate::github::types::{TimeLog, TimeSummary};

#[tauri::command]
pub fn log_time(duration_minutes: i64, repo_id: Option<i64>, comment: String, state: State<AppState>) -> Result<TimeLog, AppError> {
    let logged_at = now_secs();
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT INTO time_logs (duration_minutes, repo_id, comment, logged_at) VALUES (?1, ?2, ?3, ?4)",
        params![duration_minutes, repo_id, comment, logged_at],
    )?;
    let id = db.last_insert_rowid();
    Ok(TimeLog { id, duration_minutes, repo_id, repo_name: None, comment, logged_at })
}

#[tauri::command]
pub fn get_time_logs(limit: u32, state: State<AppState>) -> Vec<TimeLog> {
    let db = state.db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT t.id, t.duration_minutes, t.repo_id, r.name, t.comment, t.logged_at
         FROM time_logs t
         LEFT JOIN watched_repos r ON r.id = t.repo_id
         ORDER BY t.logged_at DESC LIMIT ?1",
    ) {
        Ok(s) => s,
        Err(e) => { eprintln!("get_time_logs prepare: {e}"); return vec![]; }
    };
    let result: Vec<TimeLog> = match stmt.query_map(params![limit], |row| {
        Ok(TimeLog {
            id: row.get(0)?,
            duration_minutes: row.get(1)?,
            repo_id: row.get(2)?,
            repo_name: row.get(3)?,
            comment: row.get(4)?,
            logged_at: row.get(5)?,
        })
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => { eprintln!("get_time_logs query: {e}"); vec![] }
    };
    result
}

#[tauri::command]
pub fn get_time_summary(state: State<AppState>) -> TimeSummary {
    use chrono::{Local, Datelike};
    let now_local = Local::now();
    let today_start = now_local
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|dt| dt.and_local_timezone(Local).single())
        .map(|dt| dt.timestamp())
        .unwrap_or_else(|| now_secs() - 86400);
    let days_from_monday = now_local.weekday().num_days_from_monday() as i64;
    let week_start = today_start - days_from_monday * 86400;
    let db = state.db.lock().unwrap();
    let today_minutes: i64 = db.query_row(
        "SELECT COALESCE(SUM(duration_minutes), 0) FROM time_logs WHERE logged_at >= ?1",
        params![today_start],
        |row| row.get(0),
    ).unwrap_or(0);
    let week_minutes: i64 = db.query_row(
        "SELECT COALESCE(SUM(duration_minutes), 0) FROM time_logs WHERE logged_at >= ?1",
        params![week_start],
        |row| row.get(0),
    ).unwrap_or(0);
    TimeSummary { today_minutes, week_minutes }
}

#[tauri::command]
pub fn delete_time_log(id: i64, state: State<AppState>) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    db.execute("DELETE FROM time_logs WHERE id = ?1", params![id])?;
    Ok(())
}

#[tauri::command]
pub fn refresh_widget_data(state: State<AppState>) {
    let db = state.db.lock().unwrap();
    crate::db::write_widget_data(&db);
}

#[tauri::command]
pub fn update_time_log(
    id: i64,
    duration_minutes: i64,
    comment: String,
    repo_id: Option<i64>,
    logged_at: i64,
    state: State<AppState>,
) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    db.execute(
        "UPDATE time_logs SET duration_minutes = ?1, comment = ?2, repo_id = ?3, logged_at = ?4 WHERE id = ?5",
        params![duration_minutes, comment, repo_id, logged_at, id],
    )?;
    Ok(())
}
