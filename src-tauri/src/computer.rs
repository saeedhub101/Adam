use std::process::{Child, Command};
use std::sync::{Mutex, OnceLock};

static CHILDREN: OnceLock<Mutex<Vec<Child>>> = OnceLock::new();
fn children() -> &'static Mutex<Vec<Child>> { CHILDREN.get_or_init(|| Mutex::new(Vec::new())) }

fn reap_finished() {
    if let Ok(mut list) = children().lock() {
        list.retain_mut(|child| match child.try_wait() { Ok(Some(_)) => false, Ok(None) => true, Err(_) => false });
    }
}
fn track(child: Child) -> Result<(), String> {
    reap_finished();
    children().lock().map_err(|_| "Computer control lock failed".to_string())?.push(child);
    Ok(())
}

pub fn open_target(target: &str) -> Result<String, String> {
    reap_finished();
    let key = target.trim().to_lowercase();
    let (program, args, label): (&str, &[&str], &str) = match key.as_str() {
        "notepad" | "المفكرة" => ("notepad.exe", &[], "Notepad"),
        "calculator" | "calc" | "الحاسبة" => ("calc.exe", &[], "Calculator"),
        "paint" | "mspaint" | "الرسام" => ("mspaint.exe", &[], "Paint"),
        "explorer" | "file explorer" | "الملفات" => ("explorer.exe", &[], "File Explorer"),
        _ if key.starts_with("https://") || key.starts_with("http://") => {
            let child = Command::new("rundll32.exe").args(["url.dll,FileProtocolHandler", target.trim()]).spawn().map_err(|e| e.to_string())?;
            track(child)?; return Ok(target.trim().to_string());
        }
        _ => return Err("For safety, Adam currently opens only approved Windows apps or http/https URLs.".into()),
    };
    let child = Command::new(program).args(args).spawn().map_err(|e| e.to_string())?;
    track(child)?;
    Ok(label.into())
}

#[cfg(target_os = "windows")]
pub fn mouse_click(x: i32, y: i32, double: bool) -> Result<(), String> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    unsafe {
        SetCursorPos(x, y).map_err(|e| e.to_string())?;
        let flags = MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_LEFTUP;
        let input = INPUT { r#type: INPUT_MOUSE, Anonymous: INPUT_0 { mi: MOUSEINPUT { dx: 0, dy: 0, mouseData: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 } } };
        let sent = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        if sent != 1 { return Err("Mouse click failed".into()); }
        if double {
            std::thread::sleep(std::time::Duration::from_millis(80));
            let sent = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            if sent != 1 { return Err("Mouse double-click failed".into()); }
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn type_text(text: &str) -> Result<(), String> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    unsafe {
        for unit in text.encode_utf16() {
            let down = INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VIRTUAL_KEY(0), wScan: unit, dwFlags: KEYEVENTF_UNICODE, time: 0, dwExtraInfo: 0 } } };
            let up = INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VIRTUAL_KEY(0), wScan: unit, dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP, time: 0, dwExtraInfo: 0 } } };
            if SendInput(&[down, up], std::mem::size_of::<INPUT>() as i32) != 2 { return Err("Keyboard input failed".into()); }
        }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn key_press(vk: u16) -> Result<(), String> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    unsafe {
        let down = INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VIRTUAL_KEY(vk), wScan: 0, dwFlags: KEYEVENTF(0), time: 0, dwExtraInfo: 0 } } };
        let up = INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VIRTUAL_KEY(vk), wScan: 0, dwFlags: KEYEVENTF_KEYUP, time: 0, dwExtraInfo: 0 } } };
        if SendInput(&[down, up], std::mem::size_of::<INPUT>() as i32) != 2 { return Err("Key press failed".into()); }
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn windows() -> Result<Vec<(isize, String)>, String> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible};
    unsafe extern "system" fn callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let out = &mut *(lparam.0 as *mut Vec<(isize, String)>);
        if !IsWindowVisible(hwnd).as_bool() { return BOOL(1); }
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 { return BOOL(1); }
        let mut buf = vec![0u16; len as usize + 1];
        let written = GetWindowTextW(hwnd, &mut buf);
        if written > 0 {
            out.push((hwnd.0 as isize, String::from_utf16_lossy(&buf[..written as usize])));
        }
        BOOL(1)
    }
    let mut out = Vec::new();
    unsafe { EnumWindows(Some(callback), LPARAM(&mut out as *mut _ as isize)).map_err(|e| e.to_string())?; }
    Ok(out)
}

pub fn stop_all() {
    if let Ok(mut list) = children().lock() {
        for child in list.iter_mut() { let _ = child.kill(); let _ = child.wait(); }
        list.clear();
    }
}

#[cfg(not(target_os = "windows"))]
pub fn mouse_click(_: i32, _: i32, _: bool) -> Result<(), String> { Err("Computer input is only supported on Windows".into()) }
#[cfg(not(target_os = "windows"))]
pub fn type_text(_: &str) -> Result<(), String> { Err("Computer input is only supported on Windows".into()) }
#[cfg(not(target_os = "windows"))]
pub fn key_press(_: u16) -> Result<(), String> { Err("Computer input is only supported on Windows".into()) }
#[cfg(not(target_os = "windows"))]
pub fn windows() -> Result<Vec<(isize, String)>, String> { Err("Window enumeration is only supported on Windows".into()) }

#[cfg(test)]
mod tests {
    #[test]
    fn source_has_a_bounded_allowlist() {
        let source = include_str!("computer.rs");
        assert!(source.contains("notepad.exe"));
        assert!(source.contains("calc.exe"));
        assert!(source.contains("For safety, Adam currently opens only approved"));
    }
}
