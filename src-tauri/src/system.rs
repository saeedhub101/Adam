#[derive(Debug,Clone,serde::Serialize)]
pub struct SystemInfo { pub os:String,pub arch:String,pub hostname:String,pub app_data:String,pub cpu_count:usize,pub memory_note:String }

#[derive(Debug,Clone,serde::Serialize)]
pub struct AppEntry { pub name:String,pub path:String,pub source:String,pub aliases:Vec<String> }

pub fn info()->Result<SystemInfo,String>{
 let data=dirs::data_dir().map(|p|p.join("Adam").display().to_string()).unwrap_or_default();
 Ok(SystemInfo{os:std::env::consts::OS.into(),arch:std::env::consts::ARCH.into(),hostname:std::env::var("COMPUTERNAME").or_else(|_|std::env::var("HOSTNAME")).unwrap_or_default(),app_data:data,cpu_count:std::thread::available_parallelism().map(|n|n.get()).unwrap_or(1),memory_note:"Windows runtime exposes live memory/CPU telemetry through native diagnostics; no telemetry is sent remotely.".into()})
}

pub fn discover_apps()->Result<Vec<AppEntry>,String>{
 let mut out=Vec::new();
 let mut roots=Vec::new();
 if let Ok(p)=std::env::var("ProgramData"){roots.push(std::path::PathBuf::from(p).join("Microsoft/Windows/Start Menu/Programs"));}
 if let Ok(p)=std::env::var("APPDATA"){roots.push(std::path::PathBuf::from(p).join("Microsoft/Windows/Start Menu/Programs"));}
 for root in roots { walk(&root,&mut out,0); }
 #[cfg(target_os="windows")]
 { discover_uninstall_registry(&mut out); discover_uwp_startapps(&mut out); }
 out.sort_by(|a,b|a.name.to_lowercase().cmp(&b.name.to_lowercase()));
 out.dedup_by(|a,b|a.path.eq_ignore_ascii_case(&b.path) && a.name.eq_ignore_ascii_case(&b.name));
 Ok(out)
}
fn add_entry(out:&mut Vec<AppEntry>,name:String,path:String,source:&str){
 let clean=name.trim().to_string(); if clean.is_empty()||path.trim().is_empty(){return}
 let aliases=clean.split_whitespace().filter(|x|x.len()>=3).map(|x|x.to_lowercase()).collect();
 out.push(AppEntry{name:clean,path,source:source.to_string(),aliases});
}
fn walk(root:&std::path::Path,out:&mut Vec<AppEntry>,depth:u8){
 if depth>5{return}
 let Ok(rd)=std::fs::read_dir(root) else{return};
 for e in rd.flatten(){let p=e.path();if p.is_dir(){walk(&p,out,depth+1)}else if p.extension().and_then(|x|x.to_str()).map(|x|x.eq_ignore_ascii_case("lnk")||x.eq_ignore_ascii_case("url")).unwrap_or(false){add_entry(out,p.file_stem().and_then(|x|x.to_str()).unwrap_or("").to_string(),p.display().to_string(),"start-menu");}}
}
#[cfg(target_os="windows")]
fn discover_uninstall_registry(out:&mut Vec<AppEntry>){
 use std::process::Command;
 let keys=[r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall",r"HKLM\Software\Microsoft\Windows\CurrentVersion\Uninstall",r"HKLM\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"];
 for key in keys {
  let Ok(result)=Command::new("reg").args(["query",key,"/s"]).output() else{continue};
  if !result.status.success(){continue}
  let text=String::from_utf8_lossy(&result.stdout); let mut current=String::new();
  for line in text.lines(){let t=line.trim();if t.starts_with("HKEY_"){current=t.to_string()}else if t.to_ascii_lowercase().starts_with("displayname"){let display=t.splitn(2,"REG_SZ").nth(1).unwrap_or("").trim();add_entry(out,display.to_string(),current.clone(),"uninstall-registry");}}
 }
}
#[cfg(target_os="windows")]
fn discover_uwp_startapps(out:&mut Vec<AppEntry>){
 use std::process::Command;
 let Ok(result)=Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command","Get-StartApps | ConvertTo-Json -Compress"]).output() else{return};
 if !result.status.success(){return}
 let value:serde_json::Value=serde_json::from_slice(&result.stdout).unwrap_or(serde_json::Value::Null);
 match value {
  serde_json::Value::Array(items)=>for item in items{let n=item.get("Name").and_then(|v|v.as_str()).unwrap_or("");let id=item.get("AppID").and_then(|v|v.as_str()).unwrap_or("");add_entry(out,n.to_string(),id.to_string(),"uwp-startapps");},
  serde_json::Value::Object(item)=>{let n=item.get("Name").and_then(|v|v.as_str()).unwrap_or("");let id=item.get("AppID").and_then(|v|v.as_str()).unwrap_or("");add_entry(out,n.to_string(),id.to_string(),"uwp-startapps");},
  _=>{}
 }
}

#[cfg(target_os="windows")]
pub fn action(action:&str)->Result<String,String>{
 use windows::Win32::System::Shutdown::{LockWorkStation,ExitWindowsEx,EWX_LOGOFF,EWX_REBOOT,EWX_SHUTDOWN,EWX_FORCE,SHUTDOWN_REASON};
 unsafe{match action{
  "lock"=>{LockWorkStation().map_err(|e|e.to_string())?;Ok("Windows locked".into())},
  "restart"=>{ExitWindowsEx(EWX_REBOOT|EWX_FORCE,SHUTDOWN_REASON(0)).map_err(|e|e.to_string())?;Ok("Restart requested".into())},
  "shutdown"=>{ExitWindowsEx(EWX_SHUTDOWN|EWX_FORCE,SHUTDOWN_REASON(0)).map_err(|e|e.to_string())?;Ok("Shutdown requested".into())},
  "signout"=>{ExitWindowsEx(EWX_LOGOFF|EWX_FORCE,SHUTDOWN_REASON(0)).map_err(|e|e.to_string())?;Ok("Sign out requested".into())},
  "sleep"=>Err("Sleep requires explicit confirmation and is handled by the UI".into()),
  _=>Err("Unsupported action".into())
 }}}
#[cfg(not(target_os="windows"))]pub fn action(_: &str)->Result<String,String>{Err("Windows only".into())}

#[cfg(target_os="windows")]
pub fn media_action(action:&str)->Result<(),String>{use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput,INPUT,INPUT_0,INPUT_KEYBOARD,KEYBDINPUT,KEYEVENTF_KEYUP,VIRTUAL_KEY};let vk=match action.to_lowercase().as_str(){"play"|"pause"=>0xB3,"next"=>0xB0,"previous"=>0xB1,"volume_up"=>0xAF,"volume_down"=>0xAE,"mute"=>0xAD,_=>return Err("Unsupported media action".into())};unsafe{let d=INPUT{r#type:INPUT_KEYBOARD,Anonymous:INPUT_0{ki:KEYBDINPUT{wVk:VIRTUAL_KEY(vk),wScan:0,dwFlags:Default::default(),time:0,dwExtraInfo:0}}};let u=INPUT{r#type:INPUT_KEYBOARD,Anonymous:INPUT_0{ki:KEYBDINPUT{wVk:VIRTUAL_KEY(vk),wScan:0,dwFlags:KEYEVENTF_KEYUP,time:0,dwExtraInfo:0}}};if SendInput(&[d,u],std::mem::size_of::<INPUT>() as i32)!=2{return Err("Media key failed".into())}}Ok(())}
#[cfg(not(target_os="windows"))]pub fn media_action(_: &str)->Result<(),String>{Err("Windows only".into())}

#[cfg(target_os="windows")]
pub fn clipboard_read()->Result<String,String>{
 use windows::Win32::Foundation::{HANDLE,HGLOBAL};
 use windows::Win32::System::DataExchange::{CloseClipboard,GetClipboardData,OpenClipboard};
 use windows::Win32::System::Memory::{GlobalLock,GlobalSize,GlobalUnlock};
 use windows::Win32::System::DataExchange::CF_UNICODETEXT;
 unsafe{
  OpenClipboard(None).map_err(|e|e.to_string())?;
  let handle=GetClipboardData(CF_UNICODETEXT).map_err(|e|{let _=CloseClipboard();e.to_string()})?;
  let mem=HGLOBAL(handle.0);
  let ptr=GlobalLock(mem); if ptr.is_null(){let _=CloseClipboard();return Err("Clipboard text could not be locked".into())}
  let size=GlobalSize(mem).map_err(|e|{let _=CloseClipboard();e.to_string()})?.0 as usize; let slice=std::slice::from_raw_parts(ptr as *const u16,size/2);
  let end=slice.iter().position(|v|*v==0).unwrap_or(slice.len()); let value=String::from_utf16_lossy(&slice[..end]);
  let _=GlobalUnlock(mem); let _=CloseClipboard(); Ok(value)
 }
}
#[cfg(not(target_os="windows"))]pub fn clipboard_read()->Result<String,String>{Err("Windows only".into())}
#[cfg(target_os="windows")]
pub fn clipboard_write(text:&str)->Result<(),String>{
 use windows::Win32::Foundation::{HGLOBAL,HANDLE};
 use windows::Win32::System::DataExchange::{CloseClipboard,EmptyClipboard,OpenClipboard,SetClipboardData};
 use windows::Win32::System::Memory::{GlobalAlloc,GlobalLock,GlobalUnlock,GMEM_MOVEABLE};
 use windows::Win32::System::DataExchange::CF_UNICODETEXT;
 let wide:Vec<u16>=text.encode_utf16().chain(std::iter::once(0)).collect();
 unsafe{
  OpenClipboard(None).map_err(|e|e.to_string())?; EmptyClipboard().map_err(|e|{let _=CloseClipboard();e.to_string()})?;
  let mem=GlobalAlloc(GMEM_MOVEABLE,wide.len()*2).map_err(|e|{let _=CloseClipboard();e.to_string()})?;
  let ptr=GlobalLock(mem); if ptr.is_null(){let _=CloseClipboard();return Err("Clipboard memory lock failed".into())}
  std::ptr::copy_nonoverlapping(wide.as_ptr(),ptr as *mut u16,wide.len()); let _=GlobalUnlock(mem);
  let handle:HANDLE=mem.into();
  SetClipboardData(CF_UNICODETEXT,Some(handle)).map_err(|e|{let _=CloseClipboard();e.to_string()})?; let _=CloseClipboard(); Ok(())
 }
}
#[cfg(not(target_os="windows"))]pub fn clipboard_write(_: &str)->Result<(),String>{Err("Windows only".into())}
