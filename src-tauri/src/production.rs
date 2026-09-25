use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeDiagnostics {
    pub version: String,
    pub os: String,
    pub arch: String,
    pub data_dir_exists: bool,
    pub database_exists: bool,
    pub character_asset_policy: &'static str,
    pub phase6_release: bool,
    pub phase7_security: bool,
    pub phase8_recovery: bool,
    pub phase9_observability: bool,
}

pub fn diagnostics(data_dir: PathBuf, version: &str) -> RuntimeDiagnostics {
    RuntimeDiagnostics {
        version: version.to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        data_dir_exists: data_dir.exists(),
        database_exists: data_dir.join("adam.db").exists(),
        character_asset_policy: "production asset is external; runtime validates imported assets",
        phase6_release: true,
        phase7_security: true,
        phase8_recovery: true,
        phase9_observability: true,
    }
}

pub fn redact_secret(value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else {
        "***redacted***".to_string()
    }
}

pub fn verify_data_dir(data_dir: &PathBuf) -> Result<(), String> {
    fs::create_dir_all(data_dir).map_err(|e| e.to_string())?;
    let probe = data_dir.join(".adam-write-test");
    fs::write(&probe, b"ok").map_err(|e| e.to_string())?;
    let _ = fs::remove_file(probe);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_are_never_returned() {
        assert_eq!(redact_secret("abc"), "***redacted***");
        assert_eq!(redact_secret(""), "");
    }
}
