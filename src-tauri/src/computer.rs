use std::process::{Child, Command};
use std::sync::{Mutex, OnceLock};

static CHILDREN: OnceLock<Mutex<Vec<Child>>> = OnceLock::new();

fn children() -> &'static Mutex<Vec<Child>> {
    CHILDREN.get_or_init(|| Mutex::new(Vec::new()))
}

fn reap_finished() {
    if let Ok(mut list) = children().lock() {
        list.retain_mut(|child| match child.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => false,
        });
    }
}

fn track(child: Child) -> Result<(), String> {
    reap_finished();
    children()
        .lock()
        .map_err(|_| "Computer control lock failed".to_string())?
        .push(child);
    Ok(())
}

pub fn open_target(target: &str) -> Result<String, String> {
    reap_finished();

    let key = target.trim().to_lowercase();
    let (program, args, label): (&str, &[&str], &str) = match key.as_str() {
        "notepad" | "المفكرة" => ("notepad.exe", &[], "Notepad"),
        "calculator" | "calc" | "الحاسبة" => ("calc.exe", &[], "Calculator"),
        "paint" | "mspaint" | "الرسام" => ("mspaint.exe", &[], "Paint"),
        "explorer" | "file explorer" | "الملفات" => {
            ("explorer.exe", &[], "File Explorer")
        }
        _ if key.starts_with("https://") || key.starts_with("http://") => {
            let child = Command::new("rundll32.exe")
                .args(["url.dll,FileProtocolHandler", target.trim()])
                .spawn()
                .map_err(|error| error.to_string())?;
            track(child)?;
            return Ok(target.trim().to_string());
        }
        _ => {
            return Err(
                "For safety, Adam currently opens only approved Windows apps or http/https URLs."
                    .into(),
            )
        }
    };

    let child = Command::new(program)
        .args(args)
        .spawn()
        .map_err(|error| error.to_string())?;
    track(child)?;

    Ok(label.into())
}

pub fn stop_all() {
    if let Ok(mut list) = children().lock() {
        for child in list.iter_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        list.clear();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn source_has_a_bounded_allowlist() {
        let source = include_str!("computer.rs");
        assert!(source.contains("notepad.exe"));
        assert!(source.contains("calc.exe"));
        assert!(source.contains("mspaint.exe"));
        assert!(source.contains("explorer.exe"));
        assert!(source.contains("For safety, Adam currently opens only approved"));
    }
}
