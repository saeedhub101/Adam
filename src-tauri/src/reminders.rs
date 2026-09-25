use rusqlite::{params, Connection};
use std::path::Path;

pub fn add(path: &Path, title: &str, due_at: &str) -> Result<i64, String> {
    if title.trim().is_empty() {
        return Err("Reminder title is required".into());
    }
    if chrono::DateTime::parse_from_rfc3339(due_at).is_err() {
        return Err("Reminder due time must be RFC3339".into());
    }
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
pub fn due_unnotified(path: &Path) -> Result<Vec<(i64, String, String)>, String> {
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    let mut s = c.prepare("SELECT id,title,due_at FROM reminders WHERE completed=0 AND notified=0 AND datetime(due_at) <= datetime('now') ORDER BY due_at ASC").map_err(|e| e.to_string())?;
    let rows = s.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}
pub fn mark_notified(path: &Path, id: i64) -> Result<(), String> {
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    c.execute("UPDATE reminders SET notified=1 WHERE id=?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}
pub fn complete(path: &Path, id: i64) -> Result<(), String> {
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    c.execute("UPDATE reminders SET completed=1 WHERE id=?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn snooze(path: &Path, id: i64, minutes: i64) -> Result<(), String> {
    if !(1..=7200).contains(&minutes) { return Err("Snooze must be between 1 and 7200 minutes".into()); }
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    let changed = c.execute("UPDATE reminders SET due_at=datetime(due_at, ?1 || ' minutes'), notified=0, completed=0 WHERE id=?2", params![minutes, id]).map_err(|e| e.to_string())?;
    if changed == 0 { return Err("Reminder not found".into()); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    fn db() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("adam-reminders-{n}.db"))
    }

    #[test]
    fn rejects_invalid_due_time() {
        let p = db();
        assert!(add(&p, "test", "tomorrow").is_err());
        let _ = std::fs::remove_file(p);
    }
}
