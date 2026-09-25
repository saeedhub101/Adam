use rusqlite::{params, Connection};
use std::path::Path;

pub fn add(path: &Path, title: &str, due_at: &str) -> Result<i64, String> {
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    c.execute(
        "INSERT INTO reminders(title,due_at) VALUES(?1,?2)",
        params![title, due_at],
    )
    .map_err(|e| e.to_string())?;
    Ok(c.last_insert_rowid())
}
pub fn list(path: &Path) -> Result<Vec<(i64, String, String, bool)>, String> {
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    let mut s = c
        .prepare(
            "SELECT id,title,due_at,completed FROM reminders ORDER BY completed ASC,due_at ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = s
        .query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get::<_, i64>(3)? != 0))
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}
pub fn complete(path: &Path, id: i64) -> Result<(), String> {
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    c.execute("UPDATE reminders SET completed=1 WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
