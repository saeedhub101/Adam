mod ai;
mod agent;
mod computer;
mod calendar;
mod local_brain;
mod memory;
mod permissions;
mod reminders;

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

const BASE_WIDTH: u32 = 360;
const BASE_HEIGHT: u32 = 520;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct WindowState {
    x: i32,
    y: i32,
    size: u32,
}

fn state_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("window.json"))
}

fn memory_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("adam.db"))
}

fn clamp_position(window: &WebviewWindow, x: i32, y: i32, size: PhysicalSize<u32>) -> (i32, i32) {
    let monitor = window
        .monitor_from_point(
            x as f64 + size.width as f64 / 2.0,
            y as f64 + size.height as f64 / 2.0,
        )
        .ok()
        .flatten()
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten());

    if let Some(mon) = monitor {
        let pos = mon.position();
        let ms = mon.size();
        let max_x = pos.x + ms.width as i32 - size.width as i32;
        let max_y = pos.y + ms.height as i32 - size.height as i32;
        return (
            x.clamp(pos.x, max_x.max(pos.x)),
            y.clamp(pos.y, max_y.max(pos.y)),
        );
    }
    (x, y)
}

fn read_state(app: &tauri::AppHandle) -> Result<Option<WindowState>, String> {
    let path = state_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(serde_json::from_str(&raw).ok())
}

fn write_state(app: &tauri::AppHandle, state: &WindowState) -> Result<(), String> {
    let path = state_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(state).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let win = app
                .get_webview_window("main")
                .ok_or("main window missing")?;
            let _ = win.set_ignore_cursor_events(false);

            let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            memory::init(&dir.join("adam.db"))?;
            calendar::init(&dir.join("adam.db"))?;
            permissions::init(&dir.join("adam.db"))?;

            if let Some(state) = read_state(app.handle())? {
                let scale = (state.size as f64 / 100.0).clamp(0.6, 1.6);
                let size = PhysicalSize::new(
                    (BASE_WIDTH as f64 * scale) as u32,
                    (BASE_HEIGHT as f64 * scale) as u32,
                );
                let (x, y) = clamp_position(&win, state.x, state.y, size);
                let _ = win.set_size(size);
                let _ = win.set_position(PhysicalPosition::new(x, y));
            }
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
            memory_update,
            memory_delete,
            reminder_add,
            reminder_list,
            reminder_complete,
            cloud_save_key,
            cloud_has_key,
            cloud_delete_key,
            cloud_chat,
            cloud_chat_stream,
            local_brain_reply,
            local_brain_execute,
            permission_list,
            permission_set,
            activity_log,
            excluded_apps,
            excluded_app_add,
            excluded_app_remove,
            calendar_list,
            calendar_delete,
            emergency_stop,
            agent_route,
            computer_open,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Adam");
}

