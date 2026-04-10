use rusqlite::params;
use tauri::State;

use crate::db::{AppError, AppState, get_repo_path};
use crate::github::client::{parse_github_owner_repo, parse_github_timestamp};
use crate::github::types::{Issue, PullRequest, GithubSyncResult};

#[tauri::command]
pub fn get_issues(repo_id: i64, state: State<AppState>) -> Vec<Issue> {
    let db = state.db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT id, repo_id, number, title, state, body, author, created_at, updated_at
         FROM issues WHERE repo_id = ?1 ORDER BY created_at DESC",
    ) {
        Ok(s) => s,
        Err(e) => { eprintln!("get_issues prepare: {e}"); return vec![]; }
    };
    let result: Vec<Issue> = match stmt.query_map(params![repo_id], |row| {
        Ok(Issue {
            id: row.get(0)?, repo_id: row.get(1)?, number: row.get(2)?,
            title: row.get(3)?, state: row.get(4)?, body: row.get(5)?,
            author: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)?,
        })
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => { eprintln!("get_issues query: {e}"); vec![] }
    };
    result
}

#[tauri::command]
pub fn get_pull_requests(repo_id: i64, state: State<AppState>) -> Vec<PullRequest> {
    let db = state.db.lock().unwrap();
    let mut stmt = match db.prepare(
        "SELECT id, repo_id, number, title, state, body, author, created_at, updated_at
         FROM pull_requests WHERE repo_id = ?1 ORDER BY created_at DESC",
    ) {
        Ok(s) => s,
        Err(e) => { eprintln!("get_pull_requests prepare: {e}"); return vec![]; }
    };
    let result: Vec<PullRequest> = match stmt.query_map(params![repo_id], |row| {
        Ok(PullRequest {
            id: row.get(0)?, repo_id: row.get(1)?, number: row.get(2)?,
            title: row.get(3)?, state: row.get(4)?, body: row.get(5)?,
            author: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)?,
        })
    }) {
        Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
        Err(e) => { eprintln!("get_pull_requests query: {e}"); vec![] }
    };
    result
}

#[tauri::command]
pub async fn sync_github_data(repo_id: i64, token: Option<String>, state: tauri::State<'_, AppState>) -> Result<GithubSyncResult, AppError> {
    let repo_path = {
        let db = state.db.lock().unwrap();
        get_repo_path(&db, repo_id)?
    };

    let remote_url = {
        let repo = git2::Repository::open(&repo_path)?;
        let remote = repo.find_remote("origin")
            .map_err(|_| AppError::NotFound("No remote 'origin' found".to_string()))?;
        remote.url().unwrap_or("").to_string()
    };

    let (owner, repo_name) = parse_github_owner_repo(&remote_url)
        .ok_or_else(|| AppError::Invalid(format!("Remote '{}' is not a GitHub URL", remote_url)))?;

    let client = reqwest::Client::new();

    let add_auth = |req: reqwest::RequestBuilder| -> reqwest::RequestBuilder {
        match &token {
            Some(t) if !t.is_empty() => req.header("Authorization", format!("Bearer {t}")),
            _ => req,
        }
    };

    let issues_url = format!("https://api.github.com/repos/{owner}/{repo_name}/issues?state=all&per_page=100&filter=all");
    let prs_url = format!("https://api.github.com/repos/{owner}/{repo_name}/pulls?state=all&per_page=100");

    let issues_raw: Vec<serde_json::Value> = add_auth(client.get(&issues_url))
        .header("User-Agent", "Överblick/1.0")
        .header("Accept", "application/vnd.github+json")
        .send().await?
        .json().await?;

    let prs_raw: Vec<serde_json::Value> = add_auth(client.get(&prs_url))
        .header("User-Agent", "Överblick/1.0")
        .header("Accept", "application/vnd.github+json")
        .send().await?
        .json().await?;

    let (issues_synced, prs_synced) = {
        let db = state.db.lock().unwrap();

        let mut issues_count = 0usize;
        for issue in &issues_raw {
            if issue.get("pull_request").is_some() { continue; }
            let number = issue["number"].as_i64().unwrap_or(0);
            let title = issue["title"].as_str().unwrap_or("").to_string();
            let state_str = issue["state"].as_str().unwrap_or("open").to_string();
            let body = issue["body"].as_str().unwrap_or("").to_string();
            let author = issue["user"]["login"].as_str().unwrap_or("").to_string();
            let created_at = parse_github_timestamp(issue["created_at"].as_str().unwrap_or(""));
            let updated_at = parse_github_timestamp(issue["updated_at"].as_str().unwrap_or(""));

            let rows = db.execute(
                "INSERT OR REPLACE INTO issues (repo_id, number, title, state, body, author, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![repo_id, number, title, state_str, body, author, created_at, updated_at],
            ).unwrap_or(0);
            issues_count += rows;
        }

        let mut prs_count = 0usize;
        for pr in &prs_raw {
            let number = pr["number"].as_i64().unwrap_or(0);
            let title = pr["title"].as_str().unwrap_or("").to_string();
            let state_str = pr["state"].as_str().unwrap_or("open").to_string();
            let body = pr["body"].as_str().unwrap_or("").to_string();
            let author = pr["user"]["login"].as_str().unwrap_or("").to_string();
            let created_at = parse_github_timestamp(pr["created_at"].as_str().unwrap_or(""));
            let updated_at = parse_github_timestamp(pr["updated_at"].as_str().unwrap_or(""));

            let rows = db.execute(
                "INSERT OR REPLACE INTO pull_requests (repo_id, number, title, state, body, author, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![repo_id, number, title, state_str, body, author, created_at, updated_at],
            ).unwrap_or(0);
            prs_count += rows;
        }

        (issues_count, prs_count)
    };

    Ok(GithubSyncResult { issues_synced, prs_synced })
}
