use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedRepo {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub added_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitEvent {
    pub id: i64,
    pub repo_id: i64,
    pub repo_name: String,
    pub event_type: String,
    pub description: String,
    pub occurred_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: i64, pub repo_id: i64, pub hash: String, pub short_hash: String,
    pub message: String, pub author: String, pub author_email: String, pub committed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: i64, pub repo_id: i64, pub number: i64, pub title: String,
    pub state: String, pub body: String, pub author: String,
    pub created_at: i64, pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: i64, pub repo_id: i64, pub number: i64, pub title: String,
    pub state: String, pub body: String, pub author: String,
    pub created_at: i64, pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubSyncResult { pub issues_synced: usize, pub prs_synced: usize }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeLog {
    pub id: i64,
    pub duration_minutes: i64,
    pub repo_id: Option<i64>,
    pub repo_name: Option<String>,
    pub comment: String,
    pub logged_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSummary {
    pub today_minutes: i64,
    pub week_minutes: i64,
}