#[tauri::command]
fn set_ignore_cursor_events(window: WebviewWindow, ignore: bool) -> Result<(), String> {
    window
        .set_ignore_cursor_events(ignore)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_character_size(
    app: tauri::AppHandle,
    window: WebviewWindow,
    size: u32,
) -> Result<(), String> {
    let size = size.clamp(60, 160);
    let scale = size as f64 / 100.0;
    let new_size = PhysicalSize::new(
        (BASE_WIDTH as f64 * scale) as u32,
        (BASE_HEIGHT as f64 * scale) as u32,
    );
    let pos = window
        .outer_position()
        .unwrap_or(PhysicalPosition::new(0, 0));
    let (x, y) = clamp_position(&window, pos.x, pos.y, new_size);
    window.set_size(new_size).map_err(|e| e.to_string())?;
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;
    write_state(&app, &WindowState { x, y, size })
}

#[tauri::command]
fn save_position(
    app: tauri::AppHandle,
    window: WebviewWindow,
    x: i32,
    y: i32,
) -> Result<(), String> {
    let outer = window
        .outer_size()
        .unwrap_or(PhysicalSize::new(BASE_WIDTH, BASE_HEIGHT));
    let current_size = read_state(&app)?.map(|s| s.size).unwrap_or(100);
    let (px, py) = clamp_position(&window, x, y, outer);
    window
        .set_position(PhysicalPosition::new(px, py))
        .map_err(|e| e.to_string())?;
    write_state(
        &app,
        &WindowState {
            x: px,
            y: py,
            size: current_size,
        },
    )
}

#[tauri::command]
fn load_position(app: tauri::AppHandle) -> Result<Option<WindowState>, String> {
    read_state(&app)
}

#[tauri::command]
fn memory_add(app: tauri::AppHandle, content: String, kind: Option<String>) -> Result<i64, String> {
    memory::add(
        &memory_path(&app)?,
        &content,
        kind.as_deref().unwrap_or("note"),
    )
}
#[tauri::command]
fn memory_list(
    app: tauri::AppHandle,
    limit: Option<u32>,
) -> Result<Vec<(i64, String, String, String)>, String> {
    memory::list(&memory_path(&app)?, limit.unwrap_or(50))
}
#[tauri::command]
fn memory_search(
    app: tauri::AppHandle,
    query: String,
    limit: Option<u32>,
) -> Result<Vec<(i64, String, String, String)>, String> {
    memory::search(&memory_path(&app)?, &query, limit.unwrap_or(20))
}
#[tauri::command]
fn memory_update(
    app: tauri::AppHandle,
    id: i64,
    content: String,
    kind: Option<String>,
) -> Result<(), String> {
    memory::update(
        &memory_path(&app)?,
        id,
        &content,
        kind.as_deref().unwrap_or("note"),
    )
}
#[tauri::command]
fn memory_delete(app: tauri::AppHandle, id: i64) -> Result<(), String> {
    memory::delete(&memory_path(&app)?, id)
}
#[tauri::command]
fn reminder_add(app: tauri::AppHandle, title: String, due_at: String) -> Result<i64, String> {
    reminders::add(&memory_path(&app)?, &title, &due_at)
}
#[tauri::command]
fn reminder_list(app: tauri::AppHandle) -> Result<Vec<(i64, String, String, bool)>, String> {
    reminders::list(&memory_path(&app)?)
}
#[tauri::command]
fn reminder_complete(app: tauri::AppHandle, id: i64) -> Result<(), String> {
    reminders::complete(&memory_path(&app)?, id)
}
#[tauri::command]
fn cloud_save_key(key: String) -> Result<(), String> {
    ai::save_key(&key)
}
#[tauri::command]
fn cloud_has_key() -> Result<bool, String> {
    ai::has_key()
}
#[tauri::command]
fn cloud_delete_key() -> Result<(), String> {
    ai::delete_key()
}
#[tauri::command]
async fn cloud_chat(
    base_url: String,
    model: String,
    messages: Vec<ai::Message>,
) -> Result<String, String> {
    ai::chat(ai::ChatRequest {
        base_url,
        model,
        messages,
    })
    .await
}
#[tauri::command]
fn local_brain_reply(input: String, language: String) -> Result<String, String> {
    Ok(local_brain::reply(&input, &language))
}
#[tauri::command]
fn local_brain_execute(
    app: tauri::AppHandle,
    input: String,
    language: String,
) -> Result<Option<String>, String> {
    local_brain::execute(&memory_path(&app)?, &input, &language)
}
#[tauri::command]
async fn cloud_chat_stream(
    app: tauri::AppHandle,
    request_id: String,
    base_url: String,
    model: String,
    messages: Vec<ai::Message>,
) -> Result<String, String> {
    ai::chat_stream(
        &app,
        &request_id,
        ai::ChatRequest {
            base_url,
            model,
            messages,
        },
    )
    .await
}
#[tauri::command]
fn calendar_list(app: tauri::AppHandle) -> Result<Vec<(i64, String, String)>, String> {
    calendar::list(&memory_path(&app)?)
}
#[tauri::command]
fn calendar_delete(app: tauri::AppHandle, id: i64) -> Result<(), String> {
    calendar::delete(&memory_path(&app)?, id)
}
#[tauri::command]
fn permission_list(app: tauri::AppHandle) -> Result<Vec<permissions::Permission>, String> {
    permissions::list(&memory_path(&app)?)
}
#[tauri::command]
fn permission_set(app: tauri::AppHandle, capability: String, mode: String) -> Result<(), String> {
    let p = memory_path(&app)?;
    permissions::set(&p, &capability, &mode)?;
    permissions::add_log(&p, "permission", &format!("{}={}", capability, mode))
}
#[tauri::command]
fn activity_log(
    app: tauri::AppHandle,
    limit: Option<u32>,
) -> Result<Vec<(i64, String, String, String)>, String> {
    permissions::logs(&memory_path(&app)?, limit.unwrap_or(100))
}
#[tauri::command]
fn excluded_apps(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    permissions::excluded(&memory_path(&app)?)
}
#[tauri::command]
fn excluded_app_add(app: tauri::AppHandle, name: String) -> Result<(), String> {
    permissions::add_excluded(&memory_path(&app)?, &name)
}
#[tauri::command]
fn excluded_app_remove(app: tauri::AppHandle, name: String) -> Result<(), String> {
    permissions::remove_excluded(&memory_path(&app)?, &name)
}
#[tauri::command]
fn emergency_stop(app: tauri::AppHandle) -> Result<(), String> {
    computer::stop_all();
    permissions::add_log(
        &memory_path(&app)?,
        "emergency_stop",
        "All pending computer-control actions cancelled",
    )
}

#[tauri::command]
fn agent_route(
    app: tauri::AppHandle,
    input: String,
    language: String,
) -> Result<Option<agent::AgentResult>, String> {
    agent::route(&memory_path(&app)?, &input, &language)
}

#[tauri::command]
fn computer_open(app: tauri::AppHandle, target: String) -> Result<String, String> {
    let path = memory_path(&app)?;
    if !permissions::allowed(&path, "computer.open")? {
        return Err("computer.open permission is not enabled".into());
    }
    let label = computer::open_target(&target)?;
    permissions::add_log(&path, "computer.open", &label)?;
    Ok(label)
}
