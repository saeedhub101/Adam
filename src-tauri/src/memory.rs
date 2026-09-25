use rusqlite::{params, Connection};
use std::path::Path;

pub fn init(path: &Path) -> Result<(), String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         CREATE TABLE IF NOT EXISTS memories (id INTEGER PRIMARY KEY AUTOINCREMENT, content TEXT NOT NULL, kind TEXT NOT NULL DEFAULT 'note', created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
         CREATE INDEX IF NOT EXISTS idx_memories_updated ON memories(updated_at DESC);
         CREATE TABLE IF NOT EXISTS reminders (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, due_at TEXT NOT NULL, completed INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
         CREATE INDEX IF NOT EXISTS idx_reminders_due ON reminders(completed, due_at);"
    ).map_err(|e| e.to_string())
}

pub fn add(path: &Path, content: &str, kind: &str) -> Result<i64, String> { let conn=Connection::open(path).map_err(|e|e.to_string())?; conn.execute("INSERT INTO memories(content,kind) VALUES(?1,?2)",params![content,kind]).map_err(|e|e.to_string())?; Ok(conn.last_insert_rowid()) }
pub fn list(path: &Path, limit:u32)->Result<Vec<(i64,String,String,String)>,String>{let conn=Connection::open(path).map_err(|e|e.to_string())?;let mut s=conn.prepare("SELECT id,content,kind,created_at FROM memories ORDER BY id DESC LIMIT ?1").map_err(|e|e.to_string())?;let rows=s.query_map(params![limit.min(1000)],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(|e|e.to_string())?;rows.map(|r|r.map_err(|e|e.to_string())).collect()}
pub fn search(path:&Path,q:&str,limit:u32)->Result<Vec<(i64,String,String,String)>,String>{let conn=Connection::open(path).map_err(|e|e.to_string())?;let escaped=q.replace('\\',"\\\\").replace('%',"\\%").replace('_',"\\_");let pattern=format!("%{}%",escaped);let mut s=conn.prepare("SELECT id,content,kind,created_at FROM memories WHERE content LIKE ?1 ESCAPE '\\\\' COLLATE NOCASE ORDER BY id DESC LIMIT ?2").map_err(|e|e.to_string())?;let rows=s.query_map(params![pattern,limit.min(1000)],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(|e|e.to_string())?;rows.map(|r|r.map_err(|e|e.to_string())).collect()}
