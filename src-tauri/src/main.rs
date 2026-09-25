mod memory;
mod reminders;

use std::{fs, path::PathBuf};
use tauri::{Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

fn state_path(app: &tauri::AppHandle) -> PathBuf {
    app.path().app_data_dir().expect("app data directory").join("window.json")
}

fn memory_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_data_dir().map_err(|e| e.to_string())?.join("adam.db"))
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      let win = app.get_webview_window("main").unwrap();
      let _ = win.set_ignore_cursor_events(false);
      let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
      fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
      memory::init(&dir.join("adam.db"))?;
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      set_ignore_cursor_events,
      set_character_size,
      save_position,
      load_position,
      memory_add,
      memory_list,
      memory_search,
      reminder_add,
      reminder_list,
      reminder_complete
    ])
    .run(tauri::generate_context!())
    .expect("error while running Adam");
}

#[tauri::command]
fn set_ignore_cursor_events(window: WebviewWindow, ignore: bool) -> Result<(), String> {
  window.set_ignore_cursor_events(ignore).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_character_size(window: WebviewWindow, size: u32) -> Result<(), String> {
  let scale = (size as f64 / 100.0).clamp(0.6,1.6);
  let base = PhysicalSize::new(360u32,520u32);
  let _ = window.set_size(PhysicalSize::new((base.width as f64*scale) as u32,(base.height as f64*scale) as u32));
  Ok(())
}

#[tauri::command]
fn save_position(app: tauri::AppHandle, window: WebviewWindow, x: i32, y: i32) -> Result<(), String> {
  let dir = app.path().app_data_dir().map_err(|e|e.to_string())?;
  fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
  let _ = window.set_position(PhysicalPosition::new(x,y));
  fs::write(state_path(&app), format!("{{\"x\":{},\"y\":{}}}",x,y)).map_err(|e|e.to_string())
}

#[tauri::command]
fn load_position(app: tauri::AppHandle) -> Result<Option<serde_json::Value>, String> {
  let p=state_path(&app);
  if !p.exists() { return Ok(None); }
  let s=fs::read_to_string(p).map_err(|e|e.to_string())?;
  serde_json::from_str(&s).map(Some).map_err(|e|e.to_string())
}

#[tauri::command]
fn memory_add(app: tauri::AppHandle, content: String, kind: Option<String>) -> Result<i64, String> {
  let path = memory_path(&app)?;
  memory::add(&path, &content, kind.as_deref().unwrap_or("note"))
}

#[tauri::command]
fn memory_list(app: tauri::AppHandle, limit: Option<u32>) -> Result<Vec<(i64,String,String,String)>, String> {
  memory::list(&memory_path(&app)?, limit.unwrap_or(50))
}

#[tauri::command]
fn memory_search(app: tauri::AppHandle, query: String, limit: Option<u32>) -> Result<Vec<(i64,String,String,String)>, String> {
  memory::search(&memory_path(&app)?, &query, limit.unwrap_or(20))
}

#[tauri::command]
fn reminder_add(app: tauri::AppHandle, title: String, due_at: String) -> Result<i64, String> {
  reminders::add(&memory_path(&app)?, &title, &due_at)
}

#[tauri::command]
fn reminder_list(app: tauri::AppHandle) -> Result<Vec<(i64,String,String,bool)>, String> {
  reminders::list(&memory_path(&app)?)
}

#[tauri::command]
fn reminder_complete(app: tauri::AppHandle, id: i64) -> Result<(), String> {
  reminders::complete(&memory_path(&app)?, id)
}
