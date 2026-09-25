#[derive(Debug,Clone,serde::Serialize)]
pub struct Skill { pub name:String, pub patterns_en:Vec<String>, pub patterns_ar:Vec<String>, pub permission:String, pub confirmation:bool }

pub fn registry()->Vec<Skill>{
 vec![
  Skill{name:"open_app".into(),patterns_en:vec!["open".into(),"launch".into(),"start".into()],patterns_ar:vec!["افتح".into(),"شغل".into(),"شغّل".into()],permission:"launch_apps".into(),confirmation:false},
  Skill{name:"screen_read".into(),patterns_en:vec!["what is on screen".into(),"read screen".into(),"summarize screen".into()],patterns_ar:vec!["ماذا على الشاشة".into(),"اقرأ الشاشة".into(),"لخص الشاشة".into()],permission:"screen_capture".into(),confirmation:false},
  Skill{name:"clipboard".into(),patterns_en:vec!["copy".into(),"paste".into(),"clipboard".into()],patterns_ar:vec!["انسخ".into(),"الصق".into(),"الحافظة".into()],permission:"clipboard".into(),confirmation:false},
  Skill{name:"delete".into(),patterns_en:vec!["delete".into(),"remove".into()],patterns_ar:vec!["احذف".into(),"مسح".into()],permission:"files_modify_delete".into(),confirmation:true},
  Skill{name:"shutdown".into(),patterns_en:vec!["shutdown".into(),"restart".into(),"sign out".into()],patterns_ar:vec!["إيقاف التشغيل".into(),"أعد التشغيل".into(),"تسجيل الخروج".into()],permission:"system_settings".into(),confirmation:true},
  Skill{name:"shell".into(),patterns_en:vec!["powershell".into(),"command".into(),"terminal".into()],patterns_ar:vec!["باورشل".into(),"أمر".into(),"طرفية".into()],permission:"shell_commands".into(),confirmation:true},
 ]
}
