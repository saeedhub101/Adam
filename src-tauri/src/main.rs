#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod agent;
mod ai;
mod calendar;
mod computer;
mod local_brain;
mod memory;
mod permissions;
mod production;
mod screen;
mod reminders;
mod safety;
mod skills;
mod system;
mod date_parser;
mod automation;

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, thread, time::Duration};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

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

#[cfg(target_os = "windows")]
fn grant_webview_media_permissions(window: &WebviewWindow) -> Result<(), String> {
    window
        .with_webview(|webview| {
            use webview2_com::Microsoft::Web::WebView2::Win32::{
                COREWEBVIEW2_PERMISSION_KIND, COREWEBVIEW2_PERMISSION_KIND_CAMERA,
                COREWEBVIEW2_PERMISSION_KIND_MICROPHONE, COREWEBVIEW2_PERMISSION_STATE_ALLOW,
            };
            use webview2_com::PermissionRequestedEventHandler;

            let core = match (unsafe { webview.controller().CoreWebView2() }) {
                Ok(core) => core,
                Err(_) => return,
            };

            let handler = PermissionRequestedEventHandler::create(Box::new(
                |_sender, args| {
                    let Some(args) = args else { return Ok(()); };
                    unsafe {
                        let mut kind = COREWEBVIEW2_PERMISSION_KIND::default();
                        args.PermissionKind(&mut kind)?;
                        if kind == COREWEBVIEW2_PERMISSION_KIND_MICROPHONE
                            || kind == COREWEBVIEW2_PERMISSION_KIND_CAMERA
                        {
                            args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW)?;
                        }
                    }
                    Ok(())
                },
            ));

            let mut token = 0i64;
            let _ = unsafe { core.add_PermissionRequested(&handler, &mut token) };
        })
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        })
        .setup(|app| {
            let win = app
                .get_webview_window("main")
                .ok_or("main window missing")?;
            let _ = win.set_ignore_cursor_events(false);
            #[cfg(target_os = "windows")]
            grant_webview_media_permissions(&win)?;
            let show = MenuItemBuilder::with_id("show", "Show Adam").build(app)?;
            let chat = MenuItemBuilder::with_id("chat", "Open AI Chat").build(app)?;
            let settings = MenuItemBuilder::with_id("settings", "Open Settings").build(app)?;
            let exit = MenuItemBuilder::with_id("exit", "Exit Adam").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&show, &chat, &settings, &exit])
                .build()?;
            TrayIconBuilder::new()
                .icon(tauri::include_image!("./icons/icon.ico"))
                .tooltip("Adam")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "chat" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                            let _ = app.emit("adam://open-chat", ());
                        }
                    }
                    "settings" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                            let _ = app.emit("adam://open-settings", ());
                        }
                    }
                    "exit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(w) = tray.app_handle().get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            memory::init(&dir.join("adam.db"))?;
            calendar::init(&dir.join("adam.db"))?;
            permissions::init(&dir.join("adam.db"))?;

            let reminder_app = app.handle().clone();
            thread::spawn(move || loop {
                thread::sleep(Duration::from_secs(15));
                let Ok(path) = reminder_app.path().app_data_dir().map(|p| p.join("adam.db")).map_err(|e| e.to_string()) else { continue; };
                if let Ok(items) = reminders::due_unnotified(&path) {
                    for (id, title, due_at) in items {
                        let _ = reminders::mark_notified(&path, id);
                        let _ = reminder_app.emit("adam://reminder-due", serde_json::json!({ "id": id, "title": title, "dueAt": due_at }));
                    }
                }
            });

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
            set_character_dimensions,
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
            reminder_due_now,
            cloud_save_key,
            cloud_has_key,
            cloud_delete_key,
            cloud_chat,
            cloud_chat_stream,
            cloud_test_connection,
            cloud_cancel,
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
            production_diagnostics,
            screen_capture,
            vision_analyze,
            computer_windows,
            computer_click,
            computer_type,
            computer_key,
            system_info,
            discover_apps,
            safety_check_text,
            skills_list,
            memory_export,
            memory_delete_all,
            calendar_add,
            calendar_update,
            calendar_events_between,
            reminder_snooze,
            system_action,
            clipboard_read,
            clipboard_write,
            media_action,
            parse_date_text,
            automation_plan,
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
fn set_character_dimensions(
    app: tauri::AppHandle,
    window: WebviewWindow,
    width: u32,
    height: u32,
    size: u32,
) -> Result<(), String> {
    let width = width.clamp(180, 640);
    let height = height.clamp(220, 760);
    let size = size.clamp(60, 160);
    let current = window
        .outer_position()
        .unwrap_or(PhysicalPosition::new(0, 0));
    let physical = PhysicalSize::new(width, height);
    let (x, y) = clamp_position(&window, current.x, current.y, physical);
    window.set_size(physical).map_err(|e| e.to_string())?;
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
fn reminder_due_now(app: tauri::AppHandle) -> Result<Vec<(i64, String, String)>, String> {
    reminders::due_unnotified(&memory_path(&app)?)
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
async fn cloud_test_connection(base_url: String) -> Result<String, String> {
    ai::test_connection(&base_url).await
}

#[tauri::command]
fn cloud_cancel(request_id: String) {
    ai::cancel_stream(&request_id);
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

#[tauri::command]
fn production_diagnostics(app: tauri::AppHandle) -> Result<production::RuntimeDiagnostics, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    production::verify_data_dir(&data_dir)?;
    Ok(production::diagnostics(data_dir, env!("CARGO_PKG_VERSION")))
}


#[tauri::command]
fn screen_capture(app: tauri::AppHandle) -> Result<screen::CaptureResult, String> {
    let path = memory_path(&app)?;
    if !permissions::allowed(&path, "screen.capture")? {
        return Err("screen.capture permission is not enabled".into());
    }
    let excluded = permissions::excluded(&path)?;
    let result = screen::capture_desktop(&excluded)?;
    permissions::add_log(&path, "screen.capture", &format!("{}x{}", result.width, result.height))?;
    Ok(result)
}

#[tauri::command]
async fn vision_analyze(base_url: String, model: String, prompt: String, png_base64: String) -> Result<ai::VisionResult, String> {
    ai::vision_analyze(&base_url, &model, &prompt, &png_base64).await
}


#[tauri::command]
fn computer_windows(app: tauri::AppHandle) -> Result<Vec<(isize, String)>, String> {
    let path = memory_path(&app)?;
    if !permissions::allowed(&path, "computer.control")? {
        return Err("computer.control permission is not enabled".into());
    }
    let result = computer::windows()?;
    permissions::add_log(&path, "computer.windows", &format!("{} visible windows", result.len()))?;
    Ok(result)
}

#[tauri::command]
fn computer_click(app: tauri::AppHandle, x: i32, y: i32, double: bool) -> Result<(), String> {
    let path = memory_path(&app)?;
    if !permissions::allowed(&path, "computer.control")? {
        return Err("computer.control permission is not enabled".into());
    }
    if x.abs() > 100000 || y.abs() > 100000 {
        return Err("Pointer coordinates are outside the safe bound".into());
    }
    computer::mouse_click(x, y, double)?;
    permissions::add_log(&path, "computer.click", &format!("{},{} double={}", x, y, double))
}

#[tauri::command]
fn computer_type(app: tauri::AppHandle, text: String) -> Result<(), String> {
    let path = memory_path(&app)?;
    if !permissions::allowed(&path, "computer.control")? {
        return Err("computer.control permission is not enabled".into());
    }
    if text.chars().count() > 4000 {
        return Err("Typed text exceeds the safe 4000-character bound".into());
    }
    computer::type_text(&text)?;
    permissions::add_log(&path, "computer.type", &format!("{} characters", text.chars().count()))
}

#[tauri::command]
fn computer_key(app: tauri::AppHandle, virtual_key: u16) -> Result<(), String> {
    let path = memory_path(&app)?;
    if !permissions::allowed(&path, "computer.control")? {
        return Err("computer.control permission is not enabled".into());
    }
    computer::key_press(virtual_key)?;
    permissions::add_log(&path, "computer.key", &format!("VK {}", virtual_key))
}


#[tauri::command]
fn system_info() -> Result<system::SystemInfo, String> { system::info() }

#[tauri::command]
fn discover_apps() -> Result<Vec<system::AppEntry>, String> { system::discover_apps() }

#[tauri::command]
fn safety_check_text(text: String) -> safety::SafetyResult { safety::check_untrusted_text(&text) }

#[tauri::command]
fn skills_list() -> Vec<skills::Skill> { skills::registry() }

#[tauri::command]
fn memory_export(app: tauri::AppHandle) -> Result<String, String> {
    memory::export_json(&memory_path(&app)?)
}

#[tauri::command]
fn memory_delete_all(app: tauri::AppHandle) -> Result<(), String> {
    memory::delete_all(&memory_path(&app)?)
}

#[tauri::command]
fn calendar_add(app: tauri::AppHandle,title:String,start_at:String,end_at:Option<String>,all_day:Option<bool>,recurrence:Option<String>,weekdays:Option<String>,reminder_offsets:Option<String>,notes:Option<String>)->Result<i64,String>{
 calendar::add_full(&memory_path(&app)?,&title,&start_at,end_at.as_deref(),all_day.unwrap_or(false),recurrence.as_deref(),weekdays.as_deref(),reminder_offsets.as_deref(),notes.as_deref())
}
#[tauri::command]
fn calendar_update(app:tauri::AppHandle,id:i64,title:String,start_at:String,end_at:Option<String>,all_day:Option<bool>,recurrence:Option<String>,weekdays:Option<String>,reminder_offsets:Option<String>,notes:Option<String>)->Result<(),String>{
 calendar::update_full(&memory_path(&app)?,id,&title,&start_at,end_at.as_deref(),all_day.unwrap_or(false),recurrence.as_deref(),weekdays.as_deref(),reminder_offsets.as_deref(),notes.as_deref())
}
#[tauri::command]
fn calendar_events_between(app:tauri::AppHandle,start:String,end:String)->Result<Vec<calendar::CalendarEvent>,String>{calendar::between(&memory_path(&app)?,&start,&end)}
#[tauri::command]
fn reminder_snooze(app:tauri::AppHandle,id:i64,minutes:i64)->Result<(),String>{reminders::snooze(&memory_path(&app)?,id,minutes)}
#[tauri::command]
fn system_action(app:tauri::AppHandle,action:String)->Result<String,String>{
 let p=memory_path(&app)?; if !permissions::allowed(&p,"system_settings")? {return Err("System action requires permission".into());}
 if !matches!(action.as_str(),"lock"|"sleep"|"restart"|"shutdown"|"signout"){return Err("Unsupported system action".into());}
 if matches!(action.as_str(),"restart"|"shutdown"|"signout"|"sleep"){return Err("This sensitive system action requires an explicit confirmation from the user.".into());}
 let r=system::action(&action)?; permissions::add_log(&p,"system_action",&action)?; Ok(r)
}
#[tauri::command]
fn clipboard_read(app:tauri::AppHandle)->Result<String,String>{let p=memory_path(&app)?;if !permissions::allowed(&p,"clipboard")?{return Err("Clipboard permission is not enabled".into())}system::clipboard_read()}
#[tauri::command]
fn clipboard_write(app:tauri::AppHandle,text:String)->Result<(),String>{let p=memory_path(&app)?;if !permissions::allowed(&p,"clipboard")?{return Err("Clipboard permission is not enabled".into())}if text.chars().count()>10000{return Err("Clipboard text exceeds safe limit".into())}system::clipboard_write(&text)}
#[tauri::command]
fn media_action(app:tauri::AppHandle,action:String)->Result<(),String>{let p=memory_path(&app)?;if !permissions::allowed(&p,"system_settings")?{return Err("Media control requires permission".into())}system::media_action(&action)}
#[tauri::command]
fn parse_date_text(text:String,now:String,language:String)->Result<date_parser::ParsedDate,String>{date_parser::parse(&text,&now,&language)}
#[tauri::command]
fn automation_plan(input:String)->Result<automation::AutomationPlan,String>{automation::plan(&input)}
