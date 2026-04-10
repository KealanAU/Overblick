use rusqlite::params;

use crate::db::{AppError, AppState, now_secs};
use crate::ai::{call_ai, strip_json_fences, SummaryResult, SummaryRow};

#[tauri::command]
pub async fn summarize_activity(
    provider: String,
    api_key: Option<String>,
    model: String,
    base_url: Option<String>,
    days: Option<i64>,
    state: tauri::State<'_, AppState>,
) -> Result<SummaryResult, AppError> {
    let period = days.unwrap_or(7);
    let since = now_secs() - (period * 86400);

    struct RepoActivity {
        name: String,
        commits: i64,
        time_minutes: i64,
        commit_messages: Vec<String>,
        time_comments: Vec<String>,
    }

    let repo_activities: Vec<RepoActivity> = {
        let db = state.db.lock().unwrap();

        let mut repos_stmt = db.prepare(
            "SELECT id, name FROM watched_repos ORDER BY name",
        ).map_err(AppError::Database)?;

        let repo_ids: Vec<(i64, String)> = repos_stmt
            .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))
            .map_err(AppError::Database)?
            .filter_map(|r| r.ok())
            .collect();

        let mut activities = Vec::new();
        for (repo_id, repo_name) in repo_ids {
            let commit_count: i64 = db.query_row(
                "SELECT COUNT(*) FROM commits WHERE repo_id = ?1 AND committed_at >= ?2",
                params![repo_id, since],
                |row| row.get(0),
            ).unwrap_or(0);

            let time_total: i64 = db.query_row(
                "SELECT COALESCE(SUM(duration_minutes), 0) FROM time_logs WHERE repo_id = ?1 AND logged_at >= ?2",
                params![repo_id, since],
                |row| row.get(0),
            ).unwrap_or(0);

            if commit_count == 0 && time_total == 0 {
                continue;
            }

            let mut msg_stmt = db.prepare(
                "SELECT message FROM commits WHERE repo_id = ?1 AND committed_at >= ?2 ORDER BY committed_at DESC LIMIT 20",
            ).map_err(AppError::Database)?;
            let messages: Vec<String> = msg_stmt
                .query_map(params![repo_id, since], |row| row.get::<_, String>(0))
                .map_err(AppError::Database)?
                .filter_map(|r| r.ok())
                .collect();

            let mut log_stmt = db.prepare(
                "SELECT comment FROM time_logs WHERE repo_id = ?1 AND logged_at >= ?2 AND comment != '' ORDER BY logged_at DESC LIMIT 10",
            ).map_err(AppError::Database)?;
            let comments: Vec<String> = log_stmt
                .query_map(params![repo_id, since], |row| row.get::<_, String>(0))
                .map_err(AppError::Database)?
                .filter_map(|r| r.ok())
                .collect();

            activities.push(RepoActivity {
                name: repo_name,
                commits: commit_count,
                time_minutes: time_total,
                commit_messages: messages,
                time_comments: comments,
            });
        }
        activities
    };

    if repo_activities.is_empty() {
        return Ok(SummaryResult {
            rows: vec![],
            overall: format!("No activity recorded in the last {period} days."),
            period_days: period,
        });
    }

    let mut context = String::new();
    for a in &repo_activities {
        context.push_str(&format!("\n### {} ({} commits, {} min logged)\n", a.name, a.commits, a.time_minutes));
        if !a.commit_messages.is_empty() {
            context.push_str(&format!("Commits: {}\n", a.commit_messages.join(" | ")));
        }
        if !a.time_comments.is_empty() {
            context.push_str(&format!("Time notes: {}\n", a.time_comments.join(" | ")));
        }
    }

    let repo_keys: Vec<String> = repo_activities.iter()
        .map(|a| format!("\"{}\"", a.name))
        .collect();

    let prompt = format!(
        "You are a developer activity summariser. Based on the data below, respond with ONLY valid JSON \
         (no markdown fences, no extra text):\n\n\
         {{\n  \"overall\": \"1-2 sentence overall summary\",\n  \"repos\": {{\n    {keys}: \"one-line highlight\"\n  }}\n}}\n\n\
         Keys in \"repos\" must exactly match these repo names: {names}.\n\
         Activity for the last {period} days:{context}",
        keys = repo_keys.join(": \"...\",\n    "),
        names = repo_keys.join(", "),
    );

    let model_id = if model.is_empty() {
        match provider.as_str() {
            "claude" => "claude-sonnet-4-6",
            "openai" => "gpt-4o-mini",
            _        => "llama3.2",
        }.to_string()
    } else {
        model
    };

    let raw = call_ai(
        &provider,
        api_key.as_deref(),
        &model_id,
        base_url.as_deref(),
        prompt,
    ).await.map_err(AppError::Invalid)?;

    let json_str = strip_json_fences(&raw);
    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .unwrap_or_else(|_| serde_json::json!({ "overall": raw, "repos": {} }));

    let overall = parsed["overall"].as_str().unwrap_or("").to_string();
    let repos_map = parsed["repos"].as_object();

    let rows = repo_activities.into_iter().map(|a| {
        let highlight = repos_map
            .and_then(|m| m.get(&a.name))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        SummaryRow {
            repo: a.name,
            commits: a.commits,
            time_minutes: a.time_minutes,
            highlight,
        }
    }).collect();

    Ok(SummaryResult { rows, overall, period_days: period })
}
