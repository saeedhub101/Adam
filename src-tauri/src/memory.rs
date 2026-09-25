use rusqlite::{params, Connection};
use std::path::Path;

pub fn init(path: &Path) -> Result<(), String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS memories (id INTEGER PRIMARY KEY AUTOINCREMENT, content TEXT NOT NULL, kind TEXT NOT NULL DEFAULT 'note', created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
         CREATE INDEX IF NOT EXISTS idx_memories_updated ON memories(updated_at DESC);
         CREATE TABLE IF NOT EXISTS events (id INTEGER PRIMARY KEY AUTOINCREMENT,title TEXT NOT NULL,start_at TEXT NOT NULL,end_at TEXT,all_day INTEGER NOT NULL DEFAULT 0,recurrence TEXT,weekdays TEXT,reminder_offsets TEXT,notes TEXT,created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
         CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY AUTOINCREMENT,title TEXT,content TEXT NOT NULL,created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
         CREATE TABLE IF NOT EXISTS facts (id INTEGER PRIMARY KEY AUTOINCREMENT,key TEXT NOT NULL UNIQUE,value TEXT NOT NULL,created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
         CREATE TABLE IF NOT EXISTS conversation_summaries (id INTEGER PRIMARY KEY AUTOINCREMENT,summary TEXT NOT NULL,created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
         CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY,value TEXT NOT NULL,updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
         CREATE TABLE IF NOT EXISTS action_log (id INTEGER PRIMARY KEY AUTOINCREMENT,timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,action TEXT NOT NULL,target_app TEXT,target_window TEXT,result TEXT,permission TEXT);
         CREATE TABLE IF NOT EXISTS reminders (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, due_at TEXT NOT NULL, completed INTEGER NOT NULL DEFAULT 0, notified INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
         CREATE INDEX IF NOT EXISTS idx_reminders_due ON reminders(completed, due_at);"
    ).map_err(|e| e.to_string())?;
    let has_notified: bool = {
        let mut stmt = conn
            .prepare("PRAGMA table_info(reminders)")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|e| e.to_string())?;
        let has_column = rows.filter_map(Result::ok).any(|name| name == "notified");
        has_column
    };
    if !has_notified {
        conn.execute(
            "ALTER TABLE reminders ADD COLUMN notified INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn add(path: &Path, content: &str, kind: &str) -> Result<i64, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO memories(content,kind) VALUES(?1,?2)",
        params![content, kind],
    )
    .map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}
pub fn list(path: &Path, limit: u32) -> Result<Vec<(i64, String, String, String)>, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    let mut s = conn
        .prepare("SELECT id,content,kind,created_at FROM memories ORDER BY id DESC LIMIT ?1")
        .map_err(|e| e.to_string())?;
    let rows = s
        .query_map(params![limit.min(1000)], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}
pub fn search(
    path: &Path,
    q: &str,
    limit: u32,
) -> Result<Vec<(i64, String, String, String)>, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id,content,kind,created_at FROM memories ORDER BY id DESC LIMIT 1000").map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?))).map_err(|e| e.to_string())?;
    let query_tokens: Vec<String> = q.to_lowercase().split_whitespace().filter(|t| t.len() > 1).map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()).to_string()).filter(|t| !t.is_empty()).collect();
    let mut scored = Vec::new();
    for row in rows {
        let item = row.map_err(|e| e.to_string())?;
        let hay = item.1.to_lowercase();
        let mut score = 0usize;
        for token in &query_tokens {
            if hay.contains(token) { score += 1; }
        }
        if score > 0 || query_tokens.is_empty() { scored.push((score, item)); }
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.0.cmp(&a.1.0)));
    Ok(scored.into_iter().take(limit.min(1000) as usize).map(|(_, item)| item).collect())
}pub fn update(path: &Path, id: i64, content: &str, kind: &str) -> Result<(), String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE memories SET content=?1,kind=?2,updated_at=CURRENT_TIMESTAMP WHERE id=?3",
        params![content, kind, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
pub fn delete(path: &Path, id: i64) -> Result<(), String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM memories WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
