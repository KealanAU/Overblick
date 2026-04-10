use rusqlite::{Connection, params};

use crate::db::{overblick_data_dir, AppError};

pub fn get_repo_path(db: &Connection, repo_id: i64) -> Result<String, AppError> {
    db.query_row(
        "SELECT path FROM watched_repos WHERE id = ?1",
        params![repo_id],
        |row| row.get::<_, String>(0),
    )
    .map_err(|_| AppError::NotFound(format!("Repo {repo_id} not found")))
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WidgetData {
    pub today_minutes: i64,
    pub week_minutes: i64,
    pub recent_commits: Vec<WidgetCommit>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct WidgetCommit {
    pub repo: String,
    pub message: String,
    pub author: String,
    pub committed_at: i64,
}

pub fn write_widget_data(db: &Connection) {
    use chrono::{Local, Datelike};
    let now_local = Local::now();
    let today_start = now_local
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .and_then(|dt| dt.and_local_timezone(Local).single())
        .map(|dt| dt.timestamp())
        .unwrap_or_else(|| crate::db::now_secs() - 86400);
    let days_from_monday = now_local.weekday().num_days_from_monday() as i64;
    let week_start = today_start - days_from_monday * 86400;

    let today_minutes: i64 = db
        .query_row(
            "SELECT COALESCE(SUM(duration_minutes), 0) FROM time_logs WHERE logged_at >= ?1",
            params![today_start],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let week_minutes: i64 = db
        .query_row(
            "SELECT COALESCE(SUM(duration_minutes), 0) FROM time_logs WHERE logged_at >= ?1",
            params![week_start],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let recent_commits: Vec<WidgetCommit> = {
        let mut stmt = match db.prepare(
            "SELECT r.name, c.message, c.author, c.committed_at
             FROM commits c
             JOIN watched_repos r ON r.id = c.repo_id
             ORDER BY c.committed_at DESC LIMIT 5",
        ) {
            Ok(s) => s,
            Err(_) => return,
        };
        let result = match stmt.query_map([], |row| {
            Ok(WidgetCommit {
                repo: row.get(0)?,
                message: row.get(1)?,
                author: row.get(2)?,
                committed_at: row.get(3)?,
            })
        }) {
            Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
            Err(_) => vec![],
        };
        result
    };

    let data = WidgetData {
        today_minutes,
        week_minutes,
        recent_commits,
        updated_at: crate::db::now_secs(),
    };
    let json = match serde_json::to_string_pretty(&data) {
        Ok(j) => j,
        Err(_) => return,
    };

    if let Ok(dir) = overblick_data_dir() {
        let path = dir.join("widget_data.json");
        let _ = std::fs::write(path, json);
    }
}
