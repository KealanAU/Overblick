use rusqlite::Connection;

use crate::db::AppError;

const MIGRATIONS: &[(i64, &str)] = &[
    (
        1,
        "CREATE TABLE IF NOT EXISTS watched_repos (
             id       INTEGER PRIMARY KEY AUTOINCREMENT,
             path     TEXT NOT NULL UNIQUE,
             name     TEXT NOT NULL,
             added_at INTEGER NOT NULL
         );

         CREATE TABLE IF NOT EXISTS git_events (
             id          INTEGER PRIMARY KEY AUTOINCREMENT,
             repo_id     INTEGER NOT NULL,
             event_type  TEXT NOT NULL,
             description TEXT NOT NULL DEFAULT '',
             occurred_at INTEGER NOT NULL,
             FOREIGN KEY (repo_id) REFERENCES watched_repos(id) ON DELETE CASCADE
         );

         CREATE TABLE IF NOT EXISTS commits (
             id           INTEGER PRIMARY KEY AUTOINCREMENT,
             repo_id      INTEGER NOT NULL,
             hash         TEXT NOT NULL,
             short_hash   TEXT NOT NULL DEFAULT '',
             message      TEXT NOT NULL DEFAULT '',
             author       TEXT NOT NULL DEFAULT '',
             author_email TEXT NOT NULL DEFAULT '',
             committed_at INTEGER NOT NULL,
             UNIQUE(repo_id, hash),
             FOREIGN KEY (repo_id) REFERENCES watched_repos(id) ON DELETE CASCADE
         );

         CREATE TABLE IF NOT EXISTS issues (
             id         INTEGER PRIMARY KEY AUTOINCREMENT,
             repo_id    INTEGER NOT NULL,
             number     INTEGER NOT NULL,
             title      TEXT NOT NULL DEFAULT '',
             state      TEXT NOT NULL DEFAULT 'open',
             body       TEXT NOT NULL DEFAULT '',
             author     TEXT NOT NULL DEFAULT '',
             created_at INTEGER NOT NULL,
             updated_at INTEGER NOT NULL DEFAULT 0,
             UNIQUE(repo_id, number),
             FOREIGN KEY (repo_id) REFERENCES watched_repos(id) ON DELETE CASCADE
         );

         CREATE TABLE IF NOT EXISTS pull_requests (
             id         INTEGER PRIMARY KEY AUTOINCREMENT,
             repo_id    INTEGER NOT NULL,
             number     INTEGER NOT NULL,
             title      TEXT NOT NULL DEFAULT '',
             state      TEXT NOT NULL DEFAULT 'open',
             body       TEXT NOT NULL DEFAULT '',
             author     TEXT NOT NULL DEFAULT '',
             created_at INTEGER NOT NULL,
             updated_at INTEGER NOT NULL DEFAULT 0,
             UNIQUE(repo_id, number),
             FOREIGN KEY (repo_id) REFERENCES watched_repos(id) ON DELETE CASCADE
         );

         CREATE TABLE IF NOT EXISTS time_logs (
             id               INTEGER PRIMARY KEY AUTOINCREMENT,
             duration_minutes INTEGER NOT NULL,
             repo_id          INTEGER REFERENCES watched_repos(id) ON DELETE SET NULL,
             logged_at        INTEGER NOT NULL
         );",
    ),
    (
        2,
        "ALTER TABLE time_logs ADD COLUMN comment TEXT NOT NULL DEFAULT ''",
    ),
    (
        3,
        "ALTER TABLE commits ADD COLUMN author_email TEXT NOT NULL DEFAULT ''",
    ),
    (
        4,
        "CREATE INDEX IF NOT EXISTS idx_commits_repo_time   ON commits(repo_id, committed_at DESC);
         CREATE INDEX IF NOT EXISTS idx_git_events_time     ON git_events(occurred_at DESC);
         CREATE INDEX IF NOT EXISTS idx_time_logs_time      ON time_logs(logged_at DESC);
         CREATE INDEX IF NOT EXISTS idx_time_logs_repo_time ON time_logs(repo_id, logged_at DESC);
         CREATE INDEX IF NOT EXISTS idx_issues_repo_time    ON issues(repo_id, created_at DESC);
         CREATE INDEX IF NOT EXISTS idx_prs_repo_time       ON pull_requests(repo_id, created_at DESC);",
    ),
];

pub fn run(conn: &Connection) -> Result<(), AppError> {
    // Always run PRAGMAs and ensure the migrations tracking table exists.
    // These are all idempotent and safe to repeat on every startup.
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA foreign_keys=ON;
         CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY);",
    )?;

    for &(version, sql) in MIGRATIONS {
        let applied: bool = conn
            .query_row(
                "SELECT 1 FROM schema_migrations WHERE version = ?1",
                [version],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if applied {
            continue;
        }

        let result = conn.execute_batch(sql);
        match result {
            Ok(_) => {}
            Err(ref e) if e.to_string().contains("duplicate column name") => {
                // Column was already added by the old ad-hoc migration path —
                // treat as successfully applied and fall through to record it.
            }
            Err(e) => return Err(AppError::Database(e)),
        }

        conn.execute(
            "INSERT OR IGNORE INTO schema_migrations (version) VALUES (?1)",
            [version],
        )?;
    }

    Ok(())
}
