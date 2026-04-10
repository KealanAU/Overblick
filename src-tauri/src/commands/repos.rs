use rusqlite::params;
use tauri::State;

use crate::db::{AppError, AppState, now_secs, get_repo_path};
use crate::github::types::{WatchedRepo, GitEvent, Commit};

#[tauri::command]
pub fn list_watched_repos(state: State<AppState>) -> Vec<WatchedRepo> {
    let db = state.db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT id, path, name, added_at FROM watched_repos ORDER BY added_at DESC",
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("list_watched_repos prepare error: {e}");
            return vec![];
        }
    };

    let result: Vec<WatchedRepo> = match stmt.query_map([], |row| {
        Ok(WatchedRepo {
            id: row.get(0)?,
            path: row.get(1)?,
            name: row.get(2)?,
            added_at: row.get(3)?,
        })
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            eprintln!("list_watched_repos query error: {e}");
            vec![]
        }
    };
    result
}

#[tauri::command]
pub fn add_watched_repo(path: String, state: State<AppState>) -> Result<WatchedRepo, AppError> {
    let repo_path = std::path::Path::new(&path);
    if !repo_path.join(".git").exists() {
        return Err(AppError::Invalid(format!("'{}' is not a git repository (no .git found)", path)));
    }

    let name = repo_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let added_at = now_secs();
    let db = state.db.lock().unwrap();

    db.execute(
        "INSERT INTO watched_repos (path, name, added_at) VALUES (?1, ?2, ?3)",
        params![path, name, added_at],
    )
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::Invalid(format!("Repository '{}' is already being watched", path))
        } else {
            AppError::Database(e)
        }
    })?;

    let id = db.last_insert_rowid();
    Ok(WatchedRepo { id, path, name, added_at })
}

#[tauri::command]
pub fn remove_watched_repo(id: i64, state: State<AppState>) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    db.execute("DELETE FROM watched_repos WHERE id = ?1", params![id])?;
    Ok(())
}

#[tauri::command]
pub fn get_recent_events(limit: u32, state: State<AppState>) -> Vec<GitEvent> {
    let db = state.db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT e.id, e.repo_id, r.name, e.event_type, e.description, e.occurred_at
         FROM git_events e
         JOIN watched_repos r ON r.id = e.repo_id
         ORDER BY e.occurred_at DESC
         LIMIT ?1",
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("get_recent_events prepare error: {e}");
            return vec![];
        }
    };

    let result: Vec<GitEvent> = match stmt.query_map(params![limit], |row| {
        Ok(GitEvent {
            id: row.get(0)?,
            repo_id: row.get(1)?,
            repo_name: row.get(2)?,
            event_type: row.get(3)?,
            description: row.get(4)?,
            occurred_at: row.get(5)?,
        })
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => {
            eprintln!("get_recent_events query error: {e}");
            vec![]
        }
    };
    result
}

#[tauri::command]
pub fn add_mock_event(repo_id: i64, state: State<AppState>) -> Result<(), AppError> {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT INTO git_events (repo_id, event_type, description, occurred_at)
         VALUES (?1, 'commit', 'Mock commit event', ?2)",
        params![repo_id, now_secs()],
    )?;
    Ok(())
}

#[tauri::command]
pub fn sync_repo_history(repo_id: i64, state: State<AppState>) -> Result<usize, AppError> {
    let repo_path = {
        let db = state.db.lock().unwrap();
        get_repo_path(&db, repo_id)?
    };

    let repo = git2::Repository::open(&repo_path)?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut synced = 0usize;
    let db = state.db.lock().unwrap();
    for oid_result in revwalk.take(1000) {
        let oid = match oid_result {
            Ok(o) => o,
            Err(_) => continue,
        };
        let commit = match repo.find_commit(oid) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let hash = oid.to_string();
        let short_hash = hash[..7].to_string();
        let message = commit.summary().unwrap_or("").to_string();
        let author = commit.author().name().unwrap_or("Unknown").to_string();
        let author_email = commit.author().email().unwrap_or("").to_string();
        let committed_at = commit.time().seconds();

        let rows = db.execute(
            "INSERT INTO commits (repo_id, hash, short_hash, message, author, author_email, committed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(repo_id, hash) DO UPDATE SET
               author       = excluded.author,
               author_email = excluded.author_email
             WHERE commits.author_email = '' OR commits.author = ''",
            params![repo_id, hash, short_hash, message, author, author_email, committed_at],
        ).unwrap_or(0);
        synced += rows;
    }
    Ok(synced)
}

#[tauri::command]
pub fn get_commits(repo_id: i64, limit: u32, state: State<AppState>) -> Vec<Commit> {
    let db = state.db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT id, repo_id, hash, short_hash, message, author, author_email, committed_at
         FROM commits WHERE repo_id = ?1 ORDER BY committed_at DESC LIMIT ?2",
    ) {
        Ok(s) => s,
        Err(e) => { eprintln!("get_commits prepare: {e}"); return vec![]; }
    };
    let result: Vec<Commit> = match stmt.query_map(params![repo_id, limit], |row| {
        Ok(Commit {
            id: row.get(0)?, repo_id: row.get(1)?, hash: row.get(2)?,
            short_hash: row.get(3)?, message: row.get(4)?,
            author: row.get(5)?, author_email: row.get(6)?, committed_at: row.get(7)?,
        })
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => { eprintln!("get_commits query: {e}"); vec![] }
    };
    result
}
