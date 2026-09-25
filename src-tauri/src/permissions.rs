use rusqlite::{params, Connection};
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Permission {
    pub capability: String,
    pub mode: String,
}

const MODES: [&str; 4] = ["ask", "session", "always", "deny"];

pub fn init(path: &Path) -> Result<(), String> {
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    c.execute_batch("CREATE TABLE IF NOT EXISTS permissions (capability TEXT PRIMARY KEY, mode TEXT NOT NULL); CREATE TABLE IF NOT EXISTS excluded_apps (app TEXT PRIMARY KEY); CREATE TABLE IF NOT EXISTS activity_log (id INTEGER PRIMARY KEY AUTOINCREMENT, action TEXT NOT NULL, detail TEXT NOT NULL, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);")
        .map_err(|e| e.to_string())
}

fn valid_mode(mode: &str) -> bool {
    MODES.contains(&mode)
}

pub fn list(path: &Path) -> Result<Vec<Permission>, String> {
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    let mut s = c
        .prepare("SELECT capability, mode FROM permissions ORDER BY capability")
        .map_err(|e| e.to_string())?;
    let rows = s
        .query_map([], |r| {
            Ok(Permission {
                capability: r.get(0)?,
                mode: r.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

pub fn set(path: &Path, capability: &str, mode: &str) -> Result<(), String> {
    if capability.trim().is_empty() {
        return Err("Capability is required".into());
    }
    if !valid_mode(mode) {
        return Err("Invalid permission mode".into());
    }
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    c.execute("INSERT INTO permissions(capability,mode) VALUES(?1,?2) ON CONFLICT(capability) DO UPDATE SET mode=excluded.mode", params![capability.trim(), mode])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn add_log(path: &Path, action: &str, detail: &str) -> Result<(), String> {
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    c.execute(
        "INSERT INTO activity_log(action,detail) VALUES(?1,?2)",
        params![action, detail],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn logs(path: &Path, limit: u32) -> Result<Vec<(i64, String, String, String)>, String> {
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    let mut s = c
        .prepare("SELECT id,action,detail,created_at FROM activity_log ORDER BY id DESC LIMIT ?1")
        .map_err(|e| e.to_string())?;
    let rows = s
        .query_map(params![limit.min(500)], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })
        .map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

pub fn excluded(path: &Path) -> Result<Vec<String>, String> {
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    let mut s = c
        .prepare("SELECT app FROM excluded_apps ORDER BY app")
        .map_err(|e| e.to_string())?;
    let rows = s.query_map([], |r| r.get(0)).map_err(|e| e.to_string())?;
    rows.map(|r| r.map_err(|e| e.to_string())).collect()
}

pub fn add_excluded(path: &Path, app: &str) -> Result<(), String> {
    if app.trim().is_empty() {
        return Err("App name is required".into());
    }
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    c.execute(
        "INSERT OR IGNORE INTO excluded_apps(app) VALUES(?1)",
        params![app.trim()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_excluded(path: &Path, app: &str) -> Result<(), String> {
    let c = Connection::open(path).map_err(|e| e.to_string())?;
    c.execute(
        "DELETE FROM excluded_apps WHERE app=?1",
        params![app.trim()],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
