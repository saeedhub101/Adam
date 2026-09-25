use rusqlite::{params, Connection};
use std::path::Path;

pub fn init(path: &Path) -> Result<(), String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS memories (
           id INTEGER PRIMARY KEY AUTOINCREMENT,
           content TEXT NOT NULL,
           kind TEXT NOT NULL DEFAULT 'note',
           created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
           updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
         );
         CREATE INDEX IF NOT EXISTS idx_memories_updated ON memories(updated_at DESC);
        ",
    ).map_err(|e| e.to_string())
}

pub fn add(path: &Path, content: &str, kind: &str) -> Result<i64, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO memories(content, kind) VALUES(?1, ?2)",
        params![content, kind],
    ).map_err(|e| e.to_string())?;
    Ok(conn.last_insert_rowid())
}

pub fn list(path: &Path, limit: u32) -> Result<Vec<(i64, String, String, String)>, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, content, kind, created_at FROM memories ORDER BY id DESC LIMIT ?1"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![limit.min(1000)], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    }).map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

pub fn search(path: &Path, query: &str, limit: u32) -> Result<Vec<(i64, String, String, String)>, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    let escaped = query.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    let pattern = format!("%{}%", escaped);
    let mut stmt = conn.prepare(
        "SELECT id, content, kind, created_at FROM memories
         WHERE content LIKE ?1 ESCAPE '\\' COLLATE NOCASE
         ORDER BY id DESC LIMIT ?2"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![pattern, limit.min(1000)], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    }).map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}
