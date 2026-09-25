mod ai;
mod memory;
mod reminders;
mod permissions;
mod local_brain;
use std::{fs,path::PathBuf};
use tauri::{Manager,PhysicalPosition,PhysicalSize,WebviewWindow};
fn state_path(app:&tauri::AppHandle)->PathBuf{app.path().app_data_dir().expect("app data directory").join("window.json")}
fn memory_path(app:&tauri::AppHandle)->Result<PathBuf,String>{Ok(app.path().app_data_dir().map_err(|e|e.to_string())?.join("adam.db"))}
fn main(){tauri::Builder::default().setup(|app|{let win=app.get_webview_window("main").unwrap();let _=win.set_ignore_cursor_events(false);let dir=app.path().app_data_dir().map_err(|e|e.to_string())?;fs::create_dir_all(&dir).map_err(|e|e.to_string())?;memory::init(&dir.join("adam.db"))?;permissions::init(&dir.join("adam.db"))?;Ok(())}).invoke_handler(tauri::generate_handler![set_ignore_cursor_events,set_character_size,save_position,load_position,memory_add,memory_list,memory_search,memory_update,memory_delete,reminder_add,reminder_list,reminder_complete,cloud_save_key,cloud_has_key,cloud_delete_key,cloud_chat,cloud_chat_stream,local_brain_reply,permission_list,permission_set,activity_log,excluded_apps,excluded_app_add,excluded_app_remove,emergency_stop]).run(tauri::generate_context!()).expect("error while running Adam");}
#[tauri::command] fn set_ignore_cursor_events(window:WebviewWindow,ignore:bool)->Result<(),String>{window.set_ignore_cursor_events(ignore).map_err(|e|e.to_string())}
#[tauri::command] fn set_character_size(window:WebviewWindow,size:u32)->Result<(),String>{let scale=(size as f64/100.0).clamp(0.6,1.6);let base=PhysicalSize::new(360u32,520u32);window.set_size(PhysicalSize::new((base.width as f64*scale) as u32,(base.height as f64*scale) as u32)).map_err(|e|e.to_string())}
#[tauri::command] fn save_position(app:tauri::AppHandle,window:WebviewWindow,x:i32,y:i32)->Result<(),String>{let dir=app.path().app_data_dir().map_err(|e|e.to_string())?;fs::create_dir_all(&dir).map_err(|e|e.to_string())?;let size=window.outer_size().unwrap_or(PhysicalSize::new(360,520));let (mut px,mut py)=(x,y);if let Ok(Some(mon))=window.current_monitor(){let pos=mon.position();let ms=mon.size();let max_x=pos.x+(ms.width as i32)-(size.width as i32);let max_y=pos.y+(ms.height as i32)-(size.height as i32);px=px.clamp(pos.x,max_x.max(pos.x));py=py.clamp(pos.y,max_y.max(pos.y));}window.set_position(PhysicalPosition::new(px,py)).map_err(|e|e.to_string())?;fs::write(state_path(&app),format!("{{\"x\":{},\"y\":{}}}",px,py)).map_err(|e|e.to_string())}
#[tauri::command] fn load_position(app:tauri::AppHandle)->Result<Option<serde_json::Value>,String>{let p=state_path(&app);if !p.exists(){return Ok(None)};serde_json::from_str(&fs::read_to_string(p).map_err(|e|e.to_string())?).map(Some).map_err(|e|e.to_string())}
#[tauri::command] fn memory_add(app:tauri::AppHandle,content:String,kind:Option<String>)->Result<i64,String>{memory::add(&memory_path(&app)?,&content,kind.as_deref().unwrap_or("note"))}
#[tauri::command] fn memory_list(app:tauri::AppHandle,limit:Option<u32>)->Result<Vec<(i64,String,String,String)>,String>{memory::list(&memory_path(&app)?,limit.unwrap_or(50))}
#[tauri::command] fn memory_search(app:tauri::AppHandle,query:String,limit:Option<u32>)->Result<Vec<(i64,String,String,String)>,String>{memory::search(&memory_path(&app)?,&query,limit.unwrap_or(20))}
#[tauri::command] fn memory_update(app:tauri::AppHandle,id:i64,content:String,kind:Option<String>)->Result<(),String>{memory::update(&memory_path(&app)?,id,&content,kind.as_deref().unwrap_or("note"))}
#[tauri::command] fn memory_delete(app:tauri::AppHandle,id:i64)->Result<(),String>{memory::delete(&memory_path(&app)?,id)}
#[tauri::command] fn reminder_add(app:tauri::AppHandle,title:String,due_at:String)->Result<i64,String>{reminders::add(&memory_path(&app)?,&title,&due_at)}
#[tauri::command] fn reminder_list(app:tauri::AppHandle)->Result<Vec<(i64,String,String,bool)>,String>{reminders::list(&memory_path(&app)?)}
#[tauri::command] fn reminder_complete(app:tauri::AppHandle,id:i64)->Result<(),String>{reminders::complete(&memory_path(&app)?,id)}
#[tauri::command] fn cloud_save_key(key:String)->Result<(),String>{ai::save_key(&key)}
#[tauri::command] fn cloud_has_key()->Result<bool,String>{ai::has_key()}
#[tauri::command] fn cloud_delete_key()->Result<(),String>{ai::delete_key()}
#[tauri::command] async fn cloud_chat(base_url:String,model:String,messages:Vec<ai::Message>)->Result<String,String>{ai::chat(ai::ChatRequest{base_url,model,messages}).await}
#[tauri::command] fn local_brain_reply(input:String,language:String)->Result<String,String>{Ok(local_brain::reply(&input,&language))}
#[tauri::command] async fn cloud_chat_stream(app:tauri::AppHandle,request_id:String,base_url:String,model:String,messages:Vec<ai::Message>)->Result<String,String>{ai::chat_stream(&app,&request_id,ai::ChatRequest{base_url,model,messages}).await}

#[tauri::command] fn permission_list(app:tauri::AppHandle)->Result<Vec<permissions::Permission>,String>{permissions::list(&memory_path(&app)?)}
#[tauri::command] fn permission_set(app:tauri::AppHandle,capability:String,mode:String)->Result<(),String>{let p=memory_path(&app)?;permissions::set(&p,&capability,&mode)?;permissions::add_log(&p,"permission",&format!("{}={}",capability,mode))}
#[tauri::command] fn activity_log(app:tauri::AppHandle,limit:Option<u32>)->Result<Vec<(i64,String,String,String)>,String>{permissions::logs(&memory_path(&app)?,limit.unwrap_or(100))}
#[tauri::command] fn excluded_apps(app:tauri::AppHandle)->Result<Vec<String>,String>{permissions::excluded(&memory_path(&app)?)}
#[tauri::command] fn excluded_app_add(app:tauri::AppHandle,name:String)->Result<(),String>{permissions::add_excluded(&memory_path(&app)?,&name)}
#[tauri::command] fn excluded_app_remove(app:tauri::AppHandle,name:String)->Result<(),String>{permissions::remove_excluded(&memory_path(&app)?,&name)}
#[tauri::command] fn emergency_stop(app:tauri::AppHandle)->Result<(),String>{let p=memory_path(&app)?;permissions::add_log(&p,"emergency_stop","All pending computer-control actions cancelled")}
