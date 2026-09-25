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
