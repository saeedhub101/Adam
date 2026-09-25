#[derive(Debug,Clone,serde::Serialize)]
pub struct SystemInfo { pub os:String,pub arch:String,pub hostname:String,pub app_data:String,pub cpu_count:usize,pub memory_note:String }

#[derive(Debug,Clone,serde::Serialize)]
pub struct AppEntry { pub name:String,pub path:String }

pub fn info()->Result<SystemInfo,String>{
 let data=dirs::data_dir().map(|p|p.join("Adam").display().to_string()).unwrap_or_default();
 Ok(SystemInfo{os:std::env::consts::OS.into(),arch:std::env::consts::ARCH.into(),hostname:std::env::var("COMPUTERNAME").or_else(|_|std::env::var("HOSTNAME")).unwrap_or_default(),app_data:data,cpu_count:std::thread::available_parallelism().map(|n|n.get()).unwrap_or(1),memory_note:"Windows runtime exposes live memory/CPU telemetry through native diagnostics; no telemetry is sent remotely.".into()})
}

pub fn discover_apps()->Result<Vec<AppEntry>,String>{
 let mut roots=Vec::new();
 if let Ok(p)=std::env::var("ProgramData"){roots.push(std::path::PathBuf::from(p).join("Microsoft/Windows/Start Menu/Programs"));}
 if let Ok(p)=std::env::var("APPDATA"){roots.push(std::path::PathBuf::from(p).join("Microsoft/Windows/Start Menu/Programs"));}
 let mut out=Vec::new();
 for root in roots { walk(&root,&mut out,0); }
 out.sort_by(|a,b|a.name.to_lowercase().cmp(&b.name.to_lowercase()));
 out.dedup_by(|a,b|a.path.eq_ignore_ascii_case(&b.path));
 Ok(out)
}
fn walk(root:&std::path::Path,out:&mut Vec<AppEntry>,depth:u8){
 if depth>5{return}
 let Ok(rd)=std::fs::read_dir(root) else{return};
 for e in rd.flatten(){let p=e.path();if p.is_dir(){walk(&p,out,depth+1)}else if p.extension().and_then(|x|x.to_str()).map(|x|x.eq_ignore_ascii_case("lnk")||x.eq_ignore_ascii_case("url")).unwrap_or(false){out.push(AppEntry{name:p.file_stem().and_then(|x|x.to_str()).unwrap_or("").to_string(),path:p.display().to_string()});}}
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
pub fn clipboard_read()->Result<String,String>{use std::process::Command;let out=Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command","Get-Clipboard -Raw"]).output().map_err(|e|e.to_string())?;if !out.status.success(){return Err("Clipboard read failed".into())}Ok(String::from_utf8_lossy(&out.stdout).to_string())}
#[cfg(not(target_os="windows"))]pub fn clipboard_read()->Result<String,String>{Err("Windows only".into())}
#[cfg(target_os="windows")]
pub fn clipboard_write(text:&str)->Result<(),String>{use std::process::{Command,Stdio};use std::io::Write;let mut child=Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command","Set-Clipboard"]).stdin(Stdio::piped()).spawn().map_err(|e|e.to_string())?;child.stdin.as_mut().ok_or("Clipboard stdin unavailable")?.write_all(text.as_bytes()).map_err(|e|e.to_string())?;let s=child.wait().map_err(|e|e.to_string())?;if s.success(){Ok(())}else{Err("Clipboard write failed".into())}}
#[cfg(not(target_os="windows"))]pub fn clipboard_write(_: &str)->Result<(),String>{Err("Windows only".into())}
