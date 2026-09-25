use crate::{computer, permissions};
use std::path::Path;
#[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
pub struct AgentResult{pub intent:String,pub action:String,pub message:String,pub requires_confirmation:bool}
pub fn route(path:&Path,input:&str,language:&str)->Result<Option<AgentResult>,String>{
 let q=input.trim(); if q.is_empty(){return Ok(None)}; let lower=q.to_lowercase();
 let open=["open ","launch ","start ","افتح ","شغل ","شغّل "]; if !open.iter().any(|p|lower.starts_with(p)){return Ok(None)};
 let target=q.split_once(' ').map(|(_,v)|v.trim()).unwrap_or("");
 if target.is_empty(){return Ok(Some(AgentResult{intent:"computer.open".into(),action:"none".into(),message:if language.eq_ignore_ascii_case("ar"){"ماذا تريد أن أفتح؟".into()}else{"What should I open?".into()},requires_confirmation:false}))}
 if !permissions::allowed(path,"computer.open")? {return Ok(Some(AgentResult{intent:"computer.open".into(),action:"blocked".into(),message:if language.eq_ignore_ascii_case("ar"){"فتح البرامج محظور حتى تسمح به من إعدادات الأمان.".into()}else{"Opening applications is blocked until you allow it in Safety settings.".into()},requires_confirmation:true}))}
 match computer::open_target(target){Ok(label)=>{permissions::add_log(path,"computer.open",&label)?;Ok(Some(AgentResult{intent:"computer.open".into(),action:"opened".into(),message:if language.eq_ignore_ascii_case("ar"){format!("تم فتح {label}.")}else{format!("Opened {label}.")},requires_confirmation:false}))},Err(e)=>Ok(Some(AgentResult{intent:"computer.open".into(),action:"rejected".into(),message:e,requires_confirmation:false}))}
}
